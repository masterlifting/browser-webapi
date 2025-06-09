use serde::{Deserialize, Serialize};

pub mod page;

// Common types shared across browser functionality
#[derive(Serialize)]
pub struct BrowserSession {
    pub id: String,
    pub page_id: String,
}

#[derive(Deserialize)]
pub struct SessionRequest {
    pub session_id: String,
    pub page_id: String,
}

#[derive(Serialize)]
pub struct ErrorInfo {
    pub message: String,
    pub code: Option<String>,
}

#[derive(Serialize)]
pub struct GenericResponse {
    pub success: bool,
    pub message: String,
}

// Helper types for functional operations
pub type Result<T> = std::result::Result<T, ErrorInfo>;

// Error creation functions - functional helpers
pub fn browser_error<T>(message: impl Into<String>) -> Result<T> {
    Err(ErrorInfo {
        message: message.into(),
        code: Some("BROWSER_ERROR".to_string()),
    })
}

pub fn not_found_error<T>(message: impl Into<String>) -> Result<T> {
    Err(ErrorInfo {
        message: message.into(),
        code: Some("NOT_FOUND".to_string()),
    })
}

pub fn validation_error<T>(message: impl Into<String>) -> Result<T> {
    Err(ErrorInfo {
        message: message.into(),
        code: Some("VALIDATION_ERROR".to_string()),
    })
}
