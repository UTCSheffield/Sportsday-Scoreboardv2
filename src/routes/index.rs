use actix_web::{get, web, HttpResponse};
use askama::Template;

use crate::{templates::IndexTemplate, AppState};

#[get("/")]
pub async fn get(_state: web::Data<AppState>) -> HttpResponse {
    HttpResponse::Ok().body(IndexTemplate {}.render().expect("Template should be valid"))
}

#[cfg(test)]
mod tests {
    use super::*;
    use actix::Actor;
    use actix_web::{test, App};
    use std::sync::atomic::{AtomicU64, Ordering};

    // Helper function to create a unique test database path
    fn get_test_db_path(prefix: &str) -> String {
        static COUNTER: AtomicU64 = AtomicU64::new(8000);
        let id = COUNTER.fetch_add(1, Ordering::SeqCst);
        std::fs::create_dir_all("./test").ok();
        let path = format!("./test/{}_{}.db", prefix, id);
        std::fs::remove_file(&path).ok();
        path
    }

    #[actix_web::test]
    async fn test_index_route() {
        let config = crate::configurator::parser::Configuration {
            version: "1.0.0".to_string(),
            genders: vec![],
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
            .path(&get_test_db_path("index_route"))
            .open()
            .await
            .unwrap();

        crate::create_tables(&pool).await.unwrap();

        let log_collector = crate::logger::LogCollector::new(1000);

        let app = test::init_service(
            App::new()
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

        let req = test::TestRequest::get().uri("/").to_request();
        let resp = test::call_service(&app, req).await;

        assert!(resp.status().is_success());
    }

    #[actix_web::test]
    async fn test_e2e_full_workflow() {
        use std::sync::atomic::{AtomicU64, Ordering};
        static COUNTER: AtomicU64 = AtomicU64::new(7000);
        let id = COUNTER.fetch_add(1, Ordering::SeqCst);
        let db_path = format!("./test/e2e_test_{}.db", id);
        std::fs::create_dir_all("./test").ok();

        let pool = async_sqlite::PoolBuilder::new()
            .path(&db_path)
            .open()
            .await
            .unwrap();

        crate::create_tables(&pool).await.unwrap();

        let config = crate::configurator::parser::Configuration {
            version: "1.0.0".to_string(),
            genders: vec!["boys".to_string(), "girls".to_string(), "mixed".to_string()],
            scores: vec![
                crate::configurator::parser::Score {
                    name: "1st".to_string(),
                    value: 10,
                    default: true,
                },
                crate::configurator::parser::Score {
                    name: "2nd".to_string(),
                    value: 8,
                    default: false,
                },
            ],
            years: vec![
                crate::configurator::parser::Year {
                    id: "year7".to_string(),
                    name: "Year 7".to_string(),
                },
                crate::configurator::parser::Year {
                    id: "year8".to_string(),
                    name: "Year 8".to_string(),
                },
            ],
            forms: vec![
                crate::configurator::parser::Form {
                    id: "form1".to_string(),
                    name: "Form 1".to_string(),
                    colour: "#ff0000".to_string(),
                },
                crate::configurator::parser::Form {
                    id: "form2".to_string(),
                    name: "Form 2".to_string(),
                    colour: "#00ff00".to_string(),
                },
            ],
            events: vec![
                crate::configurator::parser::Event {
                    id: "sprint".to_string(),
                    name: "100m Sprint".to_string(),
                    applicable_years: crate::configurator::parser::ApplicabilityRules::All,
                    applicable_genders: crate::configurator::parser::ApplicabilityRules::All,
                },
                crate::configurator::parser::Event {
                    id: "relay".to_string(),
                    name: "4x100m Relay".to_string(),
                    applicable_years: crate::configurator::parser::ApplicabilityRules::Include {
                        ids: vec!["year8".to_string()],
                    },
                    applicable_genders: crate::configurator::parser::ApplicabilityRules::All,
                },
            ],
        };

        let plan = crate::configurator::build::build_plan(config.clone());
        crate::configurator::run::run(plan, &pool).await.unwrap();

        let client = reqwest::Client::builder()
            .user_agent("SportsDayScore")
            .build()
            .unwrap();

        let log_collector = crate::logger::LogCollector::new(1000);
        let ws_channels = crate::websocket::ChannelsActor::new().start();

        let app = test::init_service(
            App::new()
                .wrap(crate::middleware::headers::DefaultHtmlContentType)
                .app_data(web::Data::new(crate::AppState {
                    client: client.clone(),
                    config: config.clone(),
                    pool: pool.clone(),
                    log_collector: log_collector.clone(),
                    oauth_creds: crate::OauthCreds {
                        client_id: "test_client_id".to_string(),
                        client_secret: "test_client_secret".to_string(),
                    },
                }))
                .app_data(web::Data::new(ws_channels.clone()))
                .service(get)
                .service(crate::routes::scoreboard::get)
                .service(crate::routes::results::get)
                .service(crate::routes::ws::get),
        )
        .await;

        // Test index page loads
        let req = test::TestRequest::get().uri("/").to_request();
        let resp = test::call_service(&app, req).await;
        assert!(resp.status().is_success());

        // Test scoreboard page loads
        let req = test::TestRequest::get().uri("/scoreboard").to_request();
        let resp = test::call_service(&app, req).await;
        assert!(resp.status().is_success());

        // Test results page loads
        let req = test::TestRequest::get().uri("/results").to_request();
        let resp = test::call_service(&app, req).await;
        assert!(resp.status().is_success());

        // Verify database was populated correctly
        let years = crate::db::years::Years::all(&pool).await.unwrap();
        assert_eq!(years.len(), 2);

        let events = crate::db::events::Events::all(&pool).await.unwrap();
        // year7: sprint (boys, girls, mixed) = 3
        // year8: sprint (boys, girls, mixed) + relay (boys, girls, mixed) = 6
        // Total: 9 events
        assert_eq!(events.len(), 9);
    }
}
