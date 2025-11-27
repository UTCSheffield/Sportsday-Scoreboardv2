use tokio::fs;

use async_sqlite::{Pool, PoolBuilder};

use crate::db;

pub async fn setup_db(db_name: &str) -> Pool {
    fs::remove_file(format!("./test/{db_name}.db").as_str())
        .await
        .unwrap_or_default();
    let pool = PoolBuilder::new()
        .path(format!("./test/{db_name}.db").as_str())
        .open()
        .await
        .unwrap();
    db::create_tables(&pool).await.unwrap();
    pool
}
