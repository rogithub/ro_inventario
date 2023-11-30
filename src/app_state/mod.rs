
use sqlx::{sqlite::SqliteConnectOptions, Error, SqlitePool};
use std::{future::Future, path:: Path};

pub async fn connect(filename: impl AsRef<Path>) -> impl Future<Output = Result<SqlitePool, Error>> {
    let options = SqliteConnectOptions::new()
        .filename(filename)
        .create_if_missing(true);

    SqlitePool::connect_with(options)
}