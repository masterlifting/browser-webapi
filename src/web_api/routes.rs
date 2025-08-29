use std::sync::Arc;

use actix_web::{HttpResponse, web};
use headless_chrome::Browser;
use serde_json::json;

use crate::browser::element;
use crate::browser::element::dto::{GetElement, PostElement};
use crate::browser::tab;
use crate::browser::tab::dto::{FillRequest, OpenRequest};
use crate::models::Error;

fn map_error_to_response(e: Error) -> HttpResponse {
  match e {
    Error::NotFound(msg) => HttpResponse::NotFound().body(msg),
    error => HttpResponse::BadRequest().body(error.to_string()),
  }
}

fn map_string_to_response(res: Result<String, Error>) -> HttpResponse {
  res
    .map(|s| HttpResponse::Ok().body(s))
    .unwrap_or_else(map_error_to_response)
}

fn map_unit_to_response(res: Result<(), Error>) -> HttpResponse {
  res
    .map(|_| HttpResponse::Ok().finish())
    .unwrap_or_else(map_error_to_response)
}

pub fn configure(cfg: &mut web::ServiceConfig) {
  cfg
    .service(
      web::scope("/api/v1")
        .route(
          "/health",
          web::get().to(|| async {
            HttpResponse::Ok().json(json!({
                "status": "ok",
                "version": std::env::var("VERSION")
                    .unwrap_or_else(|_| "unknown".to_string()),
            }))
          }),
        )
        .service(web::scope("/tab").route(
          "/open",
          web::post().to(
            |req: web::Json<OpenRequest>, browser: web::Data<Arc<Browser>>| async move {
              map_string_to_response(
                tab::api::open(browser.get_ref().clone(), req.into_inner()).await,
              )
            },
          ),
        ))
        .service(
          web::scope("/tabs/{id}")
            .route(
              "/close",
              web::get().to(|id: web::Path<String>| async move {
                map_unit_to_response(tab::api::close(&id).await)
              }),
            )
            .route(
              "/fill",
              web::post().to(
                |req: web::Json<FillRequest>, id: web::Path<String>| async move {
                  map_unit_to_response(tab::api::fill(&id, req.into_inner()).await)
                },
              ),
            )
            .service(
              web::scope("/element")
                .route(
                  "/click",
                  web::post().to(
                    |req: web::Json<GetElement>, id: web::Path<String>| async move {
                      map_unit_to_response(element::api::click(&id, req.into_inner()).await)
                    },
                  ),
                )
                .route(
                  "/exists",
                  web::get().to(
                    |req: web::Json<GetElement>, id: web::Path<String>| async move {
                      HttpResponse::Ok().body(
                        element::api::exists(&id, req.into_inner())
                          .await
                          .to_string(),
                      )
                    },
                  ),
                )
                .route(
                  "/content",
                  web::get().to(
                    |req: web::Json<GetElement>, id: web::Path<String>| async move {
                      map_string_to_response(element::api::content(&id, req.into_inner()).await)
                    },
                  ),
                )
                .route(
                  "/evaluate",
                  web::post().to(
                    |req: web::Json<PostElement>, id: web::Path<String>| async move {
                      map_unit_to_response(element::api::evaluate(&id, req.into_inner()).await)
                    },
                  ),
                ),
            ),
        ),
    )
    .default_service(web::to(|| async { HttpResponse::NotFound() }));
}
