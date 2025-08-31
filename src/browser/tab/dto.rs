use serde::Deserialize;

#[derive(Deserialize)]
pub struct OpenDto {
  pub url: String,
  //TODO: add expiration
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
