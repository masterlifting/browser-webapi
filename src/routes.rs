use std::sync::Arc;

use actix_web::{HttpResponse, web};
use headless_chrome::Browser;

use crate::browser;

pub fn configure(cfg: &mut web::ServiceConfig, browser: Arc<Browser>) {
  cfg.service(
    web::scope("/api/v1")
      .route(
        "/health",
        web::get().to(|| async {
          HttpResponse::Ok().json(serde_json::json!({
              "status": "ok",
              "version": std::env::var("VERSION")
                  .unwrap_or_else(|_| "unknown".to_string()),
          }))
        }),
      )
      .service(
        web::scope("/browser").service(
          web::scope("/page")
            .route(
              "/load",
              web::post().to({
                move |req| {
                  let browser = Arc::clone(&browser);
                  async move { browser::page::handlers::load(req, browser).await }
                }
              }),
            )
            .route(
              "/close",
              web::post().to(|| async { HttpResponse::Ok().finish() }),
            )
            .route(
              "/text/find",
              web::post().to(|| async { HttpResponse::Ok().finish() }),
            )
            .route(
              "/input/fill",
              web::post().to(|| async { HttpResponse::Ok().finish() }),
            )
            .route(
              "/mouse/click",
              web::post().to(|| async { HttpResponse::Ok().finish() }),
            )
            .route(
              "/mouse/shuffle",
              web::post().to(|| async { HttpResponse::Ok().finish() }),
            )
            .route(
              "/form/submit",
              web::post().to(|| async { HttpResponse::Ok().finish() }),
            ),
        ),
      ),
  );
}
