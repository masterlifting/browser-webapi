use chaser_oxide::{ChaserPage, Element};
use futures::TryFutureExt;
use futures::future;
use std::sync::Arc;
use tokio::time::{Duration, Instant, sleep, timeout};

use crate::browser::element::dto::{ClickDto, ExecuteDto, ExistsDto, ExtractDto};
use crate::browser::tab;
use crate::models::{Error, ErrorInfo};

/// Finds an element in the page using the given selector.
///
/// # Behavior
///
/// - Searches for the selector in the current page.
/// - Returns the first matching element.
///
/// # Arguments
///
/// - `chaser`: The page wrapper to query against.
/// - `selector`: A CSS selector.
///
/// # Errors
///
/// Returns an `Error` if the element with the selector is not found or waiting for it fails.
///
/// # Examples
///
/// ```ignore
/// let element = api::find(&page, "#login-button").await?;
/// ```
pub async fn find(chaser: &Arc<ChaserPage>, selector: &str) -> Result<Element, Error> {
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

/// Tries to find the element matched by the selector on the page.
///
/// # Behavior
///
/// - Searches for the selector in the current page.
/// - Returns `None` instead of propagating errors.
///
/// # Arguments
///
/// - `chaser`: The page wrapper to query against.
/// - `selector`: A CSS selector.
///
/// # Examples
///
/// ```ignore
/// if let Some(element) = api::try_find(page.clone(), ".toast").await {
///   element.click().await?;
/// }
/// ```
pub async fn try_find(chaser: Arc<ChaserPage>, selector: &str) -> Option<Element> {
  chaser.raw_page().find_element(selector).await.ok()
}

/// Fills the element matched by the selector with the given value.
///
/// # Behavior
///
/// - Encodes selector and value as JSON.
/// - Executes a script to focus and clear the element.
/// - Types the requested value into the focused element.
///
/// # Arguments
///
/// - `chaser`: The page wrapper to query against.
/// - `selector`: A CSS selector.
/// - `value`: The value to assign.
///
/// # Errors
///
/// Returns an `Error` if filling the element fails or the selector is not found.
///
/// # Examples
///
/// ```ignore
/// api::fill(page.clone(), "input[name='email']", "user@example.com").await?;
/// ```
pub async fn fill(chaser: Arc<ChaserPage>, selector: &str, value: &str) -> Result<(), Error> {
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
    .and_then(move |selector_json| async move { Ok((chaser, selector, selector_json, value)) })
    .and_then(prepare_element)
    .and_then(type_value)
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

  tab::api::find(tab_id)
    .and_then({
      let selector = dto.selector;
      |chaser| async move {
        find(&chaser, selector.as_str())
          .await
          .map(|element| (chaser, element, selector))
      }
    })
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
  tab::api::find(tab_id)
    .and_then(|page| async move { Ok(try_find(page, &dto.selector).await.is_some()) })
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
  let selector = dto.selector;

  tab::api::find(tab_id)
    .and_then(|page| async move {
      find(&page, &selector)
        .await
        .map(|element| (selector, element))
    })
    .and_then(|(selector, element)| async move {
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
    })
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
  let selector = dto.selector;
  let function = dto.function;

  tab::api::find(tab_id)
    .and_then(|page| async move {
      match selector {
        Some(selector) => {
          find(&page, &selector).await?;
          Ok((page, function))
        }
        None => Ok((page, function)),
      }
    })
    .and_then(|(page, function)| async move {
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
    })
    .map_ok(|result| {
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
    .await
}
