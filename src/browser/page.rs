//! Functions for interacting with browser pages in a functional style
//! Inspired by F# implementation

use std::sync::Arc;
use regex::Regex;
use std::time::Duration;
use rand::Rng;
use crate::browser::domain::{Selector, BrowserError, WaitFor, error};
use crate::error::AppError;
use headless_chrome::{Tab, Browser};
use std::f32;

/// Maps BrowserError to AppError
fn map_error(err: BrowserError) -> AppError {
    match err {
        BrowserError::NotFound(msg) => AppError::NotFoundError(msg),
        BrowserError::NotSupported(msg) => AppError::BrowserError(msg),
        BrowserError::Operation { message, .. } => AppError::BrowserError(message),
    }
}

/// Loads a URL in a new browser tab
pub async fn load(url: &str, browser: &Browser) -> Result<Arc<Tab>, BrowserError> {
    match browser.new_tab() {
        Ok(tab) => {
            // Navigate to URL
            if let Err(e) = tab.navigate_to(url) {
                return error::operation(
                    format!("Failed to navigate to URL: {}", e),
                    None,
                );
            }
            
            // Wait for page to load
            if let Err(e) = tab.wait_until_navigated() {
                return error::operation(
                    format!("Navigation failed: {}", e),
                    None,
                );
            }
            
            // Give page additional time to fully render
            tokio::time::sleep(Duration::from_millis(500)).await;
            
            Ok(tab)
        },
        Err(e) => error::operation(
            format!("Failed to create new tab: {}", e),
            None,
        ),
    }
}

/// Closes a browser tab
pub async fn close(tab: &Arc<Tab>) -> Result<(), BrowserError> {
    // No explicit close method in headless_chrome, tabs are dropped
    // If we wanted to implement this fully we'd need a custom close function
    Ok(())
}

/// Tries to find a DOM element using a selector
pub async fn try_find_locator(selector: &Selector, tab: &Arc<Tab>) -> Result<Option<String>, BrowserError> {
    let script = format!(
        r#"(() => {{
            const el = document.querySelector("{}");
            return el !== null;
        }})()"#,
        selector.value.replace("\"", "\\\"")
    );
    
    match tab.evaluate(&script, false) {
        Ok(result) => {
            match result.value() {
                Ok(value) => {
                    match value {
                        true => Ok(Some(selector.value.clone())),
                        false => Ok(None),
                    }
                },
                Err(e) => error::operation(
                    format!("Failed to evaluate result: {}", e),
                    None,
                ),
            }
        },
        Err(e) => error::operation(
            format!("Failed to find locator. {}", e),
            None,
        ),
    }
}

/// Retry function for operations that might fail temporarily
pub async fn retry<T, F, Fut>(
    delay_ms: u64, 
    attempts: u32, 
    f: F
) -> Result<T, BrowserError> 
where
    F: Fn() -> Fut,
    Fut: std::future::Future<Output = Result<T, BrowserError>>,
{
    let mut current_attempt = 0;
    
    loop {
        match f().await {
            Ok(result) => return Ok(result),
            Err(err) => {
                current_attempt += 1;
                if current_attempt >= attempts {
                    return Err(err);
                }
                tokio::time::sleep(Duration::from_millis(delay_ms)).await;
            }
        }
    }
}

pub mod text {
    use super::*;
    
    /// Tries to find text content of an element
    pub async fn try_find(selector: &Selector, tab: &Arc<Tab>) -> Result<Option<String>, BrowserError> {
        let locator = match try_find_locator(selector, tab).await? {
            None => return error::not_found(format!("Text selector '{}' not found", selector.value)),
            Some(_) => selector.value.clone(),
        };
        
        let script = format!(
            r#"(() => {{
                const el = document.querySelector("{}");
                return el ? el.innerText : null;
            }})()"#,
            locator.replace("\"", "\\\"")
        );
        
        match tab.evaluate(&script, false) {
            Ok(result) => {
                match result.value::<Option<String>>() {
                    Ok(text) => Ok(text),
                    Err(e) => error::operation(
                        format!("Failed to get element text: {}", e),
                        None,
                    ),
                }
            },
            Err(e) => error::operation(
                format!("Failed to evaluate text: {}", e),
                None,
            ),
        }
    }
}

