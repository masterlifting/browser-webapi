use chaser_oxide::cdp::browser_protocol::network::{Cookie, DeleteCookiesParams};
use chaser_oxide::page::ScreenshotParams;
use chaser_oxide::{Browser, ChaserPage, ChaserProfile, Element};
use futures::TryFutureExt;
use futures::future;
use futures::stream::{self, TryStreamExt};
use std::collections::HashMap;
use std::sync::Arc;
use std::sync::LazyLock;
use tokio::sync::Mutex;
use tokio::time::{Duration, Instant, sleep, timeout};
use url::Url;
use uuid::Uuid;

use crate::browser::tab::dto::{ClickDto, ExecuteDto, ExistsDto, ExtractDto, FillDto, OpenDto};
use crate::models::{Error, ErrorInfo};

static TABS: LazyLock<Mutex<HashMap<String, Arc<ChaserPage>>>> =
  LazyLock::new(|| Mutex::new(HashMap::new()));

/// Finds a tab by its ID.
///
/// # Arguments
///
/// - `tab_id`: The ID of the tab to look up.
///
/// # Errors
///
/// Returns `Error::NotFound` if the tab with the given ID does not exist.
///
/// # Examples
///
/// ```ignore
/// let page = api::find(tab_id).await?;
/// ```
pub async fn find(tab_id: &str) -> Result<Arc<ChaserPage>, Error> {
  TABS
    .lock()
    .await
    .get(tab_id)
    .cloned()
    .ok_or_else(|| Error::NotFound(format!("tab_id {tab_id}")))
}

/// Opens a new tab with the specified URL and applies anti-detection measures.
///
/// # Behavior
///
/// - Creates a new page and wraps it in `ChaserPage`.
/// - Applies the Windows stealth profile before navigation.
/// - Navigates to the requested URL.
/// - Schedules automatic tab closure after `dto.expiration` seconds.
///
/// # Arguments
///
/// - `browser`: The shared browser instance.
/// - `dto`: Open payload including the URL and expiration.
///
/// # Errors
///
/// Returns an `Error` if:
/// - The URL is invalid.
/// - Creating a new tab fails.
/// - Applying the stealth profile fails.
/// - Navigation to the URL fails.
///
/// # Examples
///
/// ```ignore
/// let tab_id = api::open(browser, OpenDto { url: "https://example.com".into(), expiration: 60 }).await?;
/// ```
pub async fn open(browser: Arc<Browser>, dto: OpenDto) -> Result<String, Error> {
  fn schedule_auto_close(tab_id: String, expiration: u64) {
    tokio::spawn(async move {
      tokio::time::sleep(Duration::from_secs(expiration)).await;
      match close(&tab_id).await {
        Ok(()) => tracing::info!("Tab {tab_id} expired after {expiration} seconds"),
        Err(e) => tracing::warn!("Failed to auto-close tab {tab_id} after expiration: {e}"),
      }
    });
  }
  #[inline]
  fn parse_url(url: &str) -> Result<Url, Error> {
    Url::parse(url).map_err(|e| {
      Error::Operation(ErrorInfo {
        message: format!("Invalid URL: {e}"),
        code: None,
      })
    })
  }
  async fn create_new_tab(
    (url, browser): (Url, Arc<Browser>),
  ) -> Result<(Arc<ChaserPage>, Url), Error> {
    let page = browser
      .new_page("about:blank")
      .map_err(|e| {
        Error::Operation(ErrorInfo {
          message: format!("Failed to create new page: {e}"),
          code: None,
        })
      })
      .await?;

    let chaser = Arc::new(ChaserPage::new(page));
    let profile = ChaserProfile::linux().build();

    match chaser.apply_profile(&profile).await {
      Ok(()) => Ok((chaser, url)),
      Err(e) => {
        close_page(chaser).await?;
        Err(Error::Operation(ErrorInfo {
          message: format!("Failed to apply stealth profile: {e}"),
          code: None,
        }))
      }
    }
  }
  async fn navigate_to_url(
    (chaser, url): (Arc<ChaserPage>, Url),
  ) -> Result<Arc<ChaserPage>, Error> {
    match chaser.goto(url.as_str()).await {
      Ok(()) => Ok(chaser),
      Err(e) => {
        close_page(chaser).await?;
        Err(Error::Operation(ErrorInfo {
          message: format!("Failed to navigate to URL: {e}"),
          code: None,
        }))
      }
    }
  }
  async fn store_tab(page: Arc<ChaserPage>) -> Result<String, Error> {
    let tab_id = Uuid::new_v4().to_string();
    let mut tabs = TABS.lock().await;
    tabs.insert(tab_id.clone(), page);
    Ok(tab_id)
  }

  future::ready(parse_url(dto.url.as_str()))
    .map_ok(move |url| (url, browser))
    .and_then(create_new_tab)
    .and_then(navigate_to_url)
    .and_then(store_tab)
    .map_ok(|tab_id| {
      schedule_auto_close(tab_id.clone(), dto.bounded_expiration());
      tab_id
    })
    .await
}

