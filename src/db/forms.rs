use async_sqlite::Pool;

#[derive(Clone)]
pub struct Forms {
    pub id: String,
    pub name: String,
}

impl Forms {
    pub fn new(id: String, name: String) -> Self {
        Self { id, name }
    }

    pub async fn insert(self, pool: &Pool) -> Result<(), async_sqlite::Error> {
        pool.conn(move |conn| {
            conn.execute(
                "INSERT INTO forms(id, name) VALUES (?1, ?2);",
                [self.id, self.name],
            )
            .unwrap();
            Ok(())
        })
        .await?;
        Ok(())
    }
}
