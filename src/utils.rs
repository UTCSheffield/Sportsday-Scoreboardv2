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

#[macro_export]
macro_rules! ternary {
    ($condition: expr => $true_expr: expr , $false_expr: expr) => {
        if $condition {
            $true_expr
        } else {
            $false_expr
        }
    };
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::configurator::parser::{ApplicabilityRules, Configuration, Event, Form, Year};
    use crate::test_harness;

    #[test]
    fn test_ternary_macro_true() {
        let result = ternary!(true => "yes", "no");
        assert_eq!(result, "yes");
    }

    #[test]
    fn test_ternary_macro_false() {
        let result = ternary!(false => "yes", "no");
        assert_eq!(result, "no");
    }

    #[test]
    fn test_ternary_macro_with_numbers() {
        let result = ternary!(5 > 3 => 1, 0);
        assert_eq!(result, 1);

        let result = ternary!(5 < 3 => 1, 0);
        assert_eq!(result, 0);
    }

    #[tokio::test]
    async fn test_render_scoreboard_empty() {
        let db = test_harness::setup_db("utils_render_scoreboard_empty").await;

        let config = Configuration {
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

        let log_collector = crate::logger::LogCollector::new(1000);

        let state = web::Data::new(crate::AppState {
            client,
            config,
            pool: db,
            log_collector,
            oauth_creds: crate::OauthCreds {
                client_id: "test".to_string(),
                client_secret: "test".to_string(),
            },
        });

        let html = render_scoreboard(state).await;
        assert!(!html.is_empty());
    }

    #[tokio::test]
    async fn test_render_scoreboard_with_data() {
        let db = test_harness::setup_db("utils_render_scoreboard_with_data").await;

        // Create test data
        use crate::db::events::Events;
        use crate::db::years::Years;

        let year = Years::new("2024".to_string(), "Year 2024".to_string());
        year.clone().insert(&db).await.unwrap();

        let event = Events::new(
            "2024-mixed-event1".to_string(),
            "Event 1".to_string(),
            "2024".to_string(),
            "mixed".to_string(),
            "event1".to_string(),
            r#"{"form1":"10","form2":"20"}"#.to_string(),
        );
        event.insert(&db).await.unwrap();

        let config = Configuration {
            version: "1.0.0".to_string(),
            genders: vec!["mixed".to_string()],
            scores: vec![],
            years: vec![Year {
                id: "2024".to_string(),
                name: "Year 2024".to_string(),
            }],
            forms: vec![
                Form {
                    id: "form1".to_string(),
                    name: "Form 1".to_string(),
                    colour: "#ff0000".to_string(),
                },
                Form {
                    id: "form2".to_string(),
                    name: "Form 2".to_string(),
                    colour: "#00ff00".to_string(),
                },
            ],
            events: vec![Event {
                id: "event1".to_string(),
                name: "Event 1".to_string(),
                applicable_years: ApplicabilityRules::All,
                applicable_genders: ApplicabilityRules::All,
            }],
        };

        let client = reqwest::Client::builder()
            .user_agent("SportsDayScore")
            .build()
            .unwrap();

        let log_collector = crate::logger::LogCollector::new(1000);

        let state = web::Data::new(crate::AppState {
            client,
            config,
            pool: db,
            log_collector,
            oauth_creds: crate::OauthCreds {
                client_id: "test".to_string(),
                client_secret: "test".to_string(),
            },
        });

        let html = render_scoreboard(state).await;
        assert!(!html.is_empty());
        // The HTML should contain some form data
        assert!(html.len() > 100);
    }

    // E2E test
    #[actix_web::test]
    async fn test_e2e_complete_scoreboard_calculation() {
        use std::sync::atomic::{AtomicU64, Ordering};
        static COUNTER: AtomicU64 = AtomicU64::new(4000);
        let id = COUNTER.fetch_add(1, Ordering::SeqCst);
        let db_path = format!("./test/e2e_test_{}.db", id);
        std::fs::create_dir_all("./test").ok();

        let pool = async_sqlite::PoolBuilder::new()
            .path(&db_path)
            .open()
            .await
            .unwrap();

        crate::create_tables(&pool).await.unwrap();

        let config = Configuration {
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
                Year {
                    id: "year7".to_string(),
                    name: "Year 7".to_string(),
                },
                Year {
                    id: "year8".to_string(),
                    name: "Year 8".to_string(),
                },
            ],
            forms: vec![
                Form {
                    id: "form1".to_string(),
                    name: "Form 1".to_string(),
                    colour: "#ff0000".to_string(),
                },
                Form {
                    id: "form2".to_string(),
                    name: "Form 2".to_string(),
                    colour: "#00ff00".to_string(),
                },
            ],
            events: vec![
                Event {
                    id: "sprint".to_string(),
                    name: "100m Sprint".to_string(),
                    applicable_years: ApplicabilityRules::All,
                    applicable_genders: ApplicabilityRules::All,
                },
                Event {
                    id: "relay".to_string(),
                    name: "4x100m Relay".to_string(),
                    applicable_years: ApplicabilityRules::Include {
                        ids: vec!["year8".to_string()],
                    },
                    applicable_genders: ApplicabilityRules::All,
                },
            ],
        };

        let plan = crate::configurator::build::build_plan(config.clone());
        crate::configurator::run::run(plan, &pool).await.unwrap();

        // Set scores for multiple events
        let events = crate::db::events::Events::all(&pool).await.unwrap();

        for (i, event) in events.iter().enumerate() {
            let scores = if i % 2 == 0 {
                serde_json::json!({
                    "form1": "10",
                    "form2": "8"
                })
            } else {
                serde_json::json!({
                    "form1": "8",
                    "form2": "10"
                })
            };

            crate::db::events::Events::set_scores(&pool, event.id.clone(), scores)
                .await
                .unwrap();
        }

        // Verify all events have scores
        let scored_events = crate::db::events::Events::all(&pool).await.unwrap();
        for event in scored_events.iter() {
            assert!(event.scores.contains("form1"));
            assert!(event.scores.contains("form2"));
        }

        // Test the actual scoreboard rendering
        let client = reqwest::Client::builder()
            .user_agent("SportsDayScore")
            .build()
            .unwrap();

        let log_collector = crate::logger::LogCollector::new(1000);

        let state = web::Data::new(crate::AppState {
            client,
            config,
            pool: pool.clone(),
            log_collector,
            oauth_creds: crate::OauthCreds {
                client_id: "test".to_string(),
                client_secret: "test".to_string(),
            },
        });

        let html = render_scoreboard(state).await;
        assert!(!html.is_empty());
        assert!(scored_events.len() > 0);
    }
}
