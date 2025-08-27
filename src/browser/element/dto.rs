use serde::Deserialize;

#[derive(Deserialize)]
pub struct GetElement {
  pub selector: String,
}

#[derive(Deserialize)]
pub struct PostElement {
  pub selector: String,
  pub value: String,
}