pub mod input {
    use super::*;
    
    /// Fills an input field with text
    pub async fn fill(selector: &Selector, value: &str, tab: &Arc<Tab>) -> Result<(), BrowserError> {
        let locator = match try_find_locator(selector, tab).await? {
            None => return error::not_found(format!("Input selector '{}' not found", selector.value)),
            Some(_) => selector.value.clone(),
        };
        
        let script = format!(
            r#"(() => {{ 
                const el = document.querySelector("{}"); 
                if (!el) return false; 
                el.value = "{}"; 
                return true;
            }})()"#,
            locator.replace("\"", "\\\""),
            value.replace("\"", "\\\"")
        );
        
        match tab.evaluate(&script, false) {
            Ok(result) => {
                match result.value::<bool>() {
                    Ok(true) => Ok(()),
                    Ok(false) => error::not_found(format!("Element not found: {}", selector.value)),
                    Err(e) => error::operation(
                        format!("Failed to fill form: {}", e),
                        None,
                    ),
                }
            },
            Err(e) => error::operation(
                format!("Failed to evaluate script: {}", e),
                None,
            ),
        }
    }
}

pub mod mouse {
    use super::*;
    
    /// Gets random coordinates for mouse movement
    fn get_random_coordinates(period: Duration) -> Vec<(f32, f32)> {
        let count = (period.as_millis() / 10) as usize;
        let mut rng = rand::thread_rng();
        
        fn generate_path(
            mut acc: Vec<(f32, f32)>, 
            remaining_points: usize, 
            current_x: f32, 
            current_y: f32,
            rng: &mut impl Rng,
        ) -> Vec<(f32, f32)> {
            if remaining_points <= 0 {
                acc.reverse();
                return acc;
            }
            
            // Add some randomness to movement
            let max_step = 0.5f32;
            let delta_x = rng.gen::<f32>() * max_step * 2.0 - max_step;
            let delta_y = rng.gen::<f32>() * max_step * 2.0 - max_step;
            
            // Sometimes make bigger jumps to simulate quick movements
            let (next_x, next_y) = if rng.gen::<f32>() < 0.2 {
                let direction = rng.gen_range(0..4);
                match direction {
                    0 => (current_x + 1.0, current_y),
                    1 => (current_x, current_y + 1.0),
                    2 => (current_x - 1.0, current_y),
                    _ => (current_x, current_y - 1.0),
                }
            } else {
                (current_x + delta_x, current_y + delta_y)
            };
            
            // Ensure coordinates are positive
            let next_x = next_x.max(0.0);
            let next_y = next_y.max(0.0);
            
            // Round to one decimal place
            let rounded_x = (next_x * 10.0).round() / 10.0;
            let rounded_y = (next_y * 10.0).round() / 10.0;
            
            acc.push((rounded_x, rounded_y));
            generate_path(acc, remaining_points - 1, rounded_x, rounded_y, rng)
        }
        
        // Start at origin and generate a path
        generate_path(Vec::with_capacity(count), count, 0.0, 0.0, &mut rng)
    }
    
