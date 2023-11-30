use std::cell::Cell;
use sqlx::{sqlite::SqliteConnectOptions, Error, SqlitePool};
use std::{future::Future};
use crate::settings::model::{Settings};

#[derive(Clone)]
pub struct AppState {
    settings: Cell<Settings>,
}
impl AppState {
    pub async fn connect(&self) -> impl Future<Output = Result<SqlitePool, Error>> {
        let options = SqliteConnectOptions::new()
            .filename(self.settings.get().cnn_str())
            .create_if_missing(true);
    
        SqlitePool::connect_with(options)
    }
}