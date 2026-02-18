use serde::Deserialize;

#[derive(Deserialize)]
pub struct OpenDto {
  pub url: String,
  #[serde(default = "default_expiration")]
  pub expiration: u64,
}

fn default_expiration() -> u64 {
  30 // default expiration time in seconds
}

impl OpenDto {
  #[must_use]
  pub(crate) fn bounded_expiration(&self) -> u64 {
    self.expiration.clamp(1, 3600)
  }
}

#[derive(Deserialize)]
pub struct InputDto {
  pub selector: String,
  pub value: String,
}

#[derive(Deserialize)]
pub struct FillDto {
  pub inputs: Vec<InputDto>,
}