/// Closes the tab with the specified ID.
///
/// # Behavior
///
/// - Removes the tab from the in-memory store.
/// - Clears cookies for the tab's current URL.
/// - Closes the underlying page.
///
/// # Arguments
///
/// - `tab_id`: The ID of the tab to close.
///
/// # Errors
///
/// Returns an `Error` if:
/// - The tab with the given ID does not exist.
/// - Reading or deleting cookies fails.
/// - Closing the tab fails.
///
/// # Examples
///
/// ```ignore
/// api::close(tab_id).await?;
/// ```
pub async fn close(tab_id: &str) -> Result<(), Error> {
  async fn remove_tab(tab_id: &str) -> Result<(String, Arc<ChaserPage>), Error> {
    TABS
      .lock()
      .await
      .remove(tab_id)
      .map(|page| (tab_id.to_string(), page))
      .ok_or_else(|| Error::NotFound(format!("tab_id {tab_id}")))
  }
  async fn get_cookies(
    (tab_id, chaser): (String, Arc<ChaserPage>),
  ) -> Result<(String, Vec<Cookie>, Arc<ChaserPage>, Option<Error>), Error> {
    match chaser.raw_page().get_cookies().await {
      Ok(cookies) => Ok((tab_id, cookies, chaser, None)),
      Err(e) => Ok((
        tab_id,
        Vec::new(),
        chaser,
        Some(Error::Operation(ErrorInfo {
          message: format!("Failed to get cookies: {e}"),
          code: None,
        })),
      )),
    }
  }
  async fn clear_cookies(
    (tab_id, cookies, chaser, cookie_error): (String, Vec<Cookie>, Arc<ChaserPage>, Option<Error>),
  ) -> Result<(Arc<ChaserPage>, Option<Error>), Error> {
    if cookie_error.is_some() {
      return Ok((chaser, cookie_error));
    }

    let to_delete = cookies
      .iter()
      .map(|cookie| {
        DeleteCookiesParams::builder()
          .name(cookie.name.clone())
          .domain(cookie.domain.clone())
          .path(cookie.path.clone())
          .build()
          .unwrap_or_else(|_| DeleteCookiesParams::new(cookie.name.clone()))
      })
      .collect::<Vec<_>>();

    if !to_delete.is_empty() {
      let deleted_count = to_delete.len();

      return match chaser.raw_page().delete_cookies(to_delete).await {
        Ok(_) => {
          tracing::info!("Deleted {} cookies for tab {}", deleted_count, tab_id);
          Ok((chaser, None))
        }
        Err(e) => Ok((
          chaser,
          Some(Error::Operation(ErrorInfo {
            message: format!("Failed to delete cookies: {e}"),
            code: None,
          })),
        )),
      };
    }

    Ok((chaser, None))
  }
  async fn close_tab(
    (chaser, cookie_error): (Arc<ChaserPage>, Option<Error>),
  ) -> Result<(), Error> {
    match close_page(chaser).await {
      Ok(()) => match cookie_error {
        Some(cookie_error) => Err(cookie_error),
        None => Ok(()),
      },
      Err(close_error) => match cookie_error {
        Some(cookie_error) => Err(Error::Operation(ErrorInfo {
          message: format!(
            "Failed to close tab after cookie cleanup error. cookie_error: {cookie_error}; close_error: {close_error}"
          ),
          code: None,
        })),
        None => Err(close_error),
      },
    }
  }

  remove_tab(tab_id)
    .and_then(get_cookies)
    .and_then(clear_cookies)
    .and_then(close_tab)
    .await
}

