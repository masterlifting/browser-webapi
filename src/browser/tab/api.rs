use headless_chrome::Element;
use headless_chrome::{Browser, Tab};
use lazy_static::lazy_static;
use std::collections::HashMap;
use std::sync::Arc;
use std::sync::Mutex;
use url::Url;
use uuid::Uuid;

use crate::browser::tab::dto::{FillRequest, GetElement, OpenRequest};
use crate::web_api::models::{Error, ErrorInfo};

lazy_static! {
  static ref TABS: Mutex<HashMap<String, Arc<Tab>>> = Mutex::new(HashMap::new());
}

fn find_tab(tab_id: &str) -> Result<Arc<Tab>, Error> {
  TABS
    .lock()
    .unwrap()
    .get(tab_id)
    .cloned()
    .ok_or_else(|| Error::NotFound(format!("tab_id {}", tab_id)))
}

fn find_element<'a>(tab: &'a Arc<Tab>, selector: &'a str) -> Result<Element<'a>, Error> {
  tab.wait_for_element(selector).map_err(|e| {
    Error::Operation(ErrorInfo {
      message: format!("Failed to find element with selector '{}': {}", selector, e),
      code: None,
    })
  })
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

  fn create_response(tab: Arc<Tab>) -> Result<String, Error> {
    let tab_id = Uuid::new_v4().to_string();
    TABS.lock().unwrap().insert(tab_id.clone(), tab);
    Ok(tab_id)
  }

  parse_url(&req.url)
    .and_then(|url| open_new_tab(url, browser))
    .and_then(|(url, tab)| navigate_to_url(tab, url))
    .and_then(wait_for_navigation)
    .and_then(create_response)
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

  find_tab(tab_id)
    .and_then(close_tab)
    .and_then(|tab| remove_tab(tab_id, tab))
}

pub async fn fill(tab_id: &str, req: FillRequest) -> Result<(), Error> {
  fn fill_element(element: &Element, value: &str) -> Result<(), String> {
    element
      .type_into(value)
      .map(|_| ())
      .map_err(|e| format!("Failed to fill input element '{}': {}", &element.value, e))
  }

  find_tab(tab_id).and_then(|tab| {
    req.inputs.iter().try_for_each(|post_element| {
      find_element(&tab, &post_element.selector).and_then(|element| {
        fill_element(&element, &post_element.value).map_err(|e| {
          Error::Operation(ErrorInfo {
            message: e,
            code: None,
          })
        })
      })
    })
  })
}

pub async fn click(tab_id: &str, req: GetElement) -> Result<(), Error> {
  find_tab(tab_id).and_then(|tab| {
    find_element(&tab, &req.selector).and_then(|element| {
      element.click().map(|_| ()).map_err(|e| {
        Error::Operation(ErrorInfo {
          message: format!("Failed to click element '{}': {}", req.selector, e),
          code: None,
        })
      })
    })
  })
}
