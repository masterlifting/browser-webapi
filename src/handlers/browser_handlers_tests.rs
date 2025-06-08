#[cfg(test)]
mod tests {
    use super::*;
    use actix_web::{http::StatusCode, test, App, web};
    use crate::config::Config;
    use crate::models::{ScreenshotRequest, PdfRequest, ExecuteJsRequest};
    use std::time::Duration;

    // Helper function to create a test config
    fn get_test_config() -> Config {
        use crate::config::{ServerConfig, ChromeConfig, CorsConfig};
        
        Config {
            server: ServerConfig {
                host: "127.0.0.1".to_string(),
                port: 8080,
            },
            chrome: ChromeConfig {
                path: None,
                page_load_timeout: Duration::from_millis(30000),
                navigation_timeout: Duration::from_millis(30000),
            },
            cors: CorsConfig {
                allowed_origins: "*".to_string(),
            },
            log_level: "debug".to_string(),
            api_rate_limit: 100,
        }
    }

    #[actix_rt::test]
    async fn test_health_check() {
        let app = test::init_service(
            App::new()
                .route("/health", web::get().to(health_check))
        ).await;
        
        let req = test::TestRequest::get()
            .uri("/health")
            .to_request();
            
        let resp = test::call_service(&app, req).await;
        assert_eq!(resp.status(), StatusCode::OK);
        
        let body = test::read_body(resp).await;
        let response: serde_json::Value = serde_json::from_slice(&body).unwrap();
        
        assert_eq!(response["status"], "success");
        assert_eq!(response["data"]["status"], "ok");
    }

    #[actix_rt::test]
    #[ignore]  // Skip in CI environments
    async fn test_screenshot_endpoint() {
        let config = get_test_config();
        let app = test::init_service(
            App::new()
                .app_data(web::Data::new(config))
                .route("/screenshot", web::post().to(take_screenshot))
        ).await;
        
        let payload = ScreenshotRequest {
            url: "https://www.example.com".to_string(),
            full_page: Some(false),
            wait_for_selector: None,
            selector: None,
            format: None,
            quality: None,
        };
        
        let req = test::TestRequest::post()
            .uri("/screenshot")
            .set_json(&payload)
            .to_request();
            
        let resp = test::call_service(&app, req).await;
        
        // Should be OK or redirect depending on example.com behavior
        assert!(resp.status().is_success() || resp.status().is_redirection());
    }

    #[actix_rt::test]
    #[ignore]  // Skip in CI environments
    async fn test_pdf_endpoint() {
        let config = get_test_config();
        let app = test::init_service(
            App::new()
                .app_data(web::Data::new(config))
                .route("/pdf", web::post().to(generate_pdf))
        ).await;
        
        let payload = PdfRequest {
            url: "https://www.example.com".to_string(),
            wait_for_selector: None,
            print_background: Some(true),
            landscape: Some(false),
            scale: Some(1.0),
            paper_width: Some(8.5),
            paper_height: Some(11.0),
            margin_top: Some(0.5),
            margin_bottom: Some(0.5),
            margin_left: Some(0.5),
            margin_right: Some(0.5),
        };
        
        let req = test::TestRequest::post()
            .uri("/pdf")
            .set_json(&payload)
            .to_request();
            
        let resp = test::call_service(&app, req).await;
        
        // Should be OK or redirect depending on example.com behavior
        assert!(resp.status().is_success() || resp.status().is_redirection());
    }

    #[actix_rt::test]
    #[ignore]  // Skip in CI environments
    async fn test_execute_js_endpoint() {
        let config = get_test_config();
        let app = test::init_service(
            App::new()
                .app_data(web::Data::new(config))
                .route("/execute", web::post().to(execute_js))
        ).await;
        
        let payload = ExecuteJsRequest {
            url: "https://www.example.com".to_string(),
            script: "return document.title".to_string(),
            wait_for_selector: None,
        };
        
        let req = test::TestRequest::post()
            .uri("/execute")
            .set_json(&payload)
            .to_request();
            
        let resp = test::call_service(&app, req).await;
        
        // Should be OK or redirect depending on example.com behavior
        assert!(resp.status().is_success() || resp.status().is_redirection());
    }
}
