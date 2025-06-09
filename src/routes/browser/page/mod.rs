use crate::handlers::browser::page;

use actix_web::web;
use headless_chrome::{Browser, Tab};
use lazy_static::lazy_static;
use std::collections::HashMap;
use std::sync::{Arc, Mutex};

// Store browser sessions in memory
lazy_static! {
    pub(crate) static ref BROWSER_SESSIONS: Mutex<HashMap<String, Arc<Browser>>> =
        Mutex::new(HashMap::new());
    pub(crate) static ref PAGE_SESSIONS: Mutex<HashMap<String, Arc<Tab>>> =
        Mutex::new(HashMap::new());
}

pub fn configure() -> actix_web::Scope {
    web::scope("/page")
        .route("/load", web::post().to(page::load))
        .route("/close", web::post().to(page::close))
        .route("/text/find", web::post().to(page::find_text))
        .route("/input/fill", web::post().to(page::fill_input))
        .route("/mouse/click", web::post().to(page::mouse_click))
        .route("/mouse/shuffle", web::post().to(page::mouse_shuffle))
        .route("/form/submit", web::post().to(page::form_submit))
}