/// Clicks the element with the given selector in the tab.
///
/// # Behavior
///
/// - Resolves the tab and element by ID and selector.
/// - Clicks the element.
/// - Waits for navigation (best-effort, timeout) and then waits for the URL to stabilize
///   to avoid returning while a redirect chain is still in progress.
///
/// # Arguments
///
/// - `tab_id`: The ID of the tab to operate on.
/// - `dto`: Click payload including the selector.
///
/// # Errors
///
/// Returns an `Error` if:
/// - The tab is not found.
/// - The element is not found.
/// - Clicking the element fails.
/// - Waiting for navigation fails.
///
/// # Examples
///
/// ```ignore
/// let title = api::click(tab_id, ClickDto { selector: "#submit".into() }).await?;
/// ```
pub async fn click(tab_id: &str, dto: ClickDto) -> Result<String, Error> {
  async fn resolve_click_target(
    (chaser, selector): (Arc<ChaserPage>, String),
  ) -> Result<(Arc<ChaserPage>, Element, String), Error> {
    find_element(&chaser, selector.as_str())
      .await
      .map(|element| (chaser, element, selector))
  }
  async fn click_element(
    (chaser, element, selector): (Arc<ChaserPage>, Element, String),
  ) -> Result<(Arc<ChaserPage>, String), Error> {
    element.click().await.map_err(|e| {
      Error::Operation(ErrorInfo {
        message: format!("Failed to click element '{}': {}", selector, e),
        code: None,
      })
    })?;

    Ok((chaser, selector))
  }
  async fn wait_for_possible_navigation(
    (chaser, selector): (Arc<ChaserPage>, String),
  ) -> Result<(Arc<ChaserPage>, String), Error> {
    // `wait_for_navigation` can return before the browser has finished a redirect chain,
    // and some clicks do not navigate at all. We treat a timeout as "no navigation".
    match timeout(
      Duration::from_secs(10),
      chaser.raw_page().wait_for_navigation(),
    )
    .await
    {
      Ok(Ok(_)) => Ok((chaser, selector)),
      Ok(Err(e)) => Err(Error::Operation(ErrorInfo {
        message: format!(
          "Failed while waiting for navigation after click '{}': {}",
          selector, e
        ),
        code: None,
      })),
      Err(_) => Ok((chaser, selector)),
    }
  }
  async fn wait_for_stable_location(
    (chaser, selector): (Arc<ChaserPage>, String),
  ) -> Result<(Arc<ChaserPage>, String), Error> {
    async fn try_get_href(chaser: &Arc<ChaserPage>) -> Option<String> {
      chaser
        .raw_page()
        .evaluate("window.location.href")
        .await
        .ok()
        .and_then(|result| {
          result
            .value()
            .and_then(|val| val.as_str())
            .map(|s| s.to_string())
        })
    }

    // Best-effort: if we can't read href, don't fail the click request.
    let Some(mut last_href) = try_get_href(&chaser).await else {
      return Ok((chaser, selector));
    };

    let started = Instant::now();
    let mut last_change = Instant::now();

    let max_wait = Duration::from_secs(15);
    let stable_for = Duration::from_millis(750);
    let poll_every = Duration::from_millis(150);

    loop {
      if Instant::now().duration_since(started) >= max_wait {
        break;
      }

      if Instant::now().duration_since(last_change) >= stable_for {
        break;
      }

      sleep(poll_every).await;

      let Some(href) = try_get_href(&chaser).await else {
        break;
      };

      if href != last_href {
        last_href = href;
        last_change = Instant::now();
      }
    }

    Ok((chaser, selector))
  }
  async fn get_title((chaser, _selector): (Arc<ChaserPage>, String)) -> Result<String, Error> {
    Ok(
      chaser
        .raw_page()
        .evaluate("({ href: window.location.href, title: document.title })")
        .await
        .map(|result| {
          let value = result.value().cloned().unwrap_or_default();

          value
            .get("title")
            .and_then(|val| val.as_str())
            .unwrap_or("<unknown>")
            .to_string()
        })
        .unwrap_or_else(|_| "<unknown>".to_string()),
    )
  }

  let selector = dto.selector;

  find(tab_id)
    .map_ok(move |chaser| (chaser, selector))
    .and_then(resolve_click_target)
    .and_then(click_element)
    .and_then(wait_for_possible_navigation)
    .and_then(wait_for_stable_location)
    .and_then(get_title)
    .await
}

