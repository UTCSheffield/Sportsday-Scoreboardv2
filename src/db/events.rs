use async_sqlite::{rusqlite::Row, Pool};
use log::debug;
use serde_json::Value;

#[derive(Clone, PartialEq, Debug)]
pub struct Events {
    pub id: String,
    pub name: String,
    pub year_id: String,
    pub gender_id: String,
    pub filter_key: String,
    pub scores: String,
}

impl Events {
    pub fn new(
        id: String,
        name: String,
        year_id: String,
        gender_id: String,
        filter_key: String,
        scores: String,
    ) -> Self {
        Self {
            id,
            name,
            year_id,
            gender_id,
            filter_key,
            scores: scores,
        }
    }

    fn map_from_row(row: &Row) -> Result<Self, async_sqlite::Error> {
        Ok(Self {
            id: row.get(0)?,
            name: row.get(1)?,
            year_id: row.get(2)?,
            gender_id: row.get(3)?,
            filter_key: row.get(4)?,
            scores: row.get(5)?,
        })
    }

    pub async fn insert(self, pool: &Pool) -> Result<(), async_sqlite::Error> {
        pool.conn(move |conn| {
            debug!("Inserting Event with id {}", self.id);
            conn.execute(
                "INSERT INTO events(id, name, year_id, gender_id, filter_key, scores) VALUES (?1, ?2, ?3, ?4, ?5, ?6);",
                [self.id, self.name, self.year_id, self.gender_id, self.filter_key, self.scores],
            )
            .unwrap();
            Ok(())
        })
        .await?;
        Ok(())
    }

    pub async fn all(pool: &Pool) -> Result<Vec<Self>, async_sqlite::Error> {
        pool.conn(move |conn| {
            let mut stmt = conn.prepare("SELECT * FROM events")?;
            let event_iter = stmt
                .query_map([], |row| Ok(Self::map_from_row(row).unwrap()))
                .unwrap();
            let mut events = Vec::new();

            for event in event_iter {
                events.push(event?);
            }
            Ok(events)
        })
        .await
    }

    pub async fn r#where(
        pool: &Pool,
        year: Option<String>,
        activity: Option<String>,
        group: Option<String>,
    ) -> Result<Vec<Self>, async_sqlite::Error> {
        pool.conn(move |conn| {
            let mut stmt = conn.prepare("SELECT * FROM events")?;
            let event_iter = stmt
                .query_map([], |row| Ok(Self::map_from_row(row).unwrap()))
                .unwrap();
            let mut events = Vec::new();

            for event in event_iter {
                let evt = event?;
                if let Some(ref y) = year {
                    if &evt.year_id != y {
                        continue;
                    }
                }
                if let Some(ref a) = activity {
                    if &evt.filter_key != a {
                        continue;
                    }
                }
                if let Some(ref g) = group {
                    if &evt.gender_id != g {
                        continue;
                    }
                }
                events.push(evt);
            }
            Ok(events)
        })
        .await
    }

    pub async fn set_scores(
        pool: &Pool,
        id: String,
        scores: Value,
    ) -> Result<(), async_sqlite::Error> {
        pool.conn(move |conn| {
            debug!("Setting Scores for Event with id {}", id);
            conn.execute(
                "UPDATE events SET scores = ?1 WHERE id = ?2;",
                [serde_json::to_string(&scores).unwrap(), id],
            )
            .unwrap();
            Ok(())
        })
        .await?;
        Ok(())
    }

    pub async fn delete_all(pool: &Pool) -> Result<(), async_sqlite::Error> {
        pool.conn(move |conn| {
            conn.execute("DELETE FROM events;", []).unwrap();
            Ok(())
        })
        .await?;
        Ok(())
    }

    pub async fn count(pool: &Pool) -> Result<i64, async_sqlite::Error> {
        pool.conn(move |conn| {
            let count: i64 = conn.query_row("SELECT COUNT(*) FROM events", [], |row| row.get(0))?;
            Ok(count)
        })
        .await
    }
}

