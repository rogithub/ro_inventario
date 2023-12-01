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
}