use actix_web::{get, web, HttpResponse};
use askama::Template;
use serde_json::Value;

use crate::{
    configurator::parser::Year,
    db,
    templates::{IndexTemplate, ResultsTemplate},
    AppState,
};

#[get("/results")]
pub async fn get(state: web::Data<AppState>) -> HttpResponse {
    let events = db::events::Events::all(&state.pool).await.unwrap();
    let mut results_events: Vec<ResultsEvent> = Vec::new();

    for event in events.iter() {
        results_events.push(ResultsEvent {
            name: event.name.clone(),
            year: state
                .config
                .years
                .iter()
                .filter(|year| year.id == event.year_id)
                .collect::<Vec<&Year>>()[0]
                .name
                .clone(),
            group: event.gender_id.clone(),
            scores: serde_json::from_str::<Value>(event.scores.as_str()).unwrap(),
        });
    }

    HttpResponse::Ok().body(
        ResultsTemplate {
            forms: state.config.forms.clone(),
            events: results_events,
        }
        .render()
        .expect("Template should be valid"),
    )
}

pub struct ResultsEvent {
    pub name: String,
    pub year: String,
    pub group: String,
    pub scores: Value,
}
