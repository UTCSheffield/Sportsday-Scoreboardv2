use async_sqlite::{rusqlite::Row, Pool};

use crate::db::events::Events;

#[derive(Clone, PartialEq, Debug)]
pub struct Years {
    pub id: String,
    pub name: String,
    events: Vec<Events>, // TODO: Events as optionals
}

impl Years {
    pub fn new(id: String, name: String) -> Self {
        Self {
            id,
            name,
            events: vec![],
        }
    }

    fn map_from_row(row: &Row) -> Result<Self, async_sqlite::Error> {
        Ok(Self {
            id: row.get(0)?,
            name: row.get(1)?,
            events: vec![],
        })
    }

    pub async fn insert(self, pool: &Pool) -> Result<Self, async_sqlite::Error> {
        let id = self.id.clone();
        let name = self.name.clone();
        pool.conn(move |conn| {
            conn.execute("INSERT INTO years(id, name) VALUES (?1, ?2);", [id, name])
                .unwrap();
            Ok(())
        })
        .await?;
        Ok(self)
    }

    pub async fn all(pool: &Pool) -> Result<Vec<Self>, async_sqlite::Error> {
        pool.conn(move |conn| {
            let mut stmt = conn.prepare("SELECT * FROM years")?;
            let year_iter = stmt
                .query_map([], |row| Ok(Self::map_from_row(row).unwrap()))
                .unwrap();
            let mut years = Vec::new();

            for year in year_iter {
                years.push(year?);
            }
            Ok(years)
        })
        .await
    }

    pub async fn new_event(
        mut self,
        pool: &Pool,
        id: String,
        name: String,
        gender_id: String,
        filter_key: String,
        scores: String,
    ) -> Result<Self, async_sqlite::Error> {
        let event = Events::new(id, name, self.clone().id, gender_id, filter_key, scores);
        self.events.push(event.clone());
        event.insert(&pool).await?;

        Ok(self)
    }

    pub async fn delete_all(pool: &Pool) -> Result<(), async_sqlite::Error> {
        pool.conn(move |conn| {
            conn.execute("DELETE FROM years;", []).unwrap();
            Ok(())
        })
        .await?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use crate::test_harness;

    use super::*;

    #[test]
    fn is_constructed_properly() {
        assert_eq!(
            Years::new("test-test".to_string(), "Test".to_string()),
            Years {
                id: "test-test".to_string(),
                name: "Test".to_string(),
                events: vec![]
            }
        )
    }

    #[tokio::test]
    async fn insert_test() {
        let db = test_harness::setup_db("years_insert").await;
        assert!(Years::new("test-test".to_string(), "Test".to_string())
            .insert(&db)
            .await
            .is_ok());
    }

    #[tokio::test]
    async fn all_test() {
        let db = test_harness::setup_db("years_all").await;
        assert!(Years::new("test-test".to_string(), "Test".to_string())
            .insert(&db)
            .await
            .is_ok());
        assert!(Years::new("test-test2".to_string(), "Test 2".to_string())
            .insert(&db)
            .await
            .is_ok());
        assert!(Years::new("test-test3".to_string(), "Test 3".to_string())
            .insert(&db)
            .await
            .is_ok());
        assert!(Years::new("test-test4".to_string(), "Test 4".to_string())
            .insert(&db)
            .await
            .is_ok());
        assert_eq!(Years::all(&db).await.unwrap().len(), 4);
    }

    #[tokio::test]
    async fn new_event_test() {
        let db = test_harness::setup_db("years_new_event").await;
        let mut obj = Years::new("test-test".to_string(), "Test".to_string());
        assert_eq!(
            obj,
            Years {
                id: "test-test".to_string(),
                name: "Test".to_string(),
                events: vec![]
            }
        );
        assert!(obj.clone().insert(&db).await.is_ok());
        let result = obj
            .new_event(
                &db,
                "test-test".to_string(),
                "Test Event".to_string(),
                "mixed".to_string(),
                "test-test".to_string(),
                "{}".to_string(),
            )
            .await;
        assert!(result.is_ok());
        obj = result.unwrap();
        assert_eq!(obj.events.len(), 1);
    }

    #[tokio::test]
    async fn delete_all_test() {
        let db = test_harness::setup_db("years_delete_all").await;
        assert!(Years::new("test-test".to_string(), "Test".to_string())
            .insert(&db)
            .await
            .is_ok());
        assert!(Years::new("test-test2".to_string(), "Test 2".to_string())
            .insert(&db)
            .await
            .is_ok());
        assert!(Years::new("test-test3".to_string(), "Test 3".to_string())
            .insert(&db)
            .await
            .is_ok());
        assert!(Years::new("test-test4".to_string(), "Test 4".to_string())
            .insert(&db)
            .await
            .is_ok());
        assert!(Years::delete_all(&db).await.is_ok());
    }
}
