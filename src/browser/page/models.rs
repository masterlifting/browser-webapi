use serde::{Deserialize, Serialize};

use crate::models::browser::Session;

#[derive(Deserialize)]
pub struct Selector {
    value: String,
}

#[derive(Deserialize)]
pub struct LoadRequest {
    pub url: String,
}

#[derive(Serialize)]
pub struct LoadResponse {
    session: Session,
    url: String,
}

#[derive(Deserialize)]
pub struct TextFindRequest {
    session_id: String,
    page_id: String,
    selector: Selector,
}

#[derive(Serialize)]
pub struct TextFindResponse {
    text: Option<String>,
}

#[derive(Deserialize)]
pub struct InputFillRequest {
    session_id: String,
    page_id: String,
    selector: Selector,
    value: String,
}

#[derive(Deserialize)]
pub struct MouseClickRequest {
    session_id: String,
    page_id: String,
    selector: Selector,
    wait_for: WaitForOption,
}

#[derive(Deserialize)]
#[serde(tag = "type")]
pub enum WaitForOption {
    #[serde(rename = "url")]
    Url { pattern: String },
    #[serde(rename = "selector")]
    Selector { value: String },
    #[serde(rename = "nothing")]
    Nothing,
}

#[derive(Deserialize)]
pub struct MouseShuffleRequest {
    session_id: String,
    page_id: String,
    period_ms: u64,
}

#[derive(Deserialize)]
pub struct FormSubmitRequest {
    session_id: String,
    page_id: String,
    selector: Selector,
    url_pattern: String,
}
