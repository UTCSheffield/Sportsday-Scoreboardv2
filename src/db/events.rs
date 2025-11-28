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
}
