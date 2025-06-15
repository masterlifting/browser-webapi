use actix_cors::Cors;
use actix_web::{App, HttpServer, middleware, web};
use dotenv;
use std::env;
use tracing_actix_web::TracingLogger;

mod browser;
mod routes;

#[actix_web::main]
async fn main() -> std::io::Result<()> {
  lib::run_server().await
}
