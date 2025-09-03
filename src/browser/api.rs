use headless_chrome::{Browser, LaunchOptionsBuilder};
use std::ffi::OsStr;
use std::io::{Error, ErrorKind};
use std::sync::Arc;

use crate::browser::models::LaunchOptions;

pub fn launch(options: LaunchOptions) -> Result<Arc<Browser>, Error> {
  LaunchOptionsBuilder::default()
    .headless(options.headless)
    .path(Some(std::path::PathBuf::from(options.user_data_dir)))
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
        "--single-process",
        "--disable-gpu",
        "--hide-scrollbars",
        "--mute-audio",
        "--disable-infobars",
        "--disable-breakpad",
        "--disable-web-security",
        "--disable-extensions",
        "--no-default-browser-check",
        "--user-agent=Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/121.0.0.0 Safari/537.36",
      ]
      .iter()
      .map(OsStr::new)
      .collect::<Vec<_>>())
    .build()
    .map_err(|e| Error::new(ErrorKind::Other, e.to_string()))
    .and_then(|options| {
      Browser::new(options)
        .map(Arc::new)
        .map(|browser| {
          tracing::info!("Browser launched successfully");
          browser
        })
        .map_err(|e| Error::new(ErrorKind::Other, e.to_string()))
    })
}