    /// Clicks on an element
    pub async fn click(selector: &Selector, wait_for: WaitFor, tab: &Arc<Tab>) -> Result<(), BrowserError> {
        let locator = match try_find_locator(selector, tab).await? {
            None => return error::not_found(format!("Mouse selector '{}' not found", selector.value)),
            Some(_) => selector.value.clone(),
        };
        
        let script = format!(
            r#"(() => {{ 
                const el = document.querySelector("{}"); 
                if (!el) return false;
                el.click(); 
                return true;
            }})()"#,
            locator.replace("\"", "\\\"")
        );
        
        match tab.evaluate(&script, false) {
            Ok(result) => {
                match result.value::<bool>() {
                    Ok(true) => {
                        match wait_for {
                            WaitFor::Url(pattern) => {
                                // Wait for navigation to complete with URL matching pattern
                                if let Err(e) = tab.wait_until_navigated() {
                                    return error::operation(
                                        format!("Failed waiting for navigation: {}", e),
                                        None,
                                    );
                                }
                                
                                // Verify the URL matches the pattern
                                let current_url = match tab.get_url() {
                                    Ok(url) => url,
                                    Err(e) => return error::operation(
                                        format!("Failed to get URL after navigation: {}", e),
                                        None,
                                    ),
                                };
                                
                                if let Ok(regex) = Regex::new(&pattern) {
                                    if !regex.is_match(&current_url) {
                                        return error::operation(
                                            format!("URL after navigation '{}' doesn't match expected pattern '{}'", current_url, pattern),
                                            None,
                                        );
                                    }
                                }
                            },
                            WaitFor::Selector(wait_selector) => {
                                // Wait for the selector to be found
                                if try_find_locator(&wait_selector, tab).await?.is_none() {
                                    return error::not_found(
                                        format!("Wait selector '{}' not found after click", wait_selector.value)
                                    );
                                }
                            },
                            WaitFor::Nothing => (),
                        }
                        Ok(())
                    },
                    Ok(false) => error::not_found(format!("Element not found: {}", selector.value)),
                    Err(e) => error::operation(
                        format!("Failed to click element: {}", e),
                        None,
                    ),
                }
            },
            Err(e) => error::operation(
                format!("Failed to evaluate script: {}", e),
                None,
            ),
        }
    }
    
    /// Moves mouse in random patterns to simulate human behavior
    pub async fn shuffle(period: Duration, tab: &Arc<Tab>) -> Result<(), BrowserError> {
        let coordinates = get_random_coordinates(period);
        
        // In headless_chrome there's no direct mouse movement API that's easily accessible
        // This would be a more complex implementation in a real-world scenario
        // We'd need to use Chrome DevTools Protocol directly for this level of control
        
        // Placeholder implementation
        for (x, y) in coordinates {
            // In a real implementation, we'd move the mouse to each coordinate
            // For now we'll just simulate the delay
            tokio::time::sleep(Duration::from_millis(10)).await;
        }
        
        Ok(())
    }
}

/// Form handling functions
pub mod form {
    use super::*;

    /// Submits a form by its selector
    pub async fn submit(selector: &Selector, tab: &Arc<Tab>) -> Result<(), BrowserError> {
        let locator = match try_find_locator(selector, tab).await? {
            None => return error::not_found(format!("Form selector '{}' not found", selector.value)),
            Some(_) => selector.value.clone(),
        };
        
        let script = format!(
            r#"(() => {{ 
                const form = document.querySelector("{}"); 
                if (!form) return false;
                form.submit(); 
                return true;
            }})()"#,
            locator.replace("\"", "\\\"")
        );
        
        match tab.evaluate(&script, false) {
            Ok(result) => {
                match result.value::<bool>() {
                    Ok(true) => {
                        // Wait for navigation to complete after form submission
                        if let Err(e) = tab.wait_until_navigated() {
                            return error::operation(
                                format!("Failed waiting for navigation after form submission: {}", e),
                                None,
                            );
                        }
                        
                        Ok(())
                    },
                    Ok(false) => error::not_found(format!("Form not found: {}", selector.value)),
                    Err(e) => error::operation(
                        format!("Failed to submit form: {}", e),
                        None,
                    ),
                }
            },
            Err(e) => error::operation(
                format!("Failed to evaluate script: {}", e),
                None,
            ),
        }
    }
    
    /// Fills multiple fields in a form
    pub async fn fill_fields(
        fields: &[(&Selector, &str)], 
        tab: &Arc<Tab>
    ) -> Result<(), BrowserError> {
        for (selector, value) in fields {
            input::fill(selector, value, tab).await?;
        }
        Ok(())
    }
    
    /// Submits a form after filling it with values
    pub async fn fill_and_submit(
        fields: &[(&Selector, &str)],
        submit_selector: &Selector,
        wait_option: WaitFor,
        tab: &Arc<Tab>
    ) -> Result<(), BrowserError> {
        // Fill all fields
        fill_fields(fields, tab).await?;
        
        // Click the submit button with provided wait option
        mouse::click(submit_selector, wait_option, tab).await
    }
}
