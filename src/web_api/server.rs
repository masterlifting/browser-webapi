use actix_web::web;
use headless_chrome::Browser;
use std::{env, sync::Arc};
use tracing_actix_web::TracingLogger;

/// Starts an HTTP server with the provided browser instance.
///
/// This function configures and starts an Actix web server with CORS support,
/// request logging via `TracingLogger`, and routes defined in the application.
/// The server host and port can be configured via environment variables
/// `SERVER_HOST` and `SERVER_PORT`, with defaults of "127.0.0.1" and "8080"
/// respectively.
///
/// # Arguments
///
/// * `browser` - A thread-safe reference to a headless Chrome browser instance
///   that will be shared with request handlers.
///
/// # Returns
///
/// A `std::io::Result<()>` that resolves when the server has completed running.
///
/// # Errors
///
/// This function will return an error if:
/// * The server fails to bind to the specified host:port combination
/// * The underlying Actix server encounters an error during operation
pub async fn run(browser: Arc<Browser>) -> std::io::Result<()> {
  let host = env::var("SERVER_HOST").unwrap_or_else(|_| "127.0.0.1".to_string());
  let port = env::var("SERVER_PORT").unwrap_or_else(|_| "8080".to_string());
  tracing::info!("Starting server at http://{}:{}", host, port);

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
      .app_data(web::Data::new(browser.clone()))
      .configure(crate::web_api::routes::configure)
  })
  .bind(format!("{host}:{port}"))?
  .run()
  .await
}