/// Checks whether an element with the selector exists in the tab.
///
/// # Behavior
///
/// - Resolves the tab by ID.
/// - Returns `true` if the element is found, otherwise `false`.
///
/// # Arguments
///
/// - `tab_id`: The ID of the tab to operate on.
/// - `dto`: Exists payload including the selector.
///
/// # Examples
///
/// ```ignore
/// let has_modal = api::exists(tab_id, ExistsDto { selector: "#modal".into() }).await;
/// ```
pub async fn exists(tab_id: &str, dto: ExistsDto) -> bool {
  async fn element_exists((page, selector): (Arc<ChaserPage>, String)) -> Result<bool, Error> {
    find_element(&page, selector.as_str()).await.map(|_| true)
  }

  let selector = dto.selector;

  find(tab_id)
    .map_ok(move |page| (page, selector))
    .and_then(element_exists)
    .await
    .unwrap_or(false)
}

/// Extracts content from the element with the given selector in the tab.
/// Returns the inner text of the element.
/// If the element has no text content, an empty string is returned.
///
/// # Behavior
///
/// - Resolves the tab and element by ID and selector.
/// - Returns the element text or an empty string.
///
/// # Arguments
///
/// - `tab_id`: The ID of the tab to operate on.
/// - `dto`: Extract payload including the selector.
///
/// # Errors
///
/// Returns an `Error` if:
/// - The tab is not found.
/// - The element is not found.
/// - Getting the content fails.
///
/// # Examples
///
/// ```ignore
/// let text = api::extract(tab_id, ExtractDto { selector: "h1".into() }).await?;
/// ```
pub async fn extract(tab_id: &str, dto: ExtractDto) -> Result<String, Error> {
  async fn resolve_extract_target(
    (page, selector): (Arc<ChaserPage>, String),
  ) -> Result<(String, Element), Error> {
    find_element(&page, selector.as_str())
      .await
      .map(|element| (selector, element))
  }
  async fn extract_inner_text((selector, element): (String, Element)) -> Result<String, Error> {
    element
      .inner_text()
      .await
      .map_err(|e| {
        Error::Operation(ErrorInfo {
          message: format!("Failed to get content of element '{}': {}", selector, e),
          code: None,
        })
      })
      .map(|opt| opt.unwrap_or_default())
  }

  let selector = dto.selector;

  find(tab_id)
    .map_ok(move |page| (page, selector))
    .and_then(resolve_extract_target)
    .and_then(extract_inner_text)
    .await
}

