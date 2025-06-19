pub mod page;

use headless_chrome::{Browser, LaunchOptions};
use std::sync::Arc;

use crate::models::{Error, ErrorInfo};

pub fn launch() -> Result<Arc<Browser>, Error> {
  use std::ffi::OsStr;
  use std::time::Duration;

  let mut options = LaunchOptions::default();

  options.headless = true;
  options.idle_browser_timeout = Duration::from_secs(0); // Disable idle timeout completely
  options.window_size = Some((1920, 1080));
  options.ignore_certificate_errors = true;
  options.sandbox = false; // May be required in some environments

  // Chrome flags for long-running stability as OsStr
  let args = vec![
    "--disable-backgrounding-occluded-windows",
    "--disable-renderer-backgrounding",
    "--disable-features=TranslateUI",
    "--disable-component-extensions-with-background-pages",
    "--disable-background-timer-throttling",
    "--disable-hang-monitor",
    "--no-default-browser-check",
    "--disable-popup-blocking",
    "--disable-features=IsolateOrigins,site-per-process",
    "--disable-gpu",
    "--no-sandbox",
  ];

  // Convert each argument to OsStr and add to options.args
  options.args = args.iter().map(OsStr::new).collect();
  // Log options for debugging
  tracing::info!(
    "Launching browser with idle_timeout: {:?}",
    options.idle_browser_timeout
  );
  tracing::info!("Chrome args: {:?}", args);

  Browser::new(options).map(Arc::new).map_err(|e| {
    Error::Operation(ErrorInfo {
      message: format!("Failed to launch browser: {}", e),
      code: None,
    })
  })
}
