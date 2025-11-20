use std::collections::HashMap;

use actix_web::web;
use askama::Template;

use crate::{
    db::{events::Events, years::Years},
    templates::ScoreboardPartialTemplate,
    AppState,
};

pub async fn render_scoreboard(state: web::Data<AppState>) -> String {
    let forms: Vec<crate::configurator::parser::Form> = state.config.forms.clone();
    let years = Years::all(&state.pool).await.unwrap();
    let events = Events::all(&state.pool).await.unwrap();

    let mut year_form_scores: HashMap<String, HashMap<String, i64>> = HashMap::new();
    for event in events.iter() {
        let year_id = event.year_id.clone();
        if let Ok(scores_map) =
            serde_json::from_str::<HashMap<String, String>>(event.scores.as_str())
        {
            let year_scores = year_form_scores.entry(year_id).or_insert_with(HashMap::new);
            for (form_id, score_str) in scores_map {
                if let Ok(score) = score_str.parse::<i64>() {
                    *year_scores.entry(form_id).or_insert(0) += score;
                }
            }
        }
    }

    // Calculate year totals (sum of all forms for each year)
    let mut year_totals: HashMap<String, i64> = HashMap::new();
    for (year_id, form_scores) in &year_form_scores {
        let total: i64 = form_scores.values().sum();
        year_totals.insert(year_id.clone(), total);
    }

    // Calculate form totals (sum of all years for each form)
    let mut form_totals: HashMap<String, i64> = HashMap::new();
    for form in &forms {
        let mut total: i64 = 0;
        for form_scores in year_form_scores.values() {
            if let Some(score) = form_scores.get(&form.id) {
                total += score;
            }
        }
        form_totals.insert(form.id.clone(), total);
    }

    // Calculate grand total
    let grand_total: i64 = form_totals.values().sum();

    let html = ScoreboardPartialTemplate {
        forms,
        years,
        scores: year_form_scores,
        year_totals,
        form_totals,
        grand_total,
    }
    .render()
    .expect("template should bee valid");
    html
}
