pub mod page;

use actix_web::web;

pub fn configure() -> actix_web::Scope {
    web::scope("/browser")
        .service(page::configure())
}
