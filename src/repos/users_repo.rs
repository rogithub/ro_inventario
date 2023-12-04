use sqlx::{SqlitePool, Row};


#[derive(Debug)]
pub struct UserEntity {
    pub id:  String,
    pub email:  String,
    pub date_created: String,
    pub is_active: bool
}


impl UserEntity {
    pub async fn get_one(db_pool: SqlitePool, email: &str) -> Option<UserEntity> {            

        let mabe_row = sqlx::query("SELECT Id, Email, DateCreated, IsActive FROM Users WHERE Email=$1;")
        .bind(email)
        .fetch_optional(&db_pool)
        .await.ok()?;
        
        db_pool.close().await;
        
        let e = match mabe_row {
            Some(row) => {
                Some(UserEntity {
                    id: row.get("Id"),
                    email: row.get("Email"),
                    date_created: row.get("DateCreated"),
                    is_active: row.get("IsActive"),
                })
            },
            None => None
        };  
        
        e
    }

    pub async fn has_access(db_pool: SqlitePool, maybe_entity: &Option<UserEntity>, password: &str) -> bool {
        let access = match maybe_entity {
            Some(entity) => {
                if !entity.is_active {
                    return false;
                }

                let row = sqlx::query("SELECT PasswordHash, PasswordSalt FROM Users WHERE Id=$1;")
                .bind(entity.id.clone())
                .fetch_one(&db_pool)
                .await.ok();
                db_pool.close().await;

                true
            },
            None => false
        };
        
        access
    }
}