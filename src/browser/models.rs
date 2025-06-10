use serde::{Deserialize, Serialize};

#[derive(Serialize)]
pub struct Session {
    pub id: String,
    pub page_id: String,
}

#[derive(Deserialize)]
pub struct SessionRequest {
    pub session_id: String,
    pub page_id: String,
}

#[derive(Serialize)]
pub struct SessionResponse {
    pub session_id: String,
    pub page_id: String,
}
