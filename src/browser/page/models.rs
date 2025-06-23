use serde::{Deserialize, Serialize};

#[derive(Deserialize)]
pub struct LoadRequest {
  pub url: String,
  //TODO: Add expiration time
}

#[derive(Serialize)]
pub struct LoadResponse {
  pub tab_id: String,
}

#[derive(Deserialize)]
pub struct CloseRequest {
  pub tab_id: String,
}

#[derive(Deserialize)]
pub struct PostElement {
  pub selector: String,
  pub value: String,
}

#[derive(Deserialize)]
pub struct FillElementsRequest {
  pub tab_id: String,
  pub elements: Vec<PostElement>,
}
