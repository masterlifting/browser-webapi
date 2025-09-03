use headless_chrome::{Browser, Tab};
use std::collections::HashMap;
use std::sync::Arc;
use std::sync::LazyLock;
use std::sync::Mutex;
use url::Url;
use uuid::Uuid;

use crate::browser::element;
use crate::browser::tab::dto::{FillDto, OpenDto};
use crate::models::{Error, ErrorInfo};

static TABS: LazyLock<Mutex<HashMap<String, Arc<Tab>>>> =
  LazyLock::new(|| Mutex::new(HashMap::new()));

/// Finds a tab by its ID.
///
/// # Errors
///
/// Returns `Error::NotFound` if the tab with the given ID does not exist.
///
/// # Panics
///
/// Panics if the internal mutex is poisoned.
pub fn find(tab_id: &str) -> Result<Arc<Tab>, Error> {
  TABS
    .lock()
    .unwrap()
    .get(tab_id)
    .cloned()
    .ok_or_else(|| Error::NotFound(format!("tab_id {tab_id}")))
}

/// Attempts to find a tab by its ID without panicking on not found.
///
/// # Panics
///
/// Panics if the internal mutex is poisoned.
#[must_use]
pub fn try_find(tab_id: &str) -> Option<Arc<Tab>> {
  TABS.lock().unwrap().get(tab_id).cloned()
}

/// Opens a new tab with the specified URL and applies anti-detection measures.
///
/// # Errors
///
/// Returns an `Error` if:
/// * The URL is invalid
/// * Creating a new tab fails
/// * JavaScript evaluation fails
/// * Navigation to the URL fails
/// * Waiting for navigation fails
///
/// # Panics
///
/// Panics if the internal mutex is poisoned.
pub fn open(browser: Arc<Browser>, dto: OpenDto) -> Result<String, Error> {
  fn parse_url(url: &str) -> Result<Url, Error> {
    Url::parse(url).map_err(|e| {
      Error::Operation(ErrorInfo {
        message: format!("Invalid URL: {e}"),
        code: None,
      })
    })
  }

  fn open_new_tab(url: Url, browser: Arc<Browser>) -> Result<(Arc<Tab>, Url), Error> {
    browser.new_tab().map(|tab| (tab, url)).map_err(|e| {
      Error::Operation(ErrorInfo {
        message: format!("Failed to create new tab: {e}"),
        code: None,
      })
    })
  }

  fn navigate_to_url((tab, url): (Arc<Tab>, Url)) -> Result<Arc<Tab>, Error> {
    match tab.navigate_to(url.as_str()) {
      Ok(_) => Ok(tab),
      Err(e) => Err(Error::Operation(ErrorInfo {
        message: format!("Failed to navigate to URL: {e}"),
        code: None,
      })),
    }
  }

  fn wait_for_navigation(tab: Arc<Tab>) -> Result<Arc<Tab>, Error> {
    match tab.wait_until_navigated() {
      Ok(_) => Ok(tab),
      Err(e) => Err(Error::Operation(ErrorInfo {
        message: format!("Failed to wait for navigation: {e}"),
        code: None,
      })),
    }
  }

  fn store_tab(tab: Arc<Tab>) -> String {
    let tab_id = Uuid::new_v4().to_string();
    TABS.lock().unwrap().insert(tab_id.clone(), tab);
    tab_id
  }

  parse_url(&dto.url)
    .and_then(|url| open_new_tab(url, browser))
    .and_then(navigate_to_url)
    .and_then(wait_for_navigation)
    .map(store_tab)
}

/// Closes the tab with the specified ID.
///
/// # Errors
///
/// Returns an `Error` if:
/// * The tab with the given ID does not exist
/// * Closing the tab fails
///
/// # Panics
///
/// Panics if the internal mutex is poisoned.
pub fn close(tab_id: &str) -> Result<(), Error> {
  fn close_tab(tab: Arc<Tab>) -> Result<(), Error> {
    tab.close(true).map(|_| ()).map_err(|e| {
      Error::Operation(ErrorInfo {
        message: format!("Failed to close tab: {e}"),
        code: None,
      })
    })
  }

  fn remove_tab(tab_id: &str) {
    TABS.lock().unwrap().remove(tab_id);
  }

  find(tab_id).and_then(close_tab).map(|_| remove_tab(tab_id))
}

/// Fills form inputs in the tab with the specified values.
///
/// # Errors
///
/// Returns an `Error` if:
/// * The tab with the given ID does not exist
/// * Finding an element fails
/// * Filling an element fails
///
/// # Panics
///
/// Panics if the internal mutex is poisoned.
pub fn fill(tab_id: &str, dto: FillDto) -> Result<(), Error> {
  find(tab_id).and_then(|tab| {
    dto.inputs.iter().try_for_each(|input| {
      element::api::find(&tab, &input.selector).and_then(|element| {
        element::api::fill(&element, &input.value).map_err(|e| {
          Error::Operation(ErrorInfo {
            message: e,
            code: None,
          })
        })
      })
    })
  })
}

/// Applies human-like behaviors to the tab to avoid detection.
///
/// # Errors
///
/// Returns an `Error` if:
/// * The tab with the given ID does not exist
/// * JavaScript evaluation fails
///
/// # Panics
///
/// Panics if the internal mutex is poisoned.
pub fn humanize(tab_id: &str) -> Result<(), Error> {
  find(tab_id)
    .and_then(|tab| {
      tab
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
          true,
        )
        .map(|_| tab.clone())
        .map_err(|e| {
          Error::Operation(ErrorInfo {
            message: format!("Failed to humanize tab: {e}"),
            code: None,
          })
        })
    })
    .map(|_| ())
}
