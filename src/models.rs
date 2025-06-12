use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
pub struct ErrorInfo {
  pub message: String,
  pub code: Option<String>,
}

impl std::fmt::Display for ErrorInfo {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    match &self.code {
      Some(code) => write!(f, "{} ({})", self.message, code),
      None => write!(f, "{}", self.message),
    }
  }
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(tag = "type")]
pub enum Error {
  #[serde(rename = "not_found")]
  NotFound(String),
  #[serde(rename = "not_implemented")]
  NotImplemented(String),
  #[serde(rename = "not_supported")]
  NotSupported(String),
  #[serde(rename = "canceled")]
  Canceled(String),
  #[serde(rename = "operation_error")]
  Operation(ErrorInfo),
}

impl std::fmt::Display for Error {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    match self {
      Error::NotFound(msg) => write!(f, "Not Found: {}", msg),
      Error::NotImplemented(msg) => write!(f, "Not Implemented: {}", msg),
      Error::NotSupported(msg) => write!(f, "Not Supported: {}", msg),
      Error::Canceled(msg) => write!(f, "Canceled: {}", msg),
      Error::Operation(info) => write!(f, "Operation Error: {}", info),
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
