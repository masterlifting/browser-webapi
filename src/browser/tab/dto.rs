use serde::Deserialize;

#[derive(Deserialize)]
pub struct OpenDto {
  pub url: String,
  #[serde(default = "default_expiration_seconds")]
  pub expiration_seconds: u64,
}

fn default_expiration_seconds() -> u64 {
  30
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
