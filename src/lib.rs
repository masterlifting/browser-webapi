pub mod browser;
pub mod models;
pub mod routes;

use actix_web::web;
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

  tracing::info!("Starting server at http://{}:{}", host, port);

  // Launch a single browser instance for the entire application
  let browser = match browser::launch() {
    Ok(browser) => {
      tracing::info!("Browser launched successfully");
      browser
    }
    Err(e) => {
      tracing::error!("Failed to launch browser: {}", e);
      return Err(std::io::Error::new(
        std::io::ErrorKind::Other,
        e.to_string(),
      ));
    }
  };

  actix_web::HttpServer::new(move || {
    // Each worker gets its own clone of the Arc<Browser>
    let browser = browser.clone();

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
      .app_data(web::Data::new(browser.clone())) // Store browser in app state
      .configure(routes::configure)
  })
  .bind(format!("{}:{}", host, port))?
  .run()
  .await
}
