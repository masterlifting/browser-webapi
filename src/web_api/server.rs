use actix_web::web;
use headless_chrome::Browser;
use std::{env, sync::Arc};
use tracing_actix_web::TracingLogger;

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
  .bind(format!("{}:{}", host, port))?
  .run()
  .await
}
