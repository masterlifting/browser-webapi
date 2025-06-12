use actix_cors::Cors;
use actix_web::{App, HttpServer, middleware, web};
use dotenv;
use std::env;
use tracing_actix_web::TracingLogger;

mod routes;

#[actix_web::main]
async fn main() -> std::io::Result<()> {
  // Load environment variables from .env file
  dotenv().ok();

  // Initialize tracing
  tracing_subscriber::fmt()
    .with_max_level(tracing::Level::INFO)
    .init();

  // Get configuration from environment or use defaults
  let host = env::var("HOST").unwrap_or_else(|_| "127.0.0.1".to_string());
  let port = env::var("PORT")
    .unwrap_or_else(|_| "8080".to_string())
    .parse::<u16>()
    .expect("PORT must be a number");

  tracing::info!("Starting server at http://{}:{}", host, port);

  // Start HTTP server
  HttpServer::new(|| {
    // Configure CORS
    let cors = Cors::default()
      .allow_any_origin()
      .allow_any_method()
      .allow_any_header()
      .max_age(3600);

    App::new()
      .wrap(TracingLogger::default())
      .wrap(middleware::Compress::default())
      .wrap(cors)
      .configure(routes::configure)
      .default_service(web::to(|| async {
        actix_web::HttpResponse::NotFound().json(serde_json::json!({
            "error": "Not Found",
            "message": "The requested resource does not exist"
        }))
      }))
  })
  .bind((host, port))?
  .run()
  .await
}
