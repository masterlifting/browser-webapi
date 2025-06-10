use actix_web::{HttpResponse, web};

use crate::handlers::browser;

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
                        .route("/close", web::post().to(browser::page::close))
                        .route("/text/find", web::post().to(browser::page::find_text))
                        .route("/input/fill", web::post().to(browser::page::fill_input))
                        .route("/mouse/click", web::post().to(browser::page::mouse_click))
                        .route(
                            "/mouse/shuffle",
                            web::post().to(browser::page::mouse_shuffle),
                        )
                        .route("/form/submit", web::post().to(browser::page::form_submit)),
                ),
            ),
    );
}
