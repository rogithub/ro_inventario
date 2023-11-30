
use sqlx::{sqlite::SqliteConnectOptions, SqlitePool};
use std::{path:: Path};

#[derive(Clone)]
pub struct AppState {
    pub db_pool: SqlitePool,
}

pub async fn connect(filename: impl AsRef<Path>) -> SqlitePool {
    let options = SqliteConnectOptions::new()
        .filename(filename);

    SqlitePool::connect_with(options).await.unwrap()
}