/// Executes JavaScript code on the element with the given selector in the tab,
/// or on the tab itself if no selector is provided, and returns the string representation of the result.
///
/// # Behavior
///
/// - Resolves the tab by ID.
/// - Optionally verifies the element exists if a selector is provided.
/// - Evaluates the JavaScript and returns a string representation.
///
/// # Arguments
///
/// - `tab_id`: The ID of the tab to operate on.
/// - `dto`: Execute payload including optional selector and JavaScript.
///
/// # Errors
///
/// Returns an `Error` if:
/// - The tab is not found.
/// - The element is not found (if selector is provided).
/// - Evaluating the JavaScript fails.
///
/// # Examples
///
/// ```ignore
/// let result = api::execute(
///   tab_id,
///   ExecuteDto {
///     selector: None,
///     function: "document.title".into(),
///   },
/// )
/// .await?;
/// ```
pub async fn execute(tab_id: &str, dto: ExecuteDto) -> Result<String, Error> {
  async fn resolve_execution_target(
    (page, selector, function): (Arc<ChaserPage>, Option<String>, String),
  ) -> Result<(Arc<ChaserPage>, String), Error> {
    match selector {
      Some(selector) => find_element(&page, selector.as_str())
        .await
        .map(|_| (page, function)),
      None => Ok((page, function)),
    }
  }
  async fn evaluate_function((page, function): (Arc<ChaserPage>, String)) -> Result<String, Error> {
    page
      .raw_page()
      .evaluate(function.as_str())
      .await
      .map_err(|e| {
        Error::Operation(ErrorInfo {
          message: format!("Failed to evaluate JS: {}", e),
          code: None,
        })
      })
      .map(|result| {
        result
          .value()
          .map(|val| {
            val
              .as_str()
              .map(|s| s.to_string())
              .unwrap_or_else(|| val.to_string())
          })
          .unwrap_or_else(|| "unit".to_string())
      })
  }

  let selector = dto.selector;
  let function = dto.function;

  find(tab_id)
    .map_ok(move |page| (page, selector, function))
    .and_then(resolve_execution_target)
    .and_then(evaluate_function)
    .await
}

/// Fills form inputs in the tab with the specified values.
///
/// # Behavior
///
/// - Resolves the tab by ID.
/// - Fills inputs sequentially (in request order).
///
/// # Arguments
///
/// - `tab_id`: The ID of the tab to operate on.
/// - `dto`: Fill payload including selectors and values.
///
/// # Errors
///
/// Returns an `Error` if:
/// - The tab with the given ID does not exist.
/// - Finding an element fails.
/// - Filling an element fails.
///
/// # Examples
///
/// ```ignore
/// api::fill(tab_id, FillDto { inputs: vec![/* ... */] }).await?;
/// ```
pub async fn fill(tab_id: &str, dto: FillDto) -> Result<(), Error> {
  async fn fill_element(chaser: Arc<ChaserPage>, selector: &str, value: &str) -> Result<(), Error> {
    #[inline]
    fn encode_selector(selector: &str) -> Result<String, Error> {
      serde_json::to_string(selector).map_err(|e| {
        Error::Operation(ErrorInfo {
          message: format!("Failed to encode selector '{selector}': {e}"),
          code: None,
        })
      })
    }
    async fn prepare_element(
      (page, selector, selector_json, value): (Arc<ChaserPage>, String, String, String),
    ) -> Result<(Arc<ChaserPage>, String, String), Error> {
      let script = format!(
        r#"(function() {{
        const el = document.querySelector({selector});
        if (!el) {{ return "not_found"; }}
        el.focus();
        if ("value" in el) {{
          el.value = "";
          el.setAttribute("value", "");
        }} else if (el.isContentEditable) {{
          el.textContent = "";
        }}
        return "ok";
      }})()"#,
        selector = selector_json
      );

      let result = page
        .raw_page()
        .evaluate(script.as_str())
        .await
        .map_err(|e| {
          Error::Operation(ErrorInfo {
            message: format!("Failed to prepare element '{selector}': {e}"),
            code: None,
          })
        })?;

      let status = result.value().and_then(|val| val.as_str()).unwrap_or("");
      if status != "ok" {
        return Err(Error::Operation(ErrorInfo {
          message: format!("Failed to find element with selector '{selector}'"),
          code: None,
        }));
      }

      Ok((page, selector, value))
    }
    async fn type_value(
      (page, selector, value): (Arc<ChaserPage>, String, String),
    ) -> Result<(), Error> {
      if value.is_empty() {
        return Ok(());
      }

      page
        // Deterministic typing: callers expect the provided value to be entered exactly.
        .type_text(value.as_str())
        .await
        .map_err(|e| {
          Error::Operation(ErrorInfo {
            message: format!("Failed to type into element '{selector}': {e}"),
            code: None,
          })
        })
    }

    let selector = selector.to_string();
    let value = value.to_string();

    future::ready(encode_selector(selector.as_str()))
      .map_ok(move |selector_json| (chaser, selector, selector_json, value))
      .and_then(prepare_element)
      .and_then(type_value)
      .await
  }
  async fn fill_inputs((chaser, dto): (Arc<ChaserPage>, FillDto)) -> Result<(), Error> {
    stream::iter(dto.inputs.into_iter().map(Ok::<_, Error>))
      .try_for_each(|input| {
        let chaser = chaser.clone();
        async move { fill_element(chaser, input.selector.as_str(), input.value.as_str()).await }
      })
      .await
  }

  find(tab_id)
    .map_ok(move |chaser| (chaser, dto))
    .and_then(fill_inputs)
    .await
}

