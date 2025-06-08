#[actix_web::main]
async fn main() -> std::io::Result<()> {
    browser_api::run_server().await
}
