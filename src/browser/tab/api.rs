use chaser_oxide::cdp::browser_protocol::network::{Cookie, DeleteCookiesParams};
use chaser_oxide::page::ScreenshotParams;
use chaser_oxide::{Browser, ChaserPage, ChaserProfile};
use futures::TryFutureExt;
use futures::future;
use futures::stream::{self, TryStreamExt};
use std::collections::HashMap;
use std::sync::Arc;
use std::sync::LazyLock;
use tokio::sync::Mutex;
use tokio::time::Duration;
use url::Url;
use uuid::Uuid;

use crate::browser::element;
use crate::browser::tab::dto::{FillDto, OpenDto};
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

/// Attempts to find a tab by its ID without panicking on not found.
///
/// # Arguments
///
/// - `tab_id`: The ID of the tab to look up.
///
/// # Examples
///
/// ```ignore
/// if let Some(page) = api::try_find(tab_id).await {
///   // use page
/// }
/// ```
#[must_use]
pub async fn try_find(tab_id: &str) -> Option<Arc<ChaserPage>> {
  TABS.lock().await.get(tab_id).cloned()
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
      if close(&tab_id).await.is_ok() {
        tracing::info!("Tab {tab_id} expired after {expiration} seconds");
      }
    });
  }

  fn parse_url(url: &str) -> Result<Url, Error> {
    Url::parse(url).map_err(|e| {
      Error::Operation(ErrorInfo {
        message: format!("Invalid URL: {e}"),
        code: None,
      })
    })
  }

  async fn create_new_tab(url: Url, browser: Arc<Browser>) -> Result<(ChaserPage, Url), Error> {
    browser
      .new_page("about:blank")
      .map_err(|e| {
        Error::Operation(ErrorInfo {
          message: format!("Failed to create new page: {e}"),
          code: None,
        })
      })
      .and_then(move |page| async move {
        let chaser = ChaserPage::new(page);
        let profile = ChaserProfile::linux().build();

        chaser.apply_profile(&profile).await.map_err(|e| {
          Error::Operation(ErrorInfo {
            message: format!("Failed to apply stealth profile: {e}"),
            code: None,
          })
        })?;

        Ok((chaser, url))
      })
      .await
  }

  async fn navigate_to_url((chaser, url): (ChaserPage, Url)) -> Result<ChaserPage, Error> {
    chaser.goto(url.as_str()).await.map_err(|e| {
      Error::Operation(ErrorInfo {
        message: format!("Failed to navigate to URL: {e}"),
        code: None,
      })
    })?;

    Ok(chaser)
  }

  async fn store_tab(page: ChaserPage) -> Result<String, Error> {
    let tab_id = Uuid::new_v4().to_string();
    let mut tabs = TABS.lock().await;
    tabs.insert(tab_id.clone(), Arc::new(page));
    Ok(tab_id)
  }

  future::ready(parse_url(&dto.url))
    .and_then(move |url| create_new_tab(url, browser))
    .and_then(navigate_to_url)
    .and_then(store_tab)
    .map_ok(|tab_id| {
      schedule_auto_close(tab_id.clone(), dto.expiration);
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
  async fn remove_tab(tab_id: &str) -> Result<(&str, Arc<ChaserPage>), Error> {
    TABS
      .lock()
      .await
      .remove(tab_id)
      .map(|page| (tab_id, page))
      .ok_or_else(|| Error::NotFound(format!("tab_id {tab_id}")))
  }

  async fn get_cookies(
    (tab_id, chaser): (&str, Arc<ChaserPage>),
  ) -> Result<(&str, Vec<Cookie>, Arc<ChaserPage>), Error> {
    let cookies = chaser.raw_page().get_cookies().await.map_err(|e| {
      Error::Operation(ErrorInfo {
        message: format!("Failed to get cookies: {e}"),
        code: None,
      })
    })?;

    Ok((tab_id, cookies, chaser))
  }

  async fn clear_cookies(
    (tab_id, cookies, chaser): (&str, Vec<Cookie>, Arc<ChaserPage>),
  ) -> Result<Arc<ChaserPage>, Error> {
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
      chaser
        .raw_page()
        .delete_cookies(to_delete.clone())
        .await
        .map_err(|e| {
          Error::Operation(ErrorInfo {
            message: format!("Failed to delete cookies: {e}"),
            code: None,
          })
        })?;

      tracing::info!("Deleted {} cookies for tab {}", to_delete.len(), tab_id);
    }

    Ok(chaser)
  }

  async fn close_page(chaser: Arc<ChaserPage>) -> Result<(), Error> {
    chaser.raw_page().clone().close().await.map_err(|e| {
      Error::Operation(ErrorInfo {
        message: format!("Failed to close tab: {e}"),
        code: None,
      })
    })
  }

  remove_tab(tab_id)
    .and_then(get_cookies)
    .and_then(clear_cookies)
    .and_then(close_page)
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
  async fn fill_input(chaser: Arc<ChaserPage>, selector: String, value: String) -> Result<(), Error> {
    element::api::fill(chaser, selector.as_str(), value.as_str()).await
  }

  find(tab_id)
    .and_then(|chaser| async move {
      stream::iter(dto.inputs.into_iter().map(Ok::<_, Error>))
        .try_for_each(|input| {
          let chaser = chaser.clone();
          async move { fill_input(chaser, input.selector, input.value).await }
        })
        .await
    })
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
  find(tab_id)
    .and_then(|page| async move {
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
    })
    .await
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
  find(tab_id)
    .and_then(|page| async move {
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
    })
    .await
}
