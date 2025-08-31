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

  let options = browser::models::LaunchOptions::from_env();

  browser::api::launch(options)
    .map(web_api::server::run)?
    .await
}
