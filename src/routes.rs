use actix_web::{HttpResponse, web};

use crate::browser;

pub fn configure(cfg: &mut web::ServiceConfig) {
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
            .route("/load", web::post().to(browser::page::load))
            .route("/close", web::post().to(HttpResponse::Ok().finish()))
            .route("/text/find", web::post().to(HttpResponse::Ok().finish()))
            .route("/input/fill", web::post().to(HttpResponse::Ok().finish()))
            .route("/mouse/click", web::post().to(HttpResponse::Ok().finish()))
            .route(
              "/mouse/shuffle",
              web::post().to(HttpResponse::Ok().finish()),
            )
            .route("/form/submit", web::post().to(HttpResponse::Ok().finish())),
        ),
      ),
  );
}
