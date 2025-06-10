use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
pub struct Error {
    message: String,
    code: Option<String>,
}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match &self.code {
            Some(code) => write!(f, "{} ({})", self.message, code),
            None => write!(f, "{}", self.message),
        }
    }
}

#[derive(Serialize)]
pub struct Response<T>
where
    T: Serialize,
{
    pub success: bool,
    pub message: String,
    pub payload: Option<T>,
}
