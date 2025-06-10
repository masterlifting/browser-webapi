use headless_chrome::{Browser, LaunchOptions};
use std::sync::Arc;

fn launch() -> Result<Arc<Browser>, String> {
    Browser::new(LaunchOptions::default())
        .map_err(|e| format!("Failed to launch browser: {}", e))
        .map(Arc::new)
}
