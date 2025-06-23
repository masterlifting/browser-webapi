pub mod models;

use actix_web::{HttpResponse, web};
use headless_chrome::{Browser, Tab};
use lazy_static::lazy_static;
use std::collections::HashMap;
use std::sync::Arc;
use std::sync::Mutex;
use url::Url;
use uuid::Uuid;

use crate::browser::page::models::{CloseRequest, LoadRequest, LoadResponse};
use crate::models::{Error, ErrorInfo};

lazy_static! {
  static ref TABS: Mutex<HashMap<String, Arc<Tab>>> = Mutex::new(HashMap::new());
}

fn try_find_tab(tab_id: &str) -> Result<Arc<Tab>, Error> {
  TABS
    .lock()
    .unwrap()
    .get(tab_id)
    .cloned()
    .ok_or_else(|| Error::NotFound(format!("tab_id {} not found", tab_id)))
}

pub async fn load(req: web::Json<LoadRequest>, browser: Arc<Browser>) -> HttpResponse {
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

  fn create_response(tab: Arc<Tab>) -> HttpResponse {
    let tab_id = Uuid::new_v4().to_string();
    TABS.lock().unwrap().insert(tab_id.clone(), tab);

    HttpResponse::Ok().json(LoadResponse { tab_id })
  }

  parse_url(&req.url)
    .and_then(|url| open_new_tab(url, browser))
    .and_then(|(url, tab)| navigate_to_url(tab, url))
    .and_then(wait_for_navigation)
    .map(create_response)
    .unwrap_or_else(|e| {
      HttpResponse::BadRequest().json(Error::Operation(ErrorInfo {
        message: e.to_string(),
        code: None,
      }))
    })
}

pub async fn close(req: web::Json<CloseRequest>) -> HttpResponse {
  fn close_tab(tab: Arc<Tab>) -> Result<Arc<Tab>, Error> {
    tab.close(true).map(|_| tab).map_err(|e| {
      Error::Operation(ErrorInfo {
        message: format!("Failed to close tab: {}", e),
        code: None,
      })
    })
  }

  fn remove_tab(tab_id: &str, tab: Arc<Tab>) -> HttpResponse {
    TABS.lock().unwrap().remove(tab_id);
    drop(tab);
    return HttpResponse::Ok().finish();
  }

  try_find_tab(&req.tab_id)
    .and_then(close_tab)
    .map(|tab| remove_tab(&req.tab_id, tab))
    .unwrap_or_else(|e| {
      HttpResponse::BadRequest().json(Error::Operation(ErrorInfo {
        message: e.to_string(),
        code: None,
      }))
    })
}
