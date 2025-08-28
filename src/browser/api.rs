use headless_chrome::{Browser, LaunchOptions};
use std::ffi::OsStr;
use std::io::{Error, ErrorKind};
use std::{sync::Arc, time};

pub fn launch() -> Result<Arc<Browser>, Error> {
  let one_week = time::Duration::from_secs(60 * 60 * 24 * 7);

  let mut options = LaunchOptions {
    headless: true,
    idle_browser_timeout: one_week,
    ignore_certificate_errors: false,
    window_size: Some((1280, 800)),
    sandbox: true,
    ..Default::default()
  };

  let args = vec![
    "--no-sandbox",
    "--lang=en-US,en",
    "--window-size=1366,768",
    "--disable-blink-features=AutomationControlled",
    "--user-agent=Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/122.0.0.0 Safari/537.36",
  ];

  options.args = args.iter().map(OsStr::new).collect();

  Browser::new(options)
    .map(Arc::new)
    .map(|browser| {
      tracing::info!("Browser launched successfully");
      browser
    })
    .map_err(|e| Error::new(ErrorKind::Other, e.to_string()))
}
