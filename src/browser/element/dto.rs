use serde::Deserialize;

#[derive(Deserialize)]
pub struct ClickDto {
  pub selector: String,
}

#[derive(Deserialize)]
pub struct ExistsDto {
  pub selector: String,
}

#[derive(Deserialize)]
pub struct ExtractDto {
  pub selector: String,
}

#[derive(Deserialize)]
pub struct ExecuteDto {
  pub selector: String,
  pub function: String,
}
