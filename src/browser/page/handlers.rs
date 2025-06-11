use actix_web::{HttpResponse, web};
use headless_chrome::browser;
use headless_chrome::{Browser, LaunchOptions, Tab};
use lazy_static::lazy_static;
use regex::Regex;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use std::sync::Mutex;
use std::time::Duration;
use url::Url;
use uuid::Uuid;

use crate::browser::{
    models::{Session, SessionRequest, SessionResponse},
    page::models::{
        FormSubmitRequest, InputFillRequest, LoadRequest, LoadResponse, MouseClickRequest,
        MouseShuffleRequest, Selector, TextFindRequest, TextFindResponse, WaitForOption,
    },
};
use crate::models::{Error, Response};

lazy_static! {
    static ref SESSIONS: Mutex<HashMap<String, Arc<Tab>>> = Mutex::new(HashMap::new());
}

fn get_session(page_id: &str) -> Result<Arc<Tab>, Error> {
    let pages = SESSIONS.lock().unwrap();
    pages
        .get(page_id)
        .cloned()
        .ok_or_else(|| Error::NotFound(format!("Session with ID {} not found", page_id)))
}

fn try_find_element(
    tab: &Arc<Tab>,
    selector: &str,
    retries: usize,
) -> Result<Option<headless_chrome::Element>, String> {
    for attempt in 0..retries {
        match tab.wait_for_element(selector) {
            Ok(element) => return Ok(Some(element)),
            Err(e) if attempt < retries - 1 => {
                std::thread::sleep(Duration::from_millis(1000));
                continue;
            }
            Err(e) => return Err(format!("Failed to find element: {}", e)),
        }
    }
    Ok(None)
}

