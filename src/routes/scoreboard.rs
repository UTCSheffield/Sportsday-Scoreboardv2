use actix_web::{get, web, HttpResponse};
use askama::Template;

use crate::{templates::ScoreboardTemplate, utils, AppState};

#[get("/scoreboard")]
pub async fn get(state: web::Data<AppState>) -> HttpResponse {
    let scores = utils::render_scoreboard(state).await;
    let html = ScoreboardTemplate { scores }
        .render()
        .expect("template should be valid");

    HttpResponse::Ok().body(html)
}
