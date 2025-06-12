pub mod browser {
  pub mod handlers;
  pub mod models;
  pub mod page {
    pub mod handlers;
    pub mod models;
  }
}
pub mod models;
pub mod routes;

use std::env;
use tracing_actix_web::TracingLogger;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

pub async fn run_server() -> std::io::Result<()> {
  dotenv::dotenv().ok();

  let host = env::var("SERVER_HOST").unwrap_or_else(|_| "127.0.0.1".to_string());
  let port = env::var("SERVER_PORT").unwrap_or_else(|_| "8080".to_string());
  let log_level = env::var("LOG_LEVEL").unwrap_or_else(|_| "info".to_string());

  tracing_subscriber::registry()
    .with(tracing_subscriber::EnvFilter::new(log_level))
    .with(tracing_subscriber::fmt::layer())
    .init();

  actix_web::HttpServer::new(move || {
    let cors = actix_cors::Cors::default()
      .allowed_methods(vec!["GET", "POST", "PUT", "DELETE"])
      .allowed_headers(vec![
        "Content-Type",
        "Authorization",
        "Accept",
        "X-Requested-With",
      ])
      .max_age(3600);

    actix_web::App::new()
      .wrap(TracingLogger::default())
      .wrap(cors)
      .configure(routes::configure)
  })
  .bind(format!("{}:{}", host, port))?
  .run()
  .await
}
