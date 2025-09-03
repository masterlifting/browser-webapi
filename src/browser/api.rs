use headless_chrome::{Browser, LaunchOptionsBuilder};
use std::ffi::OsStr;
use std::io::Error;
use std::sync::Arc;

use crate::browser::models::LaunchOptions;

/// Launches a new headless Chrome browser instance with the given options.
///
/// # Errors
///
/// Returns an `Error` if:
/// * Building the launch options fails
/// * Creating the browser instance fails
pub fn launch(options: LaunchOptions) -> Result<Arc<Browser>, Error> {
  LaunchOptionsBuilder::default()
    .headless(options.headless)
    .path(options.binary_data_dir)
    .disable_default_args(true)
    .ignore_certificate_errors(false)
    .window_size(Some((1920, 1080)))
    .idle_browser_timeout(options.idle_timeout)
    .args(
      [
        "--no-sandbox",
        "--disable-setuid-sandbox",
        "--disable-dev-shm-usage",
        "--disable-accelerated-2d-canvas",
        "--no-first-run",
        "--no-zygote",
        "--disable-namespace-sandbox",
        "--disable-seccomp-filter-sandbox",
        "--disable-gpu",
        "--hide-scrollbars",
        "--mute-audio",
        "--disable-infobars",
        "--disable-breakpad",
        "--disable-web-security",
        "--disable-extensions",
        "--no-default-browser-check",
        &format!("--user-data-dir={}", options.user_data_dir),
        "--user-agent=Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/121.0.0.0 Safari/537.36",
      ]
      .iter()
      .map(OsStr::new)
      .collect::<Vec<_>>())
    .build()
    .map_err(|e| Error::other(e.to_string()))
    .and_then(|options| {
      Browser::new(options)
        .map(Arc::new)
        .inspect(|_| {
          tracing::info!("Browser launched successfully");
        })
        .map_err(|e| Error::other(e.to_string()))
    })
}
