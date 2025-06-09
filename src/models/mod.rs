use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
struct ErrorInfo {
    message: String,
    code: Option<String>,
}

#[derive(Serialize)]
struct GenericResponse {
    success: bool,
    message: String,
}