// Route handlers
async fn load(browser: Arc<Browser>, req: web::Json<LoadRequest>) -> HttpResponse {
    match Url::parse(&req.url) {
        Ok(url) => {
            m
            let tab = match browser.new_tab() {
                Ok(tab) => Arc::new(tab),
                Err(e) => {
                    return HttpResponse::InternalServerError().json(Error {
                        message: format!("Failed to create new tab: {}", e),
                        code: None,
                    });
                }
            };

            // Navigate to the URL
            if let Err(e) = tab.navigate_to(url.as_str()) {
                return HttpResponse::InternalServerError().json(Error {
                    message: format!("Failed to navigate to URL: {}", e),
                    code: None,
                });
            }

            // Wait for the page to load
            if let Err(e) = tab.wait_until_navigated() {
                return HttpResponse::InternalServerError().json(Error {
                    message: format!("Failed to wait for navigation: {}", e),
                    code: None,
                });
            }

            // Generate a unique session ID
            let session_id = Uuid::new_v4().to_string();
            SESSIONS.lock().unwrap().insert(session_id.clone(), Arc::clone(&tab));

            HttpResponse::Ok().json(LoadResponse {
                session: Session {
                    id: session_id,
                    page_id: tab.id(),
                },
                url: url.to_string(),
            })
        }
        Err(e) => HttpResponse::BadRequest().json(Error {
            message: format!("Invalid URL provided: {}", e),
            code: None,
        }),
    }
async fn close(req: web::Json<SessionRequest>) -> HttpResponse {
    let result = get_page_session(&req.session_id, &req.page_id);

    match result {
        Ok(tab) => {
            // Close tab explicitly
            if let Err(e) = tab.close(true) {
                return HttpResponse::InternalServerError().json(ErrorInfo {
                    message: format!("Failed to close tab: {}", e),
                    code: Some("CLOSE_TAB_FAILED".to_string()),
                });
            }

            // Remove sessions from storage
            PAGE_SESSIONS.lock().unwrap().remove(&req.page_id);
            if PAGE_SESSIONS.lock().unwrap().is_empty() {
                BROWSER_SESSIONS.lock().unwrap().remove(&req.session_id);
            }

            HttpResponse::Ok().json(GenericResponse {
                success: true,
                message: "Page closed successfully".to_string(),
            })
        }
        Err(e) => HttpResponse::BadRequest().json(ErrorInfo {
            message: e,
            code: Some("SESSION_NOT_FOUND".to_string()),
        }),
    }
}

async fn find_text(req: web::Json<TextFindRequest>) -> HttpResponse {
    let tab_result = get_page_session(&req.session_id, &req.page_id);

    match tab_result {
        Ok(tab) => match try_find_element(&tab, &req.selector.value, 3) {
            Ok(Some(element)) => match element.get_inner_text() {
                Ok(text) => HttpResponse::Ok().json(TextFindResponse { text: Some(text) }),
                Err(e) => HttpResponse::InternalServerError().json(ErrorInfo {
                    message: format!("Failed to get element text: {}", e),
                    code: Some("GET_TEXT_FAILED".to_string()),
                }),
            },
            Ok(None) => HttpResponse::NotFound().json(ErrorInfo {
                message: format!("Text selector '{}' not found", req.selector.value),
                code: Some("SELECTOR_NOT_FOUND".to_string()),
            }),
            Err(e) => HttpResponse::InternalServerError().json(ErrorInfo {
                message: e,
                code: Some("FIND_ELEMENT_FAILED".to_string()),
            }),
        },
        Err(e) => HttpResponse::BadRequest().json(ErrorInfo {
            message: e,
            code: Some("SESSION_NOT_FOUND".to_string()),
        }),
    }
}

async fn fill_input(req: web::Json<InputFillRequest>) -> HttpResponse {
    let tab_result = get_page_session(&req.session_id, &req.page_id);

    match tab_result {
        Ok(tab) => {
            match try_find_element(&tab, &req.selector.value, 3) {
                Ok(Some(element)) => {
                    // Clear existing input first
                    if let Err(e) = element.click() {
                        return HttpResponse::InternalServerError().json(ErrorInfo {
                            message: format!("Failed to focus input element: {}", e),
                            code: Some("FOCUS_ELEMENT_FAILED".to_string()),
                        });
                    }

                    // Fill in the input with new value
                    if let Err(e) = element.type_into(&req.value) {
                        return HttpResponse::InternalServerError().json(ErrorInfo {
                            message: format!("Failed to fill input: {}", e),
                            code: Some("FILL_INPUT_FAILED".to_string()),
                        });
                    }

                    HttpResponse::Ok().json(GenericResponse {
                        success: true,
                        message: "Input filled successfully".to_string(),
                    })
                }
                Ok(None) => HttpResponse::NotFound().json(ErrorInfo {
                    message: format!("Input selector '{}' not found", req.selector.value),
                    code: Some("SELECTOR_NOT_FOUND".to_string()),
                }),
                Err(e) => HttpResponse::InternalServerError().json(ErrorInfo {
                    message: e,
                    code: Some("FIND_ELEMENT_FAILED".to_string()),
                }),
            }
        }
        Err(e) => HttpResponse::BadRequest().json(ErrorInfo {
            message: e,
            code: Some("SESSION_NOT_FOUND".to_string()),
        }),
    }
}

async fn mouse_click(req: web::Json<MouseClickRequest>) -> HttpResponse {
    let tab_result = get_page_session(&req.session_id, &req.page_id);

    match tab_result {
        Ok(tab) => {
            match try_find_element(&tab, &req.selector.value, 3) {
                Ok(Some(element)) => {
                    // Click the element
                    if let Err(e) = element.click() {
                        return HttpResponse::InternalServerError().json(ErrorInfo {
                            message: format!("Failed to click element: {}", e),
                            code: Some("CLICK_ELEMENT_FAILED".to_string()),
                        });
                    }

                    // Handle waiting based on the wait_for option
                    match &req.wait_for {
                        WaitForOption::Url { pattern } => {
                            // Wait for URL to change
                            std::thread::sleep(Duration::from_millis(500)); // Simple wait for navigation
                            let current_url = match tab.get_url() {
                                Ok(url) => url,
                                Err(e) => {
                                    return HttpResponse::InternalServerError().json(ErrorInfo {
                                        message: format!("Failed to get URL after click: {}", e),
                                        code: Some("GET_URL_FAILED".to_string()),
                                    });
                                }
                            };

                            // Check if URL matches pattern
                            let re = match Regex::new(pattern) {
                                Ok(re) => re,
                                Err(e) => {
                                    return HttpResponse::BadRequest().json(ErrorInfo {
                                        message: format!("Invalid regex pattern: {}", e),
                                        code: Some("INVALID_REGEX".to_string()),
                                    });
                                }
                            };

                            if !re.is_match(&current_url) {
                                return HttpResponse::BadRequest().json(ErrorInfo {
                                    message: format!(
                                        "URL did not match expected pattern: {}",
                                        pattern
                                    ),
                                    code: Some("URL_PATTERN_MISMATCH".to_string()),
                                });
                            }
                        }
                        WaitForOption::Selector { value } => {
                            // Wait for element to appear
                            match try_find_element(&tab, value, 3) {
                                Ok(Some(_)) => {} // Element found, continue
                                Ok(None) => {
                                    return HttpResponse::NotFound().json(ErrorInfo {
                                        message: format!(
                                            "Selector '{}' not found after click",
                                            value
                                        ),
                                        code: Some("WAIT_SELECTOR_NOT_FOUND".to_string()),
                                    });
                                }
                                Err(e) => {
                                    return HttpResponse::InternalServerError().json(ErrorInfo {
                                        message: e,
                                        code: Some("FIND_ELEMENT_FAILED".to_string()),
                                    });
                                }
                            }
                        }
                        WaitForOption::Nothing => {
                            // No waiting required
                        }
                    }

                    HttpResponse::Ok().json(GenericResponse {
                        success: true,
                        message: "Element clicked successfully".to_string(),
                    })
                }
                Ok(None) => HttpResponse::NotFound().json(ErrorInfo {
                    message: format!("Click selector '{}' not found", req.selector.value),
                    code: Some("SELECTOR_NOT_FOUND".to_string()),
                }),
                Err(e) => HttpResponse::InternalServerError().json(ErrorInfo {
                    message: e,
                    code: Some("FIND_ELEMENT_FAILED".to_string()),
                }),
            }
        }
        Err(e) => HttpResponse::BadRequest().json(ErrorInfo {
            message: e,
            code: Some("SESSION_NOT_FOUND".to_string()),
        }),
    }
}

async fn mouse_shuffle(req: web::Json<MouseShuffleRequest>) -> HttpResponse {
    let tab_result = get_page_session(&req.session_id, &req.page_id);

    match tab_result {
        Ok(tab) => {
            // This is a simplified version as headless_chrome has more limited mouse movement capabilities
            // than Playwright, so we'll simulate with JavaScript instead
            let script = r#"
                (() => {
                    const duration = arguments[0];
                    const moveEvery = 10;
                    const moves = Math.floor(duration / moveEvery);
                    
                    for (let i = 0; i < moves; i++) {
                        setTimeout(() => {
                            const x = Math.floor(Math.random() * window.innerWidth);
                            const y = Math.floor(Math.random() * window.innerHeight);
                            const moveEvent = new MouseEvent('mousemove', {
                                view: window,
                                bubbles: true,
                                cancelable: true,
                                clientX: x,
                                clientY: y
                            });
                            document.dispatchEvent(moveEvent);
                        }, i * moveEvery);
                    }
                    return true;
                })()
            "#;

            match tab.evaluate(script, vec![req.period_ms.into()], false) {
                Ok(_) => {
                    // Wait for the movement duration plus a small buffer
                    std::thread::sleep(Duration::from_millis(req.period_ms + 100));

                    HttpResponse::Ok().json(GenericResponse {
                        success: true,
                        message: "Mouse shuffle completed".to_string(),
                    })
                }
                Err(e) => HttpResponse::InternalServerError().json(ErrorInfo {
                    message: format!("Failed to shuffle mouse: {}", e),
                    code: Some("MOUSE_SHUFFLE_FAILED".to_string()),
                }),
            }
        }
        Err(e) => HttpResponse::BadRequest().json(ErrorInfo {
            message: e,
            code: Some("SESSION_NOT_FOUND".to_string()),
        }),
    }
}

async fn form_submit(req: web::Json<FormSubmitRequest>) -> HttpResponse {
    let tab_result = get_page_session(&req.session_id, &req.page_id);

    match tab_result {
        Ok(tab) => {
            match try_find_element(&tab, &req.selector.value, 3) {
                Ok(Some(form_element)) => {
                    // Prepare regex for URL pattern
                    let url_regex = match Regex::new(&req.url_pattern) {
                        Ok(re) => re,
                        Err(e) => {
                            return HttpResponse::BadRequest().json(ErrorInfo {
                                message: format!("Invalid regex pattern: {}", e),
                                code: Some("INVALID_REGEX".to_string()),
                            });
                        }
                    };

                    // Submit the form using JavaScript
                    let script = r#"
                        (form) => {
                            form.submit();
                            return true;
                        }
                    "#;

                    if let Err(e) = form_element.call_js_fn(script, vec![], false) {
                        return HttpResponse::InternalServerError().json(ErrorInfo {
                            message: format!("Failed to submit form: {}", e),
                            code: Some("FORM_SUBMIT_FAILED".to_string()),
                        });
                    }

                    // Wait for navigation
                    std::thread::sleep(Duration::from_millis(1000)); // Simple wait for navigation

                    // Check if URL matches pattern
                    let current_url = match tab.get_url() {
                        Ok(url) => url,
                        Err(e) => {
                            return HttpResponse::InternalServerError().json(ErrorInfo {
                                message: format!("Failed to get URL after form submission: {}", e),
                                code: Some("GET_URL_FAILED".to_string()),
                            });
                        }
                    };

                    if !url_regex.is_match(&current_url) {
                        return HttpResponse::BadRequest().json(ErrorInfo {
                            message: format!(
                                "URL did not match expected pattern after form submission: {}",
                                req.url_pattern
                            ),
                            code: Some("URL_PATTERN_MISMATCH".to_string()),
                        });
                    }

                    HttpResponse::Ok().json(GenericResponse {
                        success: true,
                        message: "Form submitted successfully".to_string(),
                    })
                }
                Ok(None) => HttpResponse::NotFound().json(ErrorInfo {
                    message: format!("Form selector '{}' not found", req.selector.value),
                    code: Some("SELECTOR_NOT_FOUND".to_string()),
                }),
                Err(e) => HttpResponse::InternalServerError().json(ErrorInfo {
                    message: e,
                    code: Some("FIND_ELEMENT_FAILED".to_string()),
                }),
            }
        }
        Err(e) => HttpResponse::BadRequest().json(ErrorInfo {
            message: e,
            code: Some("SESSION_NOT_FOUND".to_string()),
        }),
    }
}
