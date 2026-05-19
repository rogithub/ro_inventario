use axum_login::AuthUser;
use hmac::{Hmac, Mac};
use sha2::Sha512;
use sqlx::PgPool;
use uuid::Uuid;

// El tipo público del extractor de sesión — los handlers lo usan como `auth: AuthSession`
pub type AuthSession = axum_login::AuthSession<AuthBackend>;

// Lo que axum-login serializa en la sesión de cada usuario autenticado.
// Se guarda en la tabla de sesiones de PostgreSQL, así que conviene que sea pequeño.
#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
pub struct User {
    pub id: Uuid,
    pub email: String,
    pub is_active: bool,
    pub es_socio: bool,
}

impl AuthUser for User {
    type Id = Uuid;

    fn id(&self) -> Self::Id {
        self.id
    }

    // axum-login invalida la sesión cuando este valor cambia.
    // Usamos el email: si cambia el email, la sesión se invalida automáticamente.
    fn session_auth_hash(&self) -> &[u8] {
        self.email.as_bytes()
    }
}

// Credenciales que llegan del formulario de login (x-www-form-urlencoded)
#[derive(Clone, serde::Deserialize)]
pub struct Credentials {
    pub email: String,
    pub password: String,
}

// El backend que axum-login usa para autenticar y cargar usuarios.
// Necesita ser Clone + Send + Sync — PgPool ya cumple esas condiciones.
#[derive(Clone)]
pub struct AuthBackend {
    pool: PgPool,
}

impl AuthBackend {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

#[async_trait::async_trait]
impl axum_login::AuthnBackend for AuthBackend {
    type User = User;
    type Credentials = Credentials;
    type Error = crate::error::AppError;

    // Se llama al hacer POST /login. Devuelve Some(user) si las credenciales son válidas.
    async fn authenticate(
        &self,
        creds: Self::Credentials,
    ) -> Result<Option<Self::User>, Self::Error> {
        // Una sola query trae el usuario y su hash/salt
        let row = sqlx::query!(
            r#"SELECT id, email, isactive, essocio, passwordhash, passwordsalt
               FROM users WHERE email = $1"#,
            creds.email
        )
        .fetch_optional(&self.pool)
        .await?;

        let Some(row) = row else { return Ok(None) };

        // isactive no tiene NOT NULL en el esquema — puede ser NULL
        if !row.isactive.unwrap_or(false) {
            return Ok(None);
        }

        // El .NET usa HMACSHA512: la salt es la key del HMAC, el hash es HMAC(password_utf8)
        let password_ok = verify_hmac_sha512(&creds.password, &row.passwordhash, &row.passwordsalt);
        if !password_ok {
            return Ok(None);
        }

        Ok(Some(User {
            id: row.id,
            email: row.email,
            is_active: row.isactive.unwrap_or(false),
            es_socio: row.essocio,
        }))
    }

    // Se llama en cada request para cargar el usuario de la sesión activa.
    async fn get_user(
        &self,
        user_id: &axum_login::UserId<Self>,
    ) -> Result<Option<Self::User>, Self::Error> {
        let row = sqlx::query!(
            "SELECT id, email, isactive, essocio FROM users WHERE id = $1",
            user_id
        )
        .fetch_optional(&self.pool)
        .await?;

        Ok(row.map(|r| User {
            id: r.id,
            email: r.email,
            is_active: r.isactive.unwrap_or(false),
            es_socio: r.essocio,
        }))
    }
}

// Replica exacta del VerifyPasswordHash del .NET:
// `new HMACSHA512(passwordSalt)` → `ComputeHash(UTF8(password))` → comparar con stored hash
fn verify_hmac_sha512(password: &str, hash: &[u8], salt: &[u8]) -> bool {
    let Ok(mut mac) = Hmac::<Sha512>::new_from_slice(salt) else {
        return false;
    };
    mac.update(password.as_bytes());
    mac.verify_slice(hash).is_ok()
}
