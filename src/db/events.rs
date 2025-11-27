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
