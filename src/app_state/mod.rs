
use sqlx::{SqlitePool};
use crate::settings::model::{Settings};
use actix_identity::{Identity};
use log;

#[derive(Debug)]
pub enum IdentityError {
    MissingId,
    InvalidId(String),
}

#[derive(Clone)]
pub struct AppState {
    pub settings: Settings,
}

#[derive(Debug, Default)]
pub struct IdenityClaims {
    is_authenticated: bool,
    email: String
}

impl AppState {
    pub async fn connect(&self) -> SqlitePool {
        SqlitePool::connect(&self.settings.db_url()).await.unwrap()
    }

    pub fn identity_check(identity: Option<Identity>) -> Result<IdenityClaims, IdentityError> {
        match identity.map(|id| id.id()) {
            None => {
                log::warn!("Missing user identity");
                Ok(IdenityClaims { is_authenticated: false, email: "".to_string() })
            },
            Some(Ok(id)) => Ok(IdenityClaims { is_authenticated: true, email: id }),
            Some(Err(err)) => {
                log::error!("Invalid user identity: {}", err.to_string());
                Err(IdentityError::InvalidId(err.to_string()))
            },
        }
    }    
}