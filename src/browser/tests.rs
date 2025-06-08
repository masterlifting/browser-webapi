#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::ChromeConfig;
    use std::time::Duration;
    use rstest::rstest;
    use test_log::test;

    // Mock server setup
    fn setup_mock_server() -> mockito::Server {
        mockito::Server::new()
    }

    fn get_test_config() -> ChromeConfig {
        ChromeConfig {
            path: None,
            page_load_timeout: Duration::from_millis(30000),
            navigation_timeout: Duration::from_millis(30000),
        }
    }

    #[test]
    #[ignore] // Skip in CI environments
    fn test_browser_client_initialization() {
        let config = get_test_config();
        let result = BrowserClient::new(config);
        assert!(result.is_ok());
    }

    #[rstest]
    #[test]
    #[ignore] // Skip in CI environments
    fn test_navigate_to_url() {
        let mock_server = setup_mock_server();
        let url = &mock_server.url();
        
        // Create a mock endpoint
        let _m = mock_server.mock("GET", "/")
            .with_status(200)
            .with_header("content-type", "text/html")
            .with_body("<html><head><title>Test Page</title></head><body><h1>Hello World</h1></body></html>")
            .create();
        
        let config = get_test_config();
        let browser = BrowserClient::new(config).unwrap();
        let result = browser.navigate_to_url(url);
        
        assert!(result.is_ok());
    }    #[test]
    #[ignore] // Skip in CI environments
    fn test_get_page_html() {
        let mock_server = setup_mock_server();
        let url = &mock_server.url();
        
        // Create a mock endpoint with HTML content
        let test_html = "<html><head><title>Test Page</title></head><body><h1>Hello World</h1></body></html>";
        let _m = mock_server.mock("GET", "/")
            .with_status(200)
            .with_header("content-type", "text/html")
            .with_body(test_html)
            .create();
            
        let config = get_test_config();
        let browser = BrowserClient::new(config).unwrap();
        let tab = browser.navigate_to_url(url).unwrap();
        
        // Get HTML content
        let html = browser.get_page_html(&tab).unwrap();
        
        // Check that HTML contains our test content
        assert!(html.contains("Hello World"));
    }
    
    #[test]
    #[ignore] // Skip in CI environments
    fn test_execute_javascript() {
        let mock_server = setup_mock_server();
        let url = &mock_server.url();
        
        // Create a mock endpoint
        let _m = mock_server.mock("GET", "/")
            .with_status(200)
            .with_header("content-type", "text/html")
            .with_body("<html><head><title>JS Test</title></head><body><div id='result'></div></body></html>")
            .create();
            
        let config = get_test_config();
        let browser = BrowserClient::new(config).unwrap();
        let tab = browser.navigate_to_url(url).unwrap();
        
        // Execute JS that returns a value
        let result: String = browser.execute_javascript(&tab, "return 'Hello from JS'").unwrap();
        assert_eq!(result, "Hello from JS");
        
        // Execute JS that modifies DOM
        let _: serde_json::Value = browser.execute_javascript(
            &tab, 
            "document.getElementById('result').textContent = 'Updated'; return true;"
        ).unwrap();
        
        // Check that DOM was modified
        let html = browser.get_page_html(&tab).unwrap();
        assert!(html.contains("<div id='result'>Updated</div>"));
    }
    
    #[test]
    #[ignore] // Skip in CI environments
    fn test_fill_form() {
        let mock_server = setup_mock_server();
        let url = &mock_server.url();
        
        // Create a mock endpoint with a form
        let form_html = r#"
        <html>
            <head><title>Form Test</title></head>
            <body>
                <form id="testForm">
                    <input type="text" id="name" name="name">
                    <input type="email" id="email" name="email">
                    <button type="submit" id="submit">Submit</button>
                </form>
            </body>
        </html>
        "#;
        
        let _m = mock_server.mock("GET", "/")
            .with_status(200)
            .with_header("content-type", "text/html")
            .with_body(form_html)
            .create();
            
        let config = get_test_config();
        let browser = BrowserClient::new(config).unwrap();
        let tab = browser.navigate_to_url(url).unwrap();
        
        // Fill form fields
        browser.fill_form(&tab, "#name", "John Doe").unwrap();
        browser.fill_form(&tab, "#email", "john@example.com").unwrap();
        
        // Check that fields were filled
        let name_value: String = browser.execute_javascript(&tab, "document.getElementById('name').value").unwrap();
        let email_value: String = browser.execute_javascript(&tab, "document.getElementById('email').value").unwrap();
        
        assert_eq!(name_value, "John Doe");
        assert_eq!(email_value, "john@example.com");
    }
      #[test]
    #[ignore] // Skip in CI environments
    fn test_click_element() {
        let mock_server = setup_mock_server();
        let url = &mock_server.url();
        
        // Create a mock endpoint with clickable element
        let html = r#"
        <html>
            <head><title>Click Test</title></head>
            <body>
                <button id="clickMe" onclick="document.getElementById('result').textContent = 'Clicked'">Click Me</button>
                <div id="result">Not Clicked</div>
            </body>
        </html>
        "#;
        
        let _m = mock_server.mock("GET", "/")
            .with_status(200)
            .with_header("content-type", "text/html")
            .with_body(html)
            .create();
            
        let config = get_test_config();
        let browser = BrowserClient::new(config).unwrap();
        let tab = browser.navigate_to_url(url).unwrap();
        
        // Click the button
        browser.click_element(&tab, "#clickMe").unwrap();
        
        // Wait a moment for the click to process
        std::thread::sleep(Duration::from_millis(100));
        
        // Check that the result div was updated
        let result: String = browser.execute_javascript(&tab, "document.getElementById('result').textContent").unwrap();
        assert_eq!(result, "Clicked");
    }
    
    #[test]
    #[ignore] // Skip in CI environments
    fn test_take_screenshot() {
        let mock_server = setup_mock_server();
        let url = &mock_server.url();
        
        // Create a mock endpoint
        let _m = mock_server.mock("GET", "/")
            .with_status(200)
            .with_header("content-type", "text/html")
            .with_body("<html><head><title>Screenshot Test</title></head><body><h1>Hello World</h1></body></html>")
            .create();
            
        let config = get_test_config();
        let browser = BrowserClient::new(config).unwrap();
        let tab = browser.navigate_to_url(url).unwrap();
        
        // Take screenshot
        let screenshot = browser.take_screenshot(&tab, false).unwrap();
        
        // Check that screenshot is not empty and has PNG magic bytes
        assert!(!screenshot.is_empty());
        assert_eq!(&screenshot[0..4], &[0x89, 0x50, 0x4E, 0x47]); // PNG magic bytes
    }
    
    #[test]
    #[ignore] // Skip in CI environments
    fn test_generate_pdf() {
        let mock_server = setup_mock_server();
        let url = &mock_server.url();
        
        // Create a mock endpoint
        let _m = mock_server.mock("GET", "/")
            .with_status(200)
            .with_header("content-type", "text/html")
            .with_body("<html><head><title>PDF Test</title></head><body><h1>Hello PDF World</h1></body></html>")
            .create();
            
        let config = get_test_config();
        let browser = BrowserClient::new(config).unwrap();
        let tab = browser.navigate_to_url(url).unwrap();
        
        // Generate PDF
        let pdf_data = browser.generate_pdf(&tab).unwrap();
        
        // Check that PDF is not empty and has PDF magic bytes
        assert!(!pdf_data.is_empty());
        assert_eq!(&pdf_data[0..4], b"%PDF"); // PDF magic bytes
    }
        let url = &mock_server.url();
        
        // Create a mock endpoint
        let _m = mock_server.mock("GET", "/")
            .with_status(200)
            .with_header("content-type", "text/html")
            .with_body("<html><head><title>Test Page</title></head><body><h1>Hello World</h1></body></html>")
            .create();
        
        let config = get_test_config();
        let browser = BrowserClient::new(config).unwrap();
        let tab = browser.navigate_to_url(url).unwrap();
        
        let result = browser.get_page_html(&tab);
        assert!(result.is_ok());
        
        let html = result.unwrap();
        assert!(html.contains("Hello World"));
    }

    #[test]
    #[ignore] // Skip in CI environments
    fn test_execute_javascript() {
        let mock_server = setup_mock_server();
        let url = &mock_server.url();
        
        // Create a mock endpoint
        let _m = mock_server.mock("GET", "/")
            .with_status(200)
            .with_header("content-type", "text/html")
            .with_body("<html><head><title>Test Page</title></head><body><h1>Hello World</h1></body></html>")
            .create();
        
        let config = get_test_config();
        let browser = BrowserClient::new(config).unwrap();
        let tab = browser.navigate_to_url(url).unwrap();
        
        let result: Result<String, AppError> = browser.execute_javascript(&tab, "document.title");
        assert!(result.is_ok());
        
        let title = result.unwrap();
        assert_eq!(title, "Test Page");
    }
}
