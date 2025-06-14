use headless_chrome::{Browser, LaunchOptions};
use std::sync::Arc;

use crate::models::{Error, ErrorInfo};

pub fn launch() -> Result<Arc<Browser>, Error> {
  Browser::new(LaunchOptions::default())
    .map(Arc::new)
    .map_err(|e| {
      Error::Operation(ErrorInfo {
        message: format!("Failed to launch browser: {}", e),
        code: None,
      })
    })
}
