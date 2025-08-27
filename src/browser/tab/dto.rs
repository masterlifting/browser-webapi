use serde::Deserialize;

use crate::browser::element::dto::PostElement;

#[derive(Deserialize)]
pub struct OpenRequest {
  pub url: String,
  //TODO: add expiration
}

#[derive(Deserialize)]
pub struct FillRequest {
  pub inputs: Vec<PostElement>,
}
