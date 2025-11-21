use async_sqlite::{rusqlite::Row, Pool};

use crate::db::events::Events;

#[derive(Clone)]
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
