use std::sync::Arc;

use actix_web::{HttpResponse, web};
use headless_chrome::Browser;

use crate::models::Error;

pub fn configure(cfg: &mut web::ServiceConfig) {
  cfg
    .service(
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
                web::post().to(|req, browser: web::Data<Arc<Browser>>| async move {
                  crate::browser::page::load(req, browser.get_ref().clone()).await
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
    )
    .default_service(web::to(|| async {
      actix_web::HttpResponse::NotFound().json(serde_json::json!(Error::NotFound(
        "Endpoint not found".to_string()
      )))
    }));
}
