use actix_web::{HttpResponse, web};

pub fn configure_routes(cfg: &mut web::ServiceConfig) {
    cfg.service(web::scope("/api/v1").route(
        "/health",
        web::get().to(|| async { HttpResponse::Ok().finish() }),
    ));
}