/// Applies human-like behaviors to the tab to avoid detection.
///
/// # Behavior
///
/// - Resolves the tab by ID.
/// - Runs a small script to resize, scroll, and dispatch mouse movement.
///
/// # Arguments
///
/// - `tab_id`: The ID of the tab to operate on.
///
/// # Errors
///
/// Returns an `Error` if:
/// - The tab with the given ID does not exist.
/// - JavaScript evaluation fails.
///
/// # Examples
///
/// ```ignore
/// api::humanize(tab_id).await?;
/// ```
pub async fn humanize(tab_id: &str) -> Result<(), Error> {
  async fn apply_humanize(page: Arc<ChaserPage>) -> Result<(), Error> {
    page
      .evaluate(
        r"
            if (window.innerWidth > 800) {
              window.resizeTo(window.innerWidth + Math.floor(Math.random() * 100) - 50, window.innerHeight + Math.floor(Math.random() * 100) - 50);
            }
            window.scrollTo(0, Math.floor(Math.random() * 100));
            Object.defineProperty(navigator, 'hardwareConcurrency', { get: () => Math.floor(Math.random() * 8) + 4 });
            document.dispatchEvent(new MouseEvent('mousemove', { clientX: Math.random() * window.innerWidth, clientY: Math.random() * window.innerHeight }));
            true
          ",
      )
      .await
      .map(|_| ())
      .map_err(|e| {
        Error::Operation(ErrorInfo {
          message: format!("Failed to humanize tab: {e}"),
          code: None,
        })
      })
  }

  find(tab_id).and_then(apply_humanize).await
}

/// Returns a PNG screenshot of the tab.
///
/// # Behavior
///
/// - Resolves the tab by ID.
/// - Captures a PNG screenshot for the tab.
/// - Uses full-page mode to avoid clipped results.
///
/// # Arguments
///
/// - `tab_id`: The ID of the tab to capture.
///
/// # Errors
///
/// Returns an `Error` if:
/// - The tab with the given ID does not exist.
/// - Capturing the screenshot fails.
///
/// # Examples
///
/// ```ignore
/// let png = api::screenshot(tab_id).await?;
/// ```
pub async fn screenshot(tab_id: &str) -> Result<Vec<u8>, Error> {
  async fn capture_screenshot(page: Arc<ChaserPage>) -> Result<Vec<u8>, Error> {
    page
      .raw_page()
      .screenshot(ScreenshotParams::builder().full_page(true).build())
      .await
      .map_err(|e| {
        Error::Operation(ErrorInfo {
          message: format!("Failed to capture screenshot: {e}"),
          code: None,
        })
      })
  }

  find(tab_id).and_then(capture_screenshot).await
}
async fn close_page(chaser: Arc<ChaserPage>) -> Result<(), Error> {
  chaser.raw_page().clone().close().await.map_err(|e| {
    Error::Operation(ErrorInfo {
      message: format!("Failed to close tab: {e}"),
      code: None,
    })
  })
}
async fn find_element(chaser: &Arc<ChaserPage>, selector: &str) -> Result<Element, Error> {
  chaser
    .raw_page()
    .find_element(selector)
    .map_err(|e| {
      Error::Operation(ErrorInfo {
        message: format!("Failed to find element with selector '{selector}': {e}"),
        code: None,
      })
    })
    .await
}
