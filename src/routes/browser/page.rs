use actix_web::{HttpResponse, web};

pub fn configure() -> actix_web::Scope {
    web::scope("/page").route("/load", web::get().to(load))
}

async fn load() -> HttpResponse {
    HttpResponse::Ok().json(LoadResponse {
        message: "Page loaded successfully".to_string(),
    })
}

#[derive(serde::Serialize)]
struct LoadResponse {
    message: String,
}
