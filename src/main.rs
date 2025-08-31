#![warn(clippy::all, clippy::pedantic)]

pub mod browser;
pub mod models;
pub mod web_api;

use std::env;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

#[actix_web::main]
async fn main() -> std::io::Result<()> {
  dotenv::dotenv().ok();

  let log_level = env::var("LOG_LEVEL").unwrap_or_else(|_| "info".to_string());

  tracing_subscriber::registry()
    .with(tracing_subscriber::EnvFilter::new(log_level))
    .with(tracing_subscriber::fmt::layer())
    .init();

  let user_data_dir = env::var("USER_DATA_DIR").expect("USER_DATA_DIR");
  let use_ui = env::var("USE_UI")
    .unwrap_or_else(|_| "false".to_string())
    .parse::<bool>()
    .unwrap_or(false);
  let idle_timeout_days = env::var("IDLE_TIMEOUT_DAYS")
    .unwrap_or_else(|_| "1".to_string())
    .parse::<u64>()
    .unwrap_or(1);

  let options = browser::models::LaunchOptions {
    headless: !use_ui,
    user_data_dir,
    idle_timeout: std::time::Duration::from_secs(idle_timeout_days * 60 * 60 * 24),
  };

  browser::api::launch(options)
    .map(web_api::server::run)?
    .await
}
