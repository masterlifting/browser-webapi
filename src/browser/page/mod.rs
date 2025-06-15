pub mod models;

use actix_web::{HttpResponse, web};
use headless_chrome::{Browser, Tab};
use lazy_static::lazy_static;
use std::collections::HashMap;
use std::sync::Arc;
use std::sync::Mutex;
use url::Url;
use uuid::Uuid;

use crate::browser::page::models::{LoadRequest, LoadResponse};
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
    .ok_or_else(|| Error::NotFound(format!("Browser tab with ID {} not found", tab_id)))
}

pub async fn load(req: web::Json<LoadRequest>, browser: Arc<Browser>) -> HttpResponse {
  let parse_url = |url: &str| {
    Url::parse(url).map_err(|e| {
      Error::Operation(ErrorInfo {
        message: format!("Invalid URL: {}", e),
        code: None,
      })
    })
  };

  let open_new_tab = |url: Url, browser: Arc<Browser>| {
    browser.new_tab().map_err(|e| {
      Error::Operation(ErrorInfo {
        message: format!("Failed to create new tab: {}", e),
        code: None,
      })
      .map(|tab| (url, tab))
    })
  };

  let navigate_to_url = |tab: Arc<Tab>, url: Url| {
    tab
      .navigate_to(&url.as_str())
      .map_err(|e| {
        Error::Operation(ErrorInfo {
          message: format!("Failed to navigate to URL: {}", e),
          code: None,
        })
      })
      .map(|_| (url, tab))
  };

  let wait_for_navigation = |tab: Arc<Tab>, url: Url| {
    tab
      .wait_until_navigated()
      .map_err(|e| {
        Error::Operation(ErrorInfo {
          message: format!("Failed to wait for navigation: {}", e),
          code: None,
        })
      })
      .map(|_| (url, tab))
  };

  let create_response = |url: Url, tab: Arc<Tab>| {
    let tab_id = Uuid::new_v4().to_string();
    TABS.lock().unwrap().insert(tab_id.clone(), tab);

    HttpResponse::Ok().json(LoadResponse {
      tab_id,
      url: url.to_string(),
    })
  };

  parse_url(&req.url)
    .and_then(|url| open_new_tab(url, browser))
    .and_then(|(url, tab)| navigate_to_url(tab, url))
    .and_then(|(url, tab)| wait_for_navigation(tab, url))
    .map(|(url, tab)| create_response(url, tab))
    .unwrap_or_else(|e| {
      HttpResponse::BadRequest().json(Error::Operation(ErrorInfo {
        message: e.to_string(),
        code: None,
      }))
    })
}

//   async fn close(req: web::Json<CloseRequest>) -> HttpResponse {
//     let tab_id = &req.tab_id;
//     match try_find_tab(tab_id) {
//       Err(e) => {
//         return HttpResponse::BadRequest().json(Error::Operation(ErrorInfo {
//           message: e.to_string(),
//           code: None,
//         }));
//       }
//       Ok(tab) => match tab.close(true) {
//         Err(e) => {
//           return HttpResponse::InternalServerError().json(Error::Operation(ErrorInfo {
//             message: format!("Failed to close tab: {}", e),
//             code: None,
//           }));
//         }
//         Ok(res) => match res {
//           true => {
//             TABS.lock().unwrap().remove(tab_id);
//             return HttpResponse::Ok().finish();
//           }
//           false => {
//             return HttpResponse::InternalServerError().json(Error::Operation(ErrorInfo {
//               message: "Failed to close tab".to_string(),
//               code: None,
//             }));
//           }
//         },
//       },
//     }
//   }
// }