#[cfg(test)]
mod tests {
    use serde_json::json;

    use crate::{db::years::Years, test_harness};

    use super::*;

    #[test]
    fn is_constructed_properly() {
        assert_eq!(
            Events::new(
                "test-test".to_string(),
                "Test".to_string(),
                "test".to_string(),
                "mixed".to_string(),
                "test".to_string(),
                "{}".to_string()
            ),
            Events {
                id: "test-test".to_string(),
                name: "Test".to_string(),
                year_id: "test".to_string(),
                gender_id: "mixed".to_string(),
                filter_key: "test".to_string(),
                scores: "{}".to_string()
            }
        )
    }

    #[tokio::test]
    async fn insert_test() {
        let db = test_harness::setup_db("events_insert").await;
        assert!(Years::new("test".to_string(), "Test".to_string())
            .insert(&db)
            .await
            .is_ok());
        assert!(Events::new(
            "test-test".to_string(),
            "Test".to_string(),
            "test".to_string(),
            "mixed".to_string(),
            "test".to_string(),
            "{}".to_string()
        )
        .insert(&db)
        .await
        .is_ok());
    }

    #[tokio::test]
    async fn all_test() {
        let db = test_harness::setup_db("events_all").await;
        assert!(Years::new("test".to_string(), "Test".to_string())
            .insert(&db)
            .await
            .is_ok());
        assert!(Events::new(
            "test-test".to_string(),
            "Test".to_string(),
            "test".to_string(),
            "mixed".to_string(),
            "test".to_string(),
            "{}".to_string()
        )
        .insert(&db)
        .await
        .is_ok());
        assert!(Events::new(
            "test-test2".to_string(),
            "Test2".to_string(),
            "test".to_string(),
            "mixed".to_string(),
            "test".to_string(),
            "{}".to_string()
        )
        .insert(&db)
        .await
        .is_ok());
        assert!(Events::new(
            "test-test3".to_string(),
            "Test3".to_string(),
            "test".to_string(),
            "mixed".to_string(),
            "test".to_string(),
            "{}".to_string()
        )
        .insert(&db)
        .await
        .is_ok());
        assert!(Events::new(
            "test-test4".to_string(),
            "Test4".to_string(),
            "test".to_string(),
            "mixed".to_string(),
            "test".to_string(),
            "{}".to_string()
        )
        .insert(&db)
        .await
        .is_ok());
        assert_eq!(Events::all(&db).await.unwrap().len(), 4);
    }

    #[tokio::test]
    async fn where_test() {
        let db = test_harness::setup_db("events_where").await;
        for year_id in ["y9", "y10", "y11"].iter() {
            assert!(Years::new(year_id.to_string(), year_id.to_string())
                .insert(&db)
                .await
                .is_ok());
            assert!(Events::new(
                format!("{year_id}-test-test"),
                "Test".to_string(),
                year_id.to_string(),
                "boys".to_string(),
                "test".to_string(),
                "{}".to_string()
            )
            .insert(&db)
            .await
            .is_ok());
            assert!(Events::new(
                format!("{year_id}-test-test2"),
                "Test2".to_string(),
                year_id.to_string(),
                "girls".to_string(),
                "test".to_string(),
                "{}".to_string()
            )
            .insert(&db)
            .await
            .is_ok());
            assert!(Events::new(
                format!("{year_id}-test-test3"),
                "Test3".to_string(),
                year_id.to_string(),
                "mixed".to_string(),
                "test".to_string(),
                "{}".to_string()
            )
            .insert(&db)
            .await
            .is_ok());
            assert!(Events::new(
                format!("{year_id}-test-test4"),
                "Test4".to_string(),
                year_id.to_string(),
                "mixed".to_string(),
                "test".to_string(),
                "{}".to_string()
            )
            .insert(&db)
            .await
            .is_ok());
            assert_eq!(
                Events::r#where(&db, Some(year_id.to_string()), None, None)
                    .await
                    .unwrap()
                    .len(),
                4
            );
        }
        assert_eq!(
            Events::r#where(&db, None, None, None).await.unwrap().len(),
            12
        );
        assert_eq!(
            Events::r#where(&db, None, Some("test".to_string()), None)
                .await
                .unwrap()
                .len(),
            12
        );
        assert_eq!(
            Events::r#where(&db, None, None, Some("mixed".to_string()))
                .await
                .unwrap()
                .len(),
            6
        );
    }

