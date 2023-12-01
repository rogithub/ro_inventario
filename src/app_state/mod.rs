
use sqlx::{sqlite::SqliteConnectOptions, SqlitePool};
use crate::settings::model::{Settings};


#[derive(Clone)]
pub struct AppState {
    pub settings: Settings,
}

impl AppState {
    pub async fn connect(&self) -> SqlitePool {    
        let options = SqliteConnectOptions::new()
            .filename(self.settings.cnn_str());

        SqlitePool::connect_with(options).await.unwrap()
    }
}