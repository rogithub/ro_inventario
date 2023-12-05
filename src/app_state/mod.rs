
use sqlx::{SqlitePool};
use crate::settings::model::{Settings};

#[derive(Clone)]
pub struct AppState {
    pub settings: Settings,
}

impl AppState {
    pub async fn connect(&self) -> SqlitePool {
        SqlitePool::connect(&self.settings.db_url()).await.unwrap()
    }
}