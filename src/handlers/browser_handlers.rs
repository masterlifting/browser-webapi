use actix_web::{web, HttpResponse, Responder};
use crate::{
    browser::BrowserClient,
    config::Config,
    error::AppError,
    models::{
        ApiResponse, ScreenshotRequest, NavigateResponse, PdfRequest, ExecuteJsRequest,
        ExecuteJsResponse, FormSubmitRequest, FormSubmitResponse,
    },
};

pub async fn health_check() -> impl Responder {
    HttpResponse::Ok().json(ApiResponse::new(serde_json::json!({
        "status": "ok",
        "version": env!("CARGO_PKG_VERSION"),
    })))
}

pub async fn take_screenshot(
    config: web::Data<Config>,
    req: web::Json<ScreenshotRequest>,
) -> Result<HttpResponse, AppError> {
    use crate::browser::domain::Selector;
    use crate::browser::page;
    
    // Get singleton browser instance
    let browser = BrowserClient::new(config.chrome.clone())?;
    
    // Load page in functional style
    let tab = page::load(&req.url, &browser.browser)
        .await
        .map_err(|e| AppError::BrowserError(format!("{}", e)))?;
    
    // Wait for specific selector if requested
    if let Some(selector_str) = &req.wait_for_selector {
        let selector = Selector::new(selector_str);
        let _ = page::retry(1000, 3, || async {
            page::try_find_locator(&selector, &tab).await
        })
        .await
        .map_err(|e| AppError::BrowserError(format!("Failed waiting for selector: {}", e)))?;
    }
    
    // Take screenshot (either of element or full page)
    let full_page = req.full_page.unwrap_or(false);
    let screenshot_data = browser.take_screenshot(&tab, full_page)?;
    
    // Return screenshot image
    Ok(HttpResponse::Ok()
        .content_type("image/png")
        .body(screenshot_data))
}

pub async fn generate_pdf(
    config: web::Data<Config>,
    req: web::Json<PdfRequest>,
) -> Result<HttpResponse, AppError> {
    use crate::browser::domain::Selector;
    use crate::browser::page;
    
    let browser = BrowserClient::new(config.chrome.clone())?;
    
    // Navigate to page in functional style
    let tab = page::load(&req.url, &browser.browser)
        .await
        .map_err(|e| AppError::BrowserError(format!("{}", e)))?;
    
    // Wait for specific selector if requested
    if let Some(selector_str) = &req.wait_for_selector {
        let selector = Selector::new(selector_str);
        let _ = page::retry(1000, 3, || async {
            page::try_find_locator(&selector, &tab).await
        })
        .await
        .map_err(|e| AppError::BrowserError(format!("Failed waiting for selector: {}", e)))?;
    }
    
    // Generate PDF
    let pdf_data = browser.generate_pdf(&tab)?;
    
    // Return PDF document
    Ok(HttpResponse::Ok()
        .content_type("application/pdf")
        .body(pdf_data))
}

pub async fn get_page_content(
    config: web::Data<Config>,
    url: web::Json<ScreenshotRequest>,
) -> Result<HttpResponse, AppError> {
    use crate::browser::domain::Selector;
    use crate::browser::page;
    use crate::browser::page::text;
    
    let browser = BrowserClient::new(config.chrome.clone())?;
    
    // Navigate to page in functional style
    let tab = page::load(&url.url, &browser.browser)
        .await
        .map_err(|e| AppError::BrowserError(format!("{}", e)))?;
    
    // Wait for specific selector if requested
    if let Some(selector_str) = &url.wait_for_selector {
        let selector = Selector::new(selector_str);
        let _ = page::retry(1000, 3, || async {
            page::try_find_locator(&selector, &tab).await
        })
        .await
        .map_err(|e| AppError::BrowserError(format!("Failed waiting for selector: {}", e)))?;
    }
    
    // Get page title and HTML content using JavaScript execution
    let title: String = browser.execute_javascript(&tab, "document.title")?;
    let current_url: String = browser.execute_javascript(&tab, "window.location.href")?;
    let html = browser.get_page_html(&tab)?;
    
    let response = NavigateResponse {
        title,
        url: current_url,
        html: Some(html),
    };
    
    Ok(HttpResponse::Ok().json(ApiResponse::new(response)))
}

pub async fn execute_js(
    config: web::Data<Config>,
    req: web::Json<ExecuteJsRequest>,
) -> Result<HttpResponse, AppError> {
    use crate::browser::domain::Selector;
    use crate::browser::page;
    
    let browser = BrowserClient::new(config.chrome.clone())?;
    
    // Navigate to page in functional style
    let tab = page::load(&req.url, &browser.browser)
        .await
        .map_err(|e| AppError::BrowserError(format!("{}", e)))?;
    
    // Wait for specific selector if requested
    if let Some(selector_str) = &req.wait_for_selector {
        let selector = Selector::new(selector_str);
        let _ = page::retry(1000, 3, || async {
            page::try_find_locator(&selector, &tab).await
        })
        .await
        .map_err(|e| AppError::BrowserError(format!("Failed waiting for selector: {}", e)))?;
    }
    
    // Execute JavaScript
    let result: serde_json::Value = browser.execute_javascript(&tab, &req.script)?;
    
    let response = ExecuteJsResponse { result };
    
    Ok(HttpResponse::Ok().json(ApiResponse::new(response)))
}

pub async fn submit_form(
    config: web::Data<Config>,
    req: web::Json<FormSubmitRequest>,
) -> Result<HttpResponse, AppError> {
    use crate::browser::domain::{Selector, WaitFor};
    use crate::browser::page;
    use crate::browser::page::{input, mouse, form};
    
    let browser = BrowserClient::new(config.chrome.clone())?;
    
    // Navigate to page in functional style
    let tab = page::load(&req.url, &browser.browser)
        .await
        .map_err(|e| AppError::BrowserError(format!("{}", e)))?;
    
    // Fill form fields using our functional input module
    for field in &req.form_fields {
        let selector = Selector::new(&field.selector);
        input::fill(&selector, &field.value, &tab)
            .await
            .map_err(|e| AppError::BrowserError(format!("{}", e)))?;
    }
    
    // Submit form by clicking submit button
    let submit_selector = Selector::new(&req.submit_selector);
    let wait_option = if req.wait_for_navigation.unwrap_or(true) {
        WaitFor::Url(".*".to_string())  // Match any URL after navigation
    } else {
        WaitFor::Nothing
    };
      mouse::click(&submit_selector, wait_option, &tab)
        .await
        .map_err(|e| AppError::BrowserError(format!("{}", e)))?;
    
    // Get current URL
    let current_url: String = browser.execute_javascript(&tab, "window.location.href")?;
    
    let response = FormSubmitResponse {
        success: true,
        current_url,
    };
    
    Ok(HttpResponse::Ok().json(ApiResponse::new(response)))
}
