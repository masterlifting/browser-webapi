use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
struct Error {
    message: String,
    code: Option<String>,
}

#[derive(Serialize)]
struct Response {
    success: bool,
    message: String,
}
