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
  let user_data_dir = env::var("USER_DATA_DIR").expect("USER_DATA_DIR must be set");

  tracing_subscriber::registry()
    .with(tracing_subscriber::EnvFilter::new(log_level))
    .with(tracing_subscriber::fmt::layer())
    .init();

  browser::api::launch(&user_data_dir)
    .map(web_api::server::run)?
    .await
}
