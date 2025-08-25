#![warn(clippy::all, clippy::pedantic)]

mod browser {
  pub mod api;
  pub mod page {
    pub mod api;
    pub mod models;
  }
}

mod web_api {
  pub mod models;
  pub mod routes;
  pub mod server;
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
  web_api::server::run().await
}