    #[tokio::test]
    async fn set_scores_test() {
        let db = test_harness::setup_db("events_set_score").await;
        assert!(Years::new("test".to_string(), "Test".to_string())
            .insert(&db)
            .await
            .is_ok());
        assert!(Events::new(
            "test-test".to_string(),
            "Test".to_string(),
            "test".to_string(),
            "mixed".to_string(),
            "test".to_string(),
            "{}".to_string()
        )
        .insert(&db)
        .await
        .is_ok());

        assert!(Events::set_scores(
            &db,
            "test-test".to_string(),
            json!({
                "test": "test"
            })
        )
        .await
        .is_ok());
        assert_eq!(
            Events::all(&db).await.unwrap()[0].scores,
            json!({
                "test": "test"
            })
            .to_string()
        )
    }

    #[tokio::test]
    async fn delete_all_test() {
        let db = test_harness::setup_db("events_delete_all").await;
        assert!(Years::new("test".to_string(), "Test".to_string())
            .insert(&db)
            .await
            .is_ok());
        assert!(Events::new(
            "test-test".to_string(),
            "Test".to_string(),
            "test".to_string(),
            "mixed".to_string(),
            "test".to_string(),
            "{}".to_string()
        )
        .insert(&db)
        .await
        .is_ok());
        assert!(Events::new(
            "test-test2".to_string(),
            "Test2".to_string(),
            "test".to_string(),
            "mixed".to_string(),
            "test".to_string(),
            "{}".to_string()
        )
        .insert(&db)
        .await
        .is_ok());
        assert!(Events::new(
            "test-test3".to_string(),
            "Test3".to_string(),
            "test".to_string(),
            "mixed".to_string(),
            "test".to_string(),
            "{}".to_string()
        )
        .insert(&db)
        .await
        .is_ok());
        assert!(Events::new(
            "test-test4".to_string(),
            "Test4".to_string(),
            "test".to_string(),
            "mixed".to_string(),
            "test".to_string(),
            "{}".to_string()
        )
        .insert(&db)
        .await
        .is_ok());
        assert_eq!(Events::all(&db).await.unwrap().len(), 4);
        assert!(Events::delete_all(&db).await.is_ok());
        assert_eq!(Events::all(&db).await.unwrap().len(), 0);
    }

    #[tokio::test]
    async fn count_test() {
        let db = test_harness::setup_db("events_count").await;
        assert!(Years::new("test".to_string(), "Test".to_string())
            .insert(&db)
            .await
            .is_ok());

        assert_eq!(Events::count(&db).await.unwrap(), 0);

        assert!(Events::new(
            "test-test".to_string(),
            "Test".to_string(),
            "test".to_string(),
            "mixed".to_string(),
            "test".to_string(),
            "{}".to_string()
        )
        .insert(&db)
        .await
        .is_ok());

        assert_eq!(Events::count(&db).await.unwrap(), 1);

        assert!(Events::new(
            "test-test2".to_string(),
            "Test2".to_string(),
            "test".to_string(),
            "mixed".to_string(),
            "test".to_string(),
            "{}".to_string()
        )
        .insert(&db)
        .await
        .is_ok());

        assert_eq!(Events::count(&db).await.unwrap(), 2);
    }

