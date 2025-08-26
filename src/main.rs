#![warn(clippy::all, clippy::pedantic)]
use std::env;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

mod browser {
  pub mod api;
  pub mod page {
    pub mod api;
    pub mod models;
  }
}

mod web_api {
  pub mod models;
  pub mod routes;
  pub mod server;
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
  dotenv::dotenv().ok();

  let log_level = env::var("LOG_LEVEL").unwrap_or_else(|_| "info".to_string());

  tracing_subscriber::registry()
    .with(tracing_subscriber::EnvFilter::new(log_level))
    .with(tracing_subscriber::fmt::layer())
    .init();

  browser::api::launch().map(web_api::server::run)?.await
}
