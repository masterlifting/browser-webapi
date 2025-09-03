use headless_chrome::Element;
use std::sync::Arc;

use crate::browser::element::dto::{ClickDto, ExecuteDto, ExistsDto, ExtractDto};
use crate::browser::tab;
use crate::models::{Error, ErrorInfo};

/// Finds an element in the tab using the given selector.
///
/// # Errors
///
/// Returns an `Error` if the element with the selector is not found or waiting for it fails.
pub fn find<'a>(
  tab: &'a Arc<headless_chrome::Tab>,
  selector: &'a str,
) -> Result<Element<'a>, Error> {
  tab.wait_for_element(selector).map_err(|e| {
    Error::Operation(ErrorInfo {
      message: format!("Failed to find element with selector '{selector}': {e}"),
      code: None,
    })
  })
}

#[must_use]
pub fn try_find<'a>(tab: &'a Arc<headless_chrome::Tab>, selector: &'a str) -> Option<Element<'a>> {
  tab.wait_for_element(selector).ok()
}

/// Fills the element with the given value.
///
/// # Errors
///
/// Returns a `String` describing the error if filling the element fails.
pub fn fill(element: &Element, value: &str) -> Result<(), String> {
  element
    .type_into(value)
    .map(|_| ())
    .map_err(|e| format!("Failed to fill input element '{}': {}", &element.value, e))
}

/// Clicks the element with the given selector in the tab.
///
/// # Errors
///
/// Returns an `Error` if:
/// * The tab is not found
/// * The element is not found
/// * Clicking the element fails
pub fn click(tab_id: &str, dto: ClickDto) -> Result<(), Error> {
  tab::api::find(tab_id).and_then(|tab| {
    find(&tab, &dto.selector).and_then(|element| {
      element.click().map(|_| ()).map_err(|e| {
        Error::Operation(ErrorInfo {
          message: format!("Failed to click element '{}': {}", dto.selector, e),
          code: None,
        })
      })
    })
  })
}

#[must_use]
pub fn exists(tab_id: &str, dto: ExistsDto) -> bool {
  tab::api::try_find(tab_id)
    .and_then(|tab| try_find(&tab, &dto.selector).map(|_| ()))
    .is_some()
}

/// Extracts content or attribute from the element with the given selector in the tab.
///
/// # Errors
///
/// Returns an `Error` if:
/// * The tab is not found
/// * The element is not found
/// * Getting the attribute or content fails
/// * The attribute is not present (when specified)
pub fn extract(tab_id: &str, dto: ExtractDto) -> Result<String, Error> {
  tab::api::find(tab_id).and_then(|tab| {
    find(&tab, &dto.selector).and_then(|element| match dto.attribute {
      Some(ref attr_name) => element
        .get_attribute_value(attr_name)
        .map_err(|e| {
          Error::Operation(ErrorInfo {
            message: format!(
              "Failed to get attribute '{}' of element '{}': {}",
              attr_name, dto.selector, e
            ),
            code: None,
          })
        })
        .and_then(|opt| match opt {
          Some(value) => Ok(value),
          None => Err(Error::NotFound(format!(
            "Attribute '{}' not found on element '{}'",
            attr_name, dto.selector
          ))),
        }),
      None => element.get_inner_text().map_err(|e| {
        Error::Operation(ErrorInfo {
          message: format!("Failed to get content of element '{}': {}", dto.selector, e),
          code: None,
        })
      }),
    })
  })
}

/// Executes a JavaScript function on the element with the given selector in the tab.
///
/// # Errors
///
/// Returns an `Error` if:
/// * The tab is not found
/// * The element is not found
/// * Evaluating the JavaScript fails
pub fn execute(tab_id: &str, dto: ExecuteDto) -> Result<(), Error> {
  tab::api::find(tab_id).and_then(|tab| {
    find(&tab, &dto.selector).and_then(|element| {
      element
        .call_js_fn(
          &format!("function() {{ this.{} }}", dto.function),
          vec![],
          true,
        )
        .map(|_| ())
        .map_err(|e| {
          Error::Operation(ErrorInfo {
            message: format!("Failed to evaluate JS on element '{}': {}", dto.selector, e),
            code: None,
          })
        })
    })
  })
}
