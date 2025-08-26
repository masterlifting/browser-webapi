use serde::Deserialize;

#[derive(Deserialize)]
pub struct OpenRequest {
  pub url: String,
  //TODO: add expiration
}

#[derive(Deserialize)]
pub struct GetElement {
  pub selector: String,
}

#[derive(Deserialize)]
pub struct PostElement {
  pub selector: String,
  pub value: String,
}

#[derive(Deserialize)]
pub struct FillRequest {
  pub inputs: Vec<PostElement>,
}
