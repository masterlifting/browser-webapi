use actix_web::HttpResponse;

use crate::models::Error;

pub fn from_error(e: Error) -> HttpResponse {
  match e {
    Error::NotFound(msg) => HttpResponse::NotFound().body(msg),
    error => HttpResponse::BadRequest().body(error.to_string()),
  }
}

pub fn from_string(res: Result<String, Error>) -> HttpResponse {
  res.map_or_else(from_error, |s| HttpResponse::Ok().body(s))
}

pub fn from_image(res: Result<Vec<u8>, Error>) -> HttpResponse {
  res.map_or_else(from_error, |bytes| {
    HttpResponse::Ok().content_type("image/png").body(bytes)
  })
}

pub fn from_unit(res: Result<(), Error>) -> HttpResponse {
  res.map_or_else(from_error, |()| HttpResponse::Ok().finish())
}
