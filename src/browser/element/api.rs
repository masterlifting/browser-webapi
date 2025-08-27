use headless_chrome::Element;
use std::sync::Arc;

use crate::browser::element::dto::GetElement;
use crate::browser::tab;
use crate::models::{Error, ErrorInfo};

pub fn find<'a>(
  tab: &'a Arc<headless_chrome::Tab>,
  selector: &'a str,
) -> Result<Element<'a>, Error> {
  tab.wait_for_element(selector).map_err(|e| {
    Error::Operation(ErrorInfo {
      message: format!("Failed to find element with selector '{}': {}", selector, e),
      code: None,
    })
  })
}

pub fn try_find<'a>(tab: &'a Arc<headless_chrome::Tab>, selector: &'a str) -> Option<Element<'a>> {
  tab.wait_for_element(selector).ok()
}

pub async fn click(tab_id: &str, req: GetElement) -> Result<(), Error> {
  tab::api::find(tab_id).and_then(|tab| {
    find(&tab, &req.selector).and_then(|element| {
      element.click().map(|_| ()).map_err(|e| {
        Error::Operation(ErrorInfo {
          message: format!("Failed to click element '{}': {}", req.selector, e),
          code: None,
        })
      })
    })
  })
}

pub async fn exists(tab_id: &str, req: GetElement) -> bool {
  tab::api::try_find(tab_id)
    .and_then(|tab| try_find(&tab, &req.selector).map(|_| ()))
    .is_some()
}

pub fn fill(element: &Element, value: &str) -> Result<(), String> {
  element
    .type_into(value)
    .map(|_| ())
    .map_err(|e| format!("Failed to fill input element '{}': {}", &element.value, e))
}

pub async fn content(tab_id: &str, req: GetElement) -> Result<String, Error> {
  tab::api::find(tab_id).and_then(|tab| {
    find(&tab, &req.selector).and_then(|element| {
      element.get_inner_text().map_err(|e| {
        Error::Operation(ErrorInfo {
          message: format!("Failed to get content of element '{}': {}", req.selector, e),
          code: None,
        })
      })
    })
  })
}
