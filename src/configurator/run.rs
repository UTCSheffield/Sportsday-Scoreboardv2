use async_sqlite::Pool;
use log::{debug, info};

use crate::{
    configurator::build::Plan,
    db::{events::Events, years::Years},
};

pub async fn run(plan: Plan, pool: &Pool) -> Result<(), async_sqlite::Error> {
    info!("Implementing Plan");
    Events::delete_all(&pool).await.unwrap();
    Years::delete_all(&pool).await.unwrap();
    for year in plan.year_plans.iter() {
        debug!("Inserting Planned Year {}", year.id);
        let mut year_struct = Years::new(year.id.clone(), year.name.clone())
            .insert(&pool)
            .await?;
        for event in year.events.iter() {
            debug!("Inserting Planned Event {}", event.id);
            year_struct = year_struct
                .new_event(
                    &pool,
                    event.clone().id,
                    event.clone().name,
                    event.clone().gender_id,
                    event.clone().filter_key,
                    event.clone().scores,
                )
                .await?
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::configurator::parser::{ApplicabilityRules, Configuration, Event, Form, Year};
    use crate::test_harness;

    #[tokio::test]
    async fn test_run_empty_plan() {
        let db = test_harness::setup_db("run_empty_plan").await;

        let config = Configuration {
            version: "1.0.0".to_string(),
            genders: vec![],
            scores: vec![],
            years: vec![],
            forms: vec![],
            events: vec![],
        };

        let plan = crate::configurator::build::build_plan(config);
        let result = run(plan, &db).await;

        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_run_with_year() {
        let db = test_harness::setup_db("run_with_year").await;

        let config = Configuration {
            version: "1.0.0".to_string(),
            genders: vec!["mixed".to_string()],
            scores: vec![],
            years: vec![Year {
                id: "year7".to_string(),
                name: "Year 7".to_string(),
            }],
            forms: vec![],
            events: vec![],
        };

        let plan = crate::configurator::build::build_plan(config);
        let result = run(plan, &db).await;

        assert!(result.is_ok());

        let years = Years::all(&db).await.unwrap();
        assert_eq!(years.len(), 1);
        assert_eq!(years[0].id, "year7");
    }

    #[tokio::test]
    async fn test_run_with_year_and_events() {
        let db = test_harness::setup_db("run_with_year_and_events").await;

        let config = Configuration {
            version: "1.0.0".to_string(),
            genders: vec!["mixed".to_string()],
            scores: vec![],
            years: vec![Year {
                id: "year7".to_string(),
                name: "Year 7".to_string(),
            }],
            forms: vec![Form {
                id: "form1".to_string(),
                name: "Form 1".to_string(),
                colour: "#ff0000".to_string(),
            }],
            events: vec![Event {
                id: "event1".to_string(),
                name: "Event 1".to_string(),
                applicable_years: ApplicabilityRules::All,
                applicable_genders: ApplicabilityRules::All,
            }],
        };

        let plan = crate::configurator::build::build_plan(config);
        let result = run(plan, &db).await;

        assert!(result.is_ok());

        let years = Years::all(&db).await.unwrap();
        assert_eq!(years.len(), 1);

        let events = Events::all(&db).await.unwrap();
        assert_eq!(events.len(), 1);
        assert_eq!(events[0].name, "Event 1");
    }

    #[tokio::test]
    async fn test_run_deletes_existing_data() {
        let db = test_harness::setup_db("run_deletes_existing").await;

        // Insert some initial data
        Years::new("old_year".to_string(), "Old Year".to_string())
            .insert(&db)
            .await
            .unwrap();

        Events::new(
            "old-event".to_string(),
            "Old Event".to_string(),
            "old_year".to_string(),
            "mixed".to_string(),
            "old".to_string(),
            "{}".to_string(),
        )
        .insert(&db)
        .await
        .unwrap();

        // Verify data exists
        assert_eq!(Years::all(&db).await.unwrap().len(), 1);
        assert_eq!(Events::all(&db).await.unwrap().len(), 1);

        // Run with new config
        let config = Configuration {
            version: "1.0.0".to_string(),
            genders: vec!["mixed".to_string()],
            scores: vec![],
            years: vec![Year {
                id: "year7".to_string(),
                name: "Year 7".to_string(),
            }],
            forms: vec![],
            events: vec![],
        };

        let plan = crate::configurator::build::build_plan(config);
        let result = run(plan, &db).await;

        assert!(result.is_ok());

        // Verify old data is gone and new data is present
        let years = Years::all(&db).await.unwrap();
        assert_eq!(years.len(), 1);
        assert_eq!(years[0].id, "year7");

        let events = Events::all(&db).await.unwrap();
        assert_eq!(events.len(), 0);
    }

    #[tokio::test]
    async fn test_run_multiple_years_and_events() {
        let db = test_harness::setup_db("run_multiple").await;

        let config = Configuration {
            version: "1.0.0".to_string(),
            genders: vec!["boys".to_string(), "girls".to_string()],
            scores: vec![],
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
            forms: vec![Form {
                id: "form1".to_string(),
                name: "Form 1".to_string(),
                colour: "#ff0000".to_string(),
            }],
            events: vec![Event {
                id: "event1".to_string(),
                name: "Event 1".to_string(),
                applicable_years: ApplicabilityRules::All,
                applicable_genders: ApplicabilityRules::All,
            }],
        };

        let plan = crate::configurator::build::build_plan(config);
        let result = run(plan, &db).await;

        assert!(result.is_ok());

        let years = Years::all(&db).await.unwrap();
        assert_eq!(years.len(), 2);

        // Each year * each gender = 4 events
        let events = Events::all(&db).await.unwrap();
        assert_eq!(events.len(), 4);
    }

    // E2E test
    #[tokio::test]
    async fn test_e2e_configuration_rebuild() {
        use std::sync::atomic::{AtomicU64, Ordering};
        static COUNTER: AtomicU64 = AtomicU64::new(2000);
        let id = COUNTER.fetch_add(1, Ordering::SeqCst);
        let db_path = format!("./test/e2e_test_{}.db", id);
        std::fs::create_dir_all("./test").ok();

        let pool = async_sqlite::PoolBuilder::new()
            .path(&db_path)
            .open()
            .await
            .unwrap();

        crate::create_tables(&pool).await.unwrap();

        // Initial configuration
        let config1 = crate::configurator::parser::Configuration {
            version: "1.0.0".to_string(),
            genders: vec!["mixed".to_string()],
            scores: vec![],
            years: vec![crate::configurator::parser::Year {
                id: "year7".to_string(),
                name: "Year 7".to_string(),
            }],
            forms: vec![],
            events: vec![crate::configurator::parser::Event {
                id: "sprint".to_string(),
                name: "Sprint".to_string(),
                applicable_years: crate::configurator::parser::ApplicabilityRules::All,
                applicable_genders: crate::configurator::parser::ApplicabilityRules::All,
            }],
        };

        let plan1 = crate::configurator::build::build_plan(config1);
        run(plan1, &pool).await.unwrap();

        let events1 = Events::all(&pool).await.unwrap();
        assert_eq!(events1.len(), 1);

        // New configuration with more years and events
        let config2 = crate::configurator::parser::Configuration {
            version: "2.0.0".to_string(),
            genders: vec!["boys".to_string(), "girls".to_string()],
            scores: vec![],
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
            forms: vec![],
            events: vec![
                crate::configurator::parser::Event {
                    id: "sprint".to_string(),
                    name: "Sprint".to_string(),
                    applicable_years: crate::configurator::parser::ApplicabilityRules::All,
                    applicable_genders: crate::configurator::parser::ApplicabilityRules::All,
                },
                crate::configurator::parser::Event {
                    id: "relay".to_string(),
                    name: "Relay".to_string(),
                    applicable_years: crate::configurator::parser::ApplicabilityRules::All,
                    applicable_genders: crate::configurator::parser::ApplicabilityRules::All,
                },
            ],
        };

        let plan2 = crate::configurator::build::build_plan(config2);
        run(plan2, &pool).await.unwrap();

        let events2 = Events::all(&pool).await.unwrap();
        // 2 years * 2 genders * 2 events = 8 events
        assert_eq!(events2.len(), 8);

        let years2 = Years::all(&pool).await.unwrap();
        assert_eq!(years2.len(), 2);
    }
}
