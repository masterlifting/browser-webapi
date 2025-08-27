use headless_chrome::{Browser, Tab};
use lazy_static::lazy_static;
use std::collections::HashMap;
use std::sync::Arc;
use std::sync::Mutex;
use url::Url;
use uuid::Uuid;

use crate::browser::element;
use crate::browser::tab::dto::{FillRequest, OpenRequest};
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

pub async fn open(browser: Arc<Browser>, req: OpenRequest) -> Result<String, Error> {
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

  fn navigate_to_url(tab: Arc<Tab>, url: Url) -> Result<Arc<Tab>, Error> {
    tab
      .navigate_to(&url.as_str())
      .map(|_| (tab.clone()))
      .map_err(|e| {
        Error::Operation(ErrorInfo {
          message: format!("Failed to navigate to URL: {}", e),
          code: None,
        })
      })
  }

  fn wait_for_navigation(tab: Arc<Tab>) -> Result<Arc<Tab>, Error> {
    let tab_clone = tab.clone();
    tab
      .wait_until_navigated()
      .map(|_| (tab_clone))
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

  parse_url(&req.url)
    .and_then(|url| open_new_tab(url, browser))
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

pub async fn fill(tab_id: &str, req: FillRequest) -> Result<(), Error> {
  find(tab_id).and_then(|tab| {
    req.inputs.iter().try_for_each(|post_element| {
      element::api::find(&tab, &post_element.selector).and_then(|element| {
        element::api::fill(&element, &post_element.value).map_err(|e| {
          Error::Operation(ErrorInfo {
            message: e,
            code: None,
          })
        })
      })
    })
  })
}
