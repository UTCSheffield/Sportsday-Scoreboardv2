pub mod users;

use actix_web::{get, post, web, HttpResponse};
use askama::Template;

use crate::templates::{AdminConsoleTemplate, AdminIndexTemplate};

#[get("")]
pub async fn get() -> HttpResponse {
    HttpResponse::Ok().body(
        AdminIndexTemplate {}
            .render()
            .expect("Template should be valid"),
    )
}

#[get("/console")]
pub async fn console(app_state: web::Data<crate::AppState>) -> HttpResponse {
    let log_entries = app_state.log_collector.get_entries();

    HttpResponse::Ok().body(
        AdminConsoleTemplate { log_entries }
            .render()
            .expect("Template should be valid"),
    )
}

#[post("/console/clear")]
pub async fn clear_console(app_state: web::Data<crate::AppState>) -> HttpResponse {
    app_state.log_collector.clear();
    HttpResponse::Ok().json(serde_json::json!({"success": true}))
}
