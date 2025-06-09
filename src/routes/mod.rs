use actix_web::{HttpResponse, web};

pub mod browser;

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
            .service(browser::configure()),
    );
}
