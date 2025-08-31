use headless_chrome::{Browser, Tab};
use lazy_static::lazy_static;
use std::collections::HashMap;
use std::sync::Arc;
use std::sync::Mutex;
use url::Url;
use uuid::Uuid;

use crate::browser::element;
use crate::browser::tab::dto::{FillDto, OpenDto};
use crate::models::{Error, ErrorInfo};

lazy_static! {
  static ref TABS: Mutex<HashMap<String, Arc<Tab>>> = Mutex::new(HashMap::new());
}

pub fn find(tab_id: &str) -> Result<Arc<Tab>, Error> {
  TABS
    .lock()
    .unwrap()
    .get(tab_id)
    .cloned()
    .ok_or_else(|| Error::NotFound(format!("tab_id {}", tab_id)))
}

pub fn try_find(tab_id: &str) -> Option<Arc<Tab>> {
  TABS.lock().unwrap().get(tab_id).cloned()
}

pub async fn open(browser: Arc<Browser>, dto: OpenDto) -> Result<String, Error> {
  fn parse_url(url: &str) -> Result<Url, Error> {
    Url::parse(url).map_err(|e| {
      Error::Operation(ErrorInfo {
        message: format!("Invalid URL: {}", e),
        code: None,
      })
    })
  }

  fn open_new_tab(url: Url, browser: Arc<Browser>) -> Result<(Url, Arc<Tab>), Error> {
    browser.new_tab().map(|tab| (url, tab)).map_err(|e| {
      Error::Operation(ErrorInfo {
        message: format!("Failed to create new tab: {}", e),
        code: None,
      })
    })
  }

  fn call_js(tab: Arc<Tab>, url: Url) -> Result<(Url, Arc<Tab>), Error> {
    tab
      .evaluate(
        r#"
            Object.defineProperty(navigator, 'webdriver', { get: () => undefined });
            window.navigator.chrome = { runtime: {} };
            Object.defineProperty(navigator, 'plugins', { get: () => [1,2,3,4,5] });
            Object.defineProperty(navigator, 'languages', { get: () => ['en-US','en'] });"#,
        true,
      )
      .map(|_| (url, tab.clone()))
      .map_err(|e| {
        Error::Operation(ErrorInfo {
          message: format!("Failed to call JS: {}", e),
          code: None,
        })
      })
  }

  fn navigate_to_url(tab: Arc<Tab>, url: Url) -> Result<Arc<Tab>, Error> {
    tab
      .navigate_to(&url.as_str())
      .map(|_| tab.clone())
      .map_err(|e| {
        Error::Operation(ErrorInfo {
          message: format!("Failed to navigate to URL: {}", e),
          code: None,
        })
      })
  }

  fn wait_for_navigation(tab: Arc<Tab>) -> Result<Arc<Tab>, Error> {
    tab
      .wait_until_navigated()
      .map(|_| tab.clone())
      .map_err(|e| {
        Error::Operation(ErrorInfo {
          message: format!("Failed to wait for navigation: {}", e),
          code: None,
        })
      })
  }

  fn add_tab(tab: Arc<Tab>) -> Result<String, Error> {
    let tab_id = Uuid::new_v4().to_string();
    TABS.lock().unwrap().insert(tab_id.clone(), tab);
    Ok(tab_id)
  }

  parse_url(&dto.url)
    .and_then(|url| open_new_tab(url, browser))
    .and_then(|(url, tab)| call_js(tab, url))
    .and_then(|(url, tab)| navigate_to_url(tab, url))
    .and_then(wait_for_navigation)
    .and_then(add_tab)
}

pub async fn close(tab_id: &str) -> Result<(), Error> {
  fn close_tab(tab: Arc<Tab>) -> Result<Arc<Tab>, Error> {
    tab.close(true).map(|_| tab).map_err(|e| {
      Error::Operation(ErrorInfo {
        message: format!("Failed to close tab: {}", e),
        code: None,
      })
    })
  }

  fn remove_tab(tab_id: &str, tab: Arc<Tab>) -> Result<(), Error> {
    TABS.lock().unwrap().remove(tab_id);
    drop(tab);
    Ok(())
  }

  find(tab_id)
    .and_then(close_tab)
    .and_then(|tab| remove_tab(tab_id, tab))
}

pub async fn fill(tab_id: &str, dto: FillDto) -> Result<(), Error> {
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
