use headless_chrome::{Browser, LaunchOptionsBuilder};
use std::ffi::OsStr;
use std::io::{Error, ErrorKind};
use std::sync::Arc;
use std::time;

pub fn launch(user_data_dir: &str) -> Result<Arc<Browser>, Error> {
  let one_week = time::Duration::from_secs(60 * 60 * 24 * 7);

  LaunchOptionsBuilder::default()
    .headless(false)
    .disable_default_args(true)
    .ignore_certificate_errors(false)
    .window_size(Some((1920, 1080)))
    .idle_browser_timeout(one_week)
    .args(vec![
      OsStr::new("--no-sandbox"),
      OsStr::new("--disable-setuid-sandbox"),
      OsStr::new("--disable-dev-shm-usage"),
      OsStr::new("--disable-accelerated-2d-canvas"),
      OsStr::new("--no-first-run"),
      OsStr::new("--no-zygote"),
      OsStr::new("--disable-gpu"),
      OsStr::new("--hide-scrollbars"),
      OsStr::new("--mute-audio"),
      OsStr::new("--disable-infobars"),
      OsStr::new("--disable-breakpad"),
      OsStr::new("--disable-web-security"),
      OsStr::new("--disable-extensions"),
      OsStr::new("--no-default-browser-check"),
      OsStr::new(&format!("--user-data-dir={}", user_data_dir)),
      OsStr::new("--user-agent=Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/121.0.0.0 Safari/537.36"),
    ])
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
