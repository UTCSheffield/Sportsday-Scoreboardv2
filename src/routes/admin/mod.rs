pub mod console;
pub mod users;

use actix_web::{get, HttpResponse};
use askama::Template;

use crate::templates::AdminIndexTemplate;

#[get("")]
pub async fn get() -> HttpResponse {
    HttpResponse::Ok().body(
        AdminIndexTemplate {}
            .render()
            .expect("Template should be valid"),
    )
}
