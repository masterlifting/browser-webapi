use chaser_oxide::{Browser, BrowserConfig, Handler};
use std::path::PathBuf;
use std::sync::Arc;

use crate::browser::models::LaunchOptions;
use crate::models::{Error, ErrorInfo};

/// Launches a new Chrome browser instance with the provided options.
///
/// # Behavior
///
/// - Uses `options.user_data_dir` as the Chrome user data directory.
/// - Disables the Chrome sandbox (`--no-sandbox`).
/// - Sets the window size to 1920x1080.
///
/// # Errors
///
/// Returns an error if:
/// - Building the browser configuration fails.
/// - Creating the browser instance fails.
pub async fn launch(options: LaunchOptions) -> Result<(Arc<Browser>, Handler), Error> {
  let config = BrowserConfig::builder()
    .user_data_dir(PathBuf::from(&options.user_data_dir))
    .no_sandbox()
    .build()
    .map_err(|e| {
      Error::Operation(ErrorInfo {
        message: format!("Failed to build browser config: {e}"),
        code: None,
      })
    })?;

  Browser::launch(config)
    .await
    .map(|(browser, handler)| (Arc::new(browser), handler))
    .map_err(|e| {
      Error::Operation(ErrorInfo {
        message: format!("Failed to launch browser: {e}"),
        code: None,
      })
    })
}
