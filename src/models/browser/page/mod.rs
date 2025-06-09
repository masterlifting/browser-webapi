use serde::{Deserialize, Serialize};

#[derive(Deserialize)]
struct Selector {
    value: String,
}

#[derive(Deserialize)]
struct LoadRequest {
    url: String,
}

#[derive(Serialize)]
struct LoadResponse {
    session: BrowserSession,
    url: String,
}

#[derive(Deserialize)]
struct TextFindRequest {
    session_id: String,
    page_id: String,
    selector: Selector,
}

#[derive(Serialize)]
struct TextFindResponse {
    text: Option<String>,
}

#[derive(Deserialize)]
struct InputFillRequest {
    session_id: String,
    page_id: String,
    selector: Selector,
    value: String,
}

#[derive(Deserialize)]
struct MouseClickRequest {
    session_id: String,
    page_id: String,
    selector: Selector,
    wait_for: WaitForOption,
}

#[derive(Deserialize)]
#[serde(tag = "type")]
enum WaitForOption {
    #[serde(rename = "url")]
    Url { pattern: String },
    #[serde(rename = "selector")]
    Selector { value: String },
    #[serde(rename = "nothing")]
    Nothing,
}

#[derive(Deserialize)]
struct MouseShuffleRequest {
    session_id: String,
    page_id: String,
    period_ms: u64,
}

#[derive(Deserialize)]
struct FormSubmitRequest {
    session_id: String,
    page_id: String,
    selector: Selector,
    url_pattern: String,
}