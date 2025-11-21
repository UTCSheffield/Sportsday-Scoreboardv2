use async_sqlite::Pool;

pub mod events;
pub mod years;

pub async fn create_tables(pool: &Pool) -> Result<(), async_sqlite::Error> {
    pool.conn(move |conn| {
        conn.execute("PRAGMA foreign_keys = ON", []).unwrap();

        conn.execute(
            "CREATE TABLE IF NOT EXISTS years (
                id TEXT PRIMARY KEY,
                name TEXT NOT NULL
            );",
            [],
        )
        .unwrap();

        conn.execute(
            "CREATE TABLE IF NOT EXISTS events (
                id TEXT PRIMARY KEY,
                name TEXT NOT NULL,
                year_id TEXT NOT NULL,
                gender_id TEXT NOT NULL,
                filter_key TEXT NOT NULL,
                scores TEXT NOT NULL DEFAULT '{}',
                FOREIGN KEY (year_id) REFERENCES years(id)
            );",
            [],
        )
        .unwrap();

        Ok(())
    })
    .await?;
    Ok(())
}
