//! Domain models for browser operations

use std::time::Duration;
use regex::Regex;

/// Represents a CSS selector for locating elements in the DOM
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Selector {
    pub value: String,
}

impl Selector {
    pub fn new(value: impl Into<String>) -> Self {
        Self { value: value.into() }
    }
}

impl From<String> for Selector {
    fn from(value: String) -> Self {
        Self { value }
    }
}

impl From<&str> for Selector {
    fn from(value: &str) -> Self {
        Self { value: value.to_string() }
    }
}

/// Represents different types of mouse wait actions
#[derive(Debug, Clone)]
pub enum WaitFor {
    /// Wait for a URL that matches a pattern
    Url(String),
    /// Wait for an element that matches a selector
    Selector(Selector),
    /// Don't wait for anything
    Nothing,
}

/// Error types for browser operations
#[derive(Debug, thiserror::Error)]
pub enum BrowserError {
    #[error("Not found: {0}")]
    NotFound(String),
    
    #[error("Operation not supported: {0}")]
    NotSupported(String),
    
    #[error("Operation failed: {message}")]
    Operation {
        message: String,
        source_info: Option<SourceInfo>,
    },
}

/// Source code location information for error reporting
#[derive(Debug, Clone)]
pub struct SourceInfo {
    pub directory: &'static str,
    pub file: &'static str,
    pub line: u32,
}

/// Utility functions for error handling
pub mod error {
    use super::*;
    use std::fmt::Display;
    
    pub fn not_found<T, E: Display>(msg: E) -> Result<T, BrowserError> {
        Err(BrowserError::NotFound(msg.to_string()))
    }
    
    pub fn not_supported<T, E: Display>(msg: E) -> Result<T, BrowserError> {
        Err(BrowserError::NotSupported(msg.to_string()))
    }
    
    pub fn operation<T, E: Display>(msg: E, source_info: Option<SourceInfo>) -> Result<T, BrowserError> {
        Err(BrowserError::Operation {
            message: msg.to_string(),
            source_info,
        })
    }
}
