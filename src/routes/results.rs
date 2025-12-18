use actix_web::{get, web, HttpResponse};
use askama::Template;
use serde_json::Value;

use crate::{configurator::parser::Year, db, templates::ResultsTemplate, AppState};

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

#[cfg(test)]
mod tests {
    use super::*;
    use actix_web::test;
    use std::sync::atomic::{AtomicU64, Ordering};

    fn get_test_db_path(prefix: &str) -> String {
        static COUNTER: AtomicU64 = AtomicU64::new(10000);
        let id = COUNTER.fetch_add(1, Ordering::SeqCst);
        std::fs::create_dir_all("./test").ok();
        let path = format!("./test/{}_{}.db", prefix, id);
        std::fs::remove_file(&path).ok();
        path
    }

    #[actix_web::test]
    async fn test_results_route() {
        let config = crate::configurator::parser::Configuration {
            version: "1.0.0".to_string(),
            genders: vec!["mixed".to_string()],
            scores: vec![],
            years: vec![],
            forms: vec![],
            events: vec![],
        };

        let client = reqwest::Client::builder()
            .user_agent("SportsDayScore")
            .build()
            .unwrap();

        let pool = async_sqlite::PoolBuilder::new()
            .path(&get_test_db_path("results_route"))
            .open()
            .await
            .unwrap();

        crate::create_tables(&pool).await.unwrap();

        let log_collector = crate::logger::LogCollector::new(1000);

        let app = test::init_service(
            actix_web::App::new()
                .app_data(web::Data::new(crate::AppState {
                    client: client.clone(),
                    config: config.clone(),
                    pool: pool.clone(),
                    log_collector: log_collector.clone(),
                    oauth_creds: crate::OauthCreds {
                        client_id: "test".to_string(),
                        client_secret: "test".to_string(),
                    },
                }))
                .service(get),
        )
        .await;

        let req = test::TestRequest::get().uri("/results").to_request();
        let resp = test::call_service(&app, req).await;

        assert!(resp.status().is_success());
    }
}
