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

  fn open_new_tab(url: Url, browser: Arc<Browser>) -> Result<(Url, Arc<Tab>), Error> {
    browser.new_tab().map(|tab| (url, tab)).map_err(|e| {
      Error::Operation(ErrorInfo {
        message: format!("Failed to create new tab: {e}"),
        code: None,
      })
    })
  }

  fn call_js(tab: Arc<Tab>, url: Url) -> Result<(Url, Arc<Tab>), Error> {
    tab
      .evaluate(
        r"
            Object.defineProperty(navigator, 'webdriver', { get: () => undefined });
            window.navigator.chrome = { runtime: {} };
            Object.defineProperty(navigator, 'plugins', { get: () => [1,2,3,4,5] });
            Object.defineProperty(navigator, 'languages', { get: () => ['en-US','en'] });",
        true,
      )
      .map(|_| (url, tab))
      .map_err(|e| {
        Error::Operation(ErrorInfo {
          message: format!("Failed to call JS: {e}"),
          code: None,
        })
      })
  }

  fn navigate_to_url(tab: Arc<Tab>, url: Url) -> Result<Arc<Tab>, Error> {
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

  fn add_tab(tab: Arc<Tab>) -> String {
    let tab_id = Uuid::new_v4().to_string();
    TABS.lock().unwrap().insert(tab_id.clone(), tab);
    tab_id
  }

  parse_url(&dto.url)
    .and_then(|url| open_new_tab(url, browser))
    .and_then(|(url, tab)| call_js(tab, url))
    .and_then(|(url, tab)| navigate_to_url(tab, url))
    .and_then(|tab| wait_for_navigation(tab))
    .map(add_tab)
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
  fn close_tab(tab: &Arc<Tab>) -> Result<Arc<Tab>, Error> {
    let tab = tab.clone();
    tab.close(true).map(|_| tab).map_err(|e| {
      Error::Operation(ErrorInfo {
        message: format!("Failed to close tab: {e}"),
        code: None,
      })
    })
  }

  fn remove_tab(tab_id: &str, _tab: &Arc<Tab>) {
    TABS.lock().unwrap().remove(tab_id);
  }

  find(tab_id)
    .and_then(|tab| close_tab(&tab))
    .map(|tab| remove_tab(tab_id, &tab))
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