    #[tokio::test]
    async fn where_with_multiple_filters_test() {
        let db = test_harness::setup_db("events_where_multiple").await;
        assert!(Years::new("y9".to_string(), "Year 9".to_string())
            .insert(&db)
            .await
            .is_ok());

        assert!(Events::new(
            "y9-boys-test".to_string(),
            "Test".to_string(),
            "y9".to_string(),
            "boys".to_string(),
            "test".to_string(),
            "{}".to_string()
        )
        .insert(&db)
        .await
        .is_ok());

        assert!(Events::new(
            "y9-girls-test".to_string(),
            "Test".to_string(),
            "y9".to_string(),
            "girls".to_string(),
            "test".to_string(),
            "{}".to_string()
        )
        .insert(&db)
        .await
        .is_ok());

        // Filter by year and group
        let events = Events::r#where(&db, Some("y9".to_string()), None, Some("boys".to_string()))
            .await
            .unwrap();

        assert_eq!(events.len(), 1);
        assert_eq!(events[0].id, "y9-boys-test");

        // Filter by year, activity and group
        let events = Events::r#where(
            &db,
            Some("y9".to_string()),
            Some("test".to_string()),
            Some("boys".to_string()),
        )
        .await
        .unwrap();

        assert_eq!(events.len(), 1);
        assert_eq!(events[0].id, "y9-boys-test");
    }

    // E2E tests
    #[actix_web::test]
    async fn test_e2e_event_filtering() {
        use std::sync::atomic::{AtomicU64, Ordering};
        static COUNTER: AtomicU64 = AtomicU64::new(5000);
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
            scores: vec![crate::configurator::parser::Score {
                name: "1st".to_string(),
                value: 10,
                default: true,
            }],
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

        // Test filtering by year
        let year7_events = Events::r#where(&pool, Some("year7".to_string()), None, None)
            .await
            .unwrap();
        assert_eq!(year7_events.len(), 3); // Only sprint events

        let year8_events = Events::r#where(&pool, Some("year8".to_string()), None, None)
            .await
            .unwrap();
        assert_eq!(year8_events.len(), 6); // Sprint + relay events

        // Test filtering by gender
        let boys_events = Events::r#where(&pool, None, None, Some("boys".to_string()))
            .await
            .unwrap();
        assert_eq!(boys_events.len(), 3); // boys events across all years

        // Test filtering by activity
        let sprint_events = Events::r#where(&pool, None, Some("sprint".to_string()), None)
            .await
            .unwrap();
        assert_eq!(sprint_events.len(), 6); // All sprint events

        let relay_events = Events::r#where(&pool, None, Some("relay".to_string()), None)
            .await
            .unwrap();
        assert_eq!(relay_events.len(), 3); // Only year8 relay events
    }

    #[actix_web::test]
    async fn test_e2e_score_updates() {
        use std::sync::atomic::{AtomicU64, Ordering};
        static COUNTER: AtomicU64 = AtomicU64::new(6000);
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
            genders: vec!["mixed".to_string()],
            scores: vec![],
            years: vec![crate::configurator::parser::Year {
                id: "year7".to_string(),
                name: "Year 7".to_string(),
            }],
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
            events: vec![crate::configurator::parser::Event {
                id: "sprint".to_string(),
                name: "Sprint".to_string(),
                applicable_years: crate::configurator::parser::ApplicabilityRules::All,
                applicable_genders: crate::configurator::parser::ApplicabilityRules::All,
            }],
        };

        let plan = crate::configurator::build::build_plan(config.clone());
        crate::configurator::run::run(plan, &pool).await.unwrap();

        // Get an event
        let events = Events::all(&pool).await.unwrap();
        let event = &events[0];

        // Update scores
        let new_scores = serde_json::json!({
            "form1": "10",
            "form2": "8"
        });

        Events::set_scores(&pool, event.id.clone(), new_scores.clone())
            .await
            .unwrap();

        // Verify scores were updated
        let updated_events = Events::all(&pool).await.unwrap();
        let updated_event = updated_events.iter().find(|e| e.id == event.id).unwrap();

        assert_eq!(updated_event.scores, new_scores.to_string());
    }
}
