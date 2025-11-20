use actix_web::{get, post, web, HttpResponse};
use askama::Template;
use serde_json::Value;

use crate::{
    db::{self, events::Events},
    templates::SetScoresTemplate,
    websocket::{ChannelsActor, Publish},
    AppState,
};

#[get("/set_scores")]
pub async fn get(state: web::Data<AppState>) -> HttpResponse {
    let events = Events::all(&state.pool).await.unwrap();
    HttpResponse::Ok().body(
        SetScoresTemplate {
            events,
            forms: state.config.forms.clone(),
            scores: state.config.scores.clone(),
        }
        .render()
        .expect("Template should be valid"),
    )
}

#[post("/set_scores")]
pub async fn post(
    state: web::Data<AppState>,
    body: String,
    channels: web::Data<actix::Addr<ChannelsActor>>,
) -> HttpResponse {
    let body: Value = serde_json::from_str(body.as_str()).unwrap();

    for events in body.as_object().unwrap() {
        let event_id = events.0;
        let event_scores = events.1;
        println!("Making DB Call with {} and {:#?}", event_id, event_scores);
        db::events::Events::set_scores(&state.pool, event_id.to_owned(), event_scores.to_owned())
            .await
            .unwrap();
    }

    let scores = crate::utils::render_scoreboard(state).await;
    channels.do_send(Publish {
        channel: "scores".to_string(),
        payload: scores,
    });

    HttpResponse::NoContent().finish()
}
