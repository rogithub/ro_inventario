use sqlx::{SqlitePool, Row};
use pbkdf2::pbkdf2_hmac;
use sha2::Sha512;

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

    pub fn verify_password_hash(password: &str, password_hash: &[u8], password_salt: &[u8]) -> bool {
        let password_bytes = password.as_bytes();
        let mut output = [0u8; 64]; // SHA512 produces a 64-byte hash
        pbkdf2_hmac::<Sha512>(password_bytes, password_salt, 10000, &mut output);
        output == password_hash
    }

    pub async fn has_access(db_pool: SqlitePool, maybe_entity: &Option<UserEntity>, password: &str) -> bool {
        let access = match maybe_entity {
            Some(entity) => {
                if !entity.is_active {
                    return false;
                }

                let maybe_row = sqlx::query("SELECT PasswordHash, PasswordSalt FROM Users WHERE Id=$1;")
                .bind(entity.id.clone())
                .fetch_one(&db_pool)
                .await.ok();
                db_pool.close().await;

                let result = match maybe_row {
                    Some(row) => {
                        let hash: &[u8] = row.get("PasswordHash");
                        let salt: &[u8] = row.get("PasswordSalt");
                    
                        Self::verify_password_hash(password, hash, salt)
                    },
                    None => false
                };
                
                result
            },
            None => false
        };
        
        access
    }
}