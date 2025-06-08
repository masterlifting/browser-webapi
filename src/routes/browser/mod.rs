use actix_web::web;

mod page;

pub fn configure() -> actix_web::Scope {
    web::scope("/browser").service(page::configure())
}
