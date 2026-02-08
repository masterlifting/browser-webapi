use std::sync::Arc;

use actix_web::{HttpResponse, web};
use chaser_oxide::Browser;
use serde_json::json;

use crate::browser::element;
use crate::browser::element::dto::{ClickDto, ExecuteDto, ExistsDto, ExtractDto};
use crate::browser::tab;
use crate::browser::tab::dto::{FillDto, OpenDto};
use crate::web_api::response;

pub fn configure(cfg: &mut web::ServiceConfig) {
  cfg
    .route(
      "/health",
      web::get().to(|| async {
        HttpResponse::Ok().json(json!({
            "status": "ok",
            "version": env!("CARGO_PKG_VERSION"),
        }))
      }),
    )
    .service(
      web::scope("/api/v1")
        .service(web::scope("/tab").route(
          "/open",
          web::post().to(
            |req: web::Json<OpenDto>, browser: web::Data<Arc<Browser>>| async move {
              response::from_string(
                tab::api::open(browser.get_ref().clone(), req.into_inner()).await,
              )
            },
          ),
        ))
        .service(
          web::scope("/tabs/{id}")
            .route(
              "/close",
              web::delete().to(|id: web::Path<String>| async move {
                response::from_unit(tab::api::close(&id).await)
              }),
            )
            .route(
              "/fill",
              web::post().to(
                |req: web::Json<FillDto>, id: web::Path<String>| async move {
                  response::from_unit(tab::api::fill(&id, req.into_inner()).await)
                },
              ),
            )
            .route(
              "/humanize",
              web::post().to(|id: web::Path<String>| async move {
                response::from_unit(tab::api::humanize(&id).await)
              }),
            )
            .route(
              "/screenshot",
              web::get().to(|id: web::Path<String>| async move {
                response::from_image(tab::api::screenshot(&id).await)
              }),
            )
            .service(
              web::scope("/element")
                .route(
                  "/click",
                  web::post().to(
                    |req: web::Json<ClickDto>, id: web::Path<String>| async move {
                      response::from_string(element::api::click(&id, req.into_inner()).await)
                    },
                  ),
                )
                .route(
                  "/exists",
                  web::post().to(
                    |req: web::Json<ExistsDto>, id: web::Path<String>| async move {
                      HttpResponse::Ok().body(
                        element::api::exists(&id, req.into_inner())
                          .await
                          .to_string(),
                      )
                    },
                  ),
                )
                .route(
                  "/extract",
                  web::post().to(
                    |req: web::Json<ExtractDto>, id: web::Path<String>| async move {
                      response::from_string(element::api::extract(&id, req.into_inner()).await)
                    },
                  ),
                )
                .route(
                  "/execute",
                  web::post().to(
                    |req: web::Json<ExecuteDto>, id: web::Path<String>| async move {
                      response::from_string(element::api::execute(&id, req.into_inner()).await)
                    },
                  ),
                ),
            ),
        ),
    )
    .default_service(web::to(HttpResponse::NotFound));
}
