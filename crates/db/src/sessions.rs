use std::time::Duration;

use kernel::RepoError;
use sqlx::PgPool;
use usuarios::sessions::{SessionsRepo, TokenHash};
use usuarios::usuarios::Email;

#[derive(Clone)]
pub struct PgSessions {
    pool: PgPool,
}

impl PgSessions {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

fn seconds(duration: Duration) -> i64 {
    i64::try_from(duration.as_secs()).unwrap_or(i64::MAX)
}

fn repo_error(error: sqlx::Error) -> RepoError {
    RepoError(error.to_string())
}

impl SessionsRepo for PgSessions {
    async fn create(&self, token_hash: &TokenHash, email: &Email) -> Result<(), RepoError> {
        let insertadas = sqlx::query!(
            "INSERT INTO sessions (token_hash, usuario_id) SELECT $1, id FROM usuarios WHERE email = $2",
            &token_hash.0[..],
            email.as_str()
        )
        .execute(&self.pool)
        .await
        .map_err(repo_error)?
        .rows_affected();
        if insertadas == 1 {
            Ok(())
        } else {
            Err(RepoError(format!(
                "no existe el usuario {}",
                email.as_str()
            )))
        }
    }

    async fn find_email(
        &self,
        token_hash: &TokenHash,
        duration: Duration,
    ) -> Result<Option<Email>, RepoError> {
        // Encontrarla y renovarla en la misma consulta.
        let email = sqlx::query_scalar!(
            "UPDATE sessions s SET last_used_at = now()
             FROM usuarios u
             WHERE s.token_hash = $1
               AND s.last_used_at > now() - ($2::bigint * interval '1 second')
               AND u.id = s.usuario_id
             RETURNING u.email",
            &token_hash.0[..],
            seconds(duration)
        )
        .fetch_optional(&self.pool)
        .await
        .map_err(repo_error)?;
        email
            .map(|e| Email::parse(&e).map_err(|error| RepoError(error.to_string())))
            .transpose()
    }

    async fn delete(&self, token_hash: &TokenHash) -> Result<(), RepoError> {
        sqlx::query!(
            "DELETE FROM sessions WHERE token_hash = $1",
            &token_hash.0[..]
        )
        .execute(&self.pool)
        .await
        .map_err(repo_error)?;
        Ok(())
    }

    async fn record_failure(&self, email: &Email) -> Result<(), RepoError> {
        sqlx::query!(
            "INSERT INTO failed_logins (email) VALUES ($1)",
            email.as_str()
        )
        .execute(&self.pool)
        .await
        .map_err(repo_error)?;
        Ok(())
    }

    async fn count_recent_failures(
        &self,
        email: &Email,
        window: Duration,
    ) -> Result<u32, RepoError> {
        let total = sqlx::query_scalar!(
            r#"SELECT count(*) AS "total!" FROM failed_logins
               WHERE email = $1 AND at > now() - ($2::bigint * interval '1 second')"#,
            email.as_str(),
            seconds(window)
        )
        .fetch_one(&self.pool)
        .await
        .map_err(repo_error)?;
        Ok(u32::try_from(total).unwrap_or(u32::MAX))
    }

    async fn clear_failures(&self, email: &Email) -> Result<(), RepoError> {
        sqlx::query!("DELETE FROM failed_logins WHERE email = $1", email.as_str())
            .execute(&self.pool)
            .await
            .map_err(repo_error)?;
        Ok(())
    }

    async fn purge_expired(
        &self,
        session_duration: Duration,
        failures_window: Duration,
    ) -> Result<(), RepoError> {
        sqlx::query!(
            "DELETE FROM sessions WHERE last_used_at <= now() - ($1::bigint * interval '1 second')",
            seconds(session_duration)
        )
        .execute(&self.pool)
        .await
        .map_err(repo_error)?;
        sqlx::query!(
            "DELETE FROM failed_logins WHERE at <= now() - ($1::bigint * interval '1 second')",
            seconds(failures_window)
        )
        .execute(&self.pool)
        .await
        .map_err(repo_error)?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::PgUsuarios;
    use usuarios::sessions::{LOCKOUT, SESSION_DURATION, SessionToken, contrato};
    use usuarios::usuarios::{NuevoUsuario, UsuariosRepo};

    #[sqlx::test]
    async fn cumple_el_contrato_sesion_creada(pool: PgPool) {
        contrato::sesion_creada_se_encuentra_y_borrada_ya_no(
            &PgUsuarios::new(pool.clone()),
            &PgSessions::new(pool),
        )
        .await;
    }

    #[sqlx::test]
    async fn cumple_el_contrato_huella_desconocida(pool: PgPool) {
        contrato::huella_desconocida_no_se_encuentra(&PgSessions::new(pool)).await;
    }

    #[sqlx::test]
    async fn cumple_el_contrato_fallos(pool: PgPool) {
        contrato::los_fallos_se_cuentan_por_email_y_se_borran(&PgSessions::new(pool)).await;
    }

    #[sqlx::test]
    async fn cumple_el_contrato_purgar(pool: PgPool) {
        contrato::purgar_no_borra_lo_vigente(
            &PgUsuarios::new(pool.clone()),
            &PgSessions::new(pool),
        )
        .await;
    }

    async fn sesion_de_ana(pool: &PgPool) -> (Email, TokenHash) {
        PgUsuarios::new(pool.clone())
            .add(NuevoUsuario::new("ana@x.mx", "Ana", "Cajero", "caja-de-lapices").unwrap())
            .await
            .unwrap();
        let email = Email::parse("ana@x.mx").unwrap();
        let token_hash = SessionToken::generate().unwrap().token_hash();
        PgSessions::new(pool.clone())
            .create(&token_hash, &email)
            .await
            .unwrap();
        (email, token_hash)
    }

    #[sqlx::test]
    async fn una_sesion_sin_usar_7_dias_ya_no_sirve_y_se_purga(pool: PgPool) {
        let (_, token_hash) = sesion_de_ana(&pool).await;
        sqlx::query("UPDATE sessions SET last_used_at = now() - interval '7 days 1 minute'")
            .execute(&pool)
            .await
            .unwrap();
        let sessions = PgSessions::new(pool.clone());

        assert_eq!(
            sessions
                .find_email(&token_hash, SESSION_DURATION)
                .await
                .unwrap(),
            None
        );

        sessions
            .purge_expired(SESSION_DURATION, LOCKOUT)
            .await
            .unwrap();
        let quedan: i64 = sqlx::query_scalar("SELECT count(*) FROM sessions")
            .fetch_one(&pool)
            .await
            .unwrap();
        assert_eq!(quedan, 0);
    }

    #[sqlx::test]
    async fn usar_la_sesion_la_renueva(pool: PgPool) {
        let (_, token_hash) = sesion_de_ana(&pool).await;
        sqlx::query("UPDATE sessions SET last_used_at = now() - interval '6 days'")
            .execute(&pool)
            .await
            .unwrap();

        PgSessions::new(pool.clone())
            .find_email(&token_hash, SESSION_DURATION)
            .await
            .unwrap()
            .unwrap();

        let recien: bool =
            sqlx::query_scalar("SELECT last_used_at > now() - interval '1 minute' FROM sessions")
                .fetch_one(&pool)
                .await
                .unwrap();
        assert!(recien);
    }

    /// Sin esto, alguien sin sesión podría llenar la tabla con emails inventados mientras nadie
    /// entre bien.
    #[sqlx::test]
    async fn un_intento_fallido_tambien_limpia_los_intentos_viejos(pool: PgPool) {
        let sessions = PgSessions::new(pool.clone());
        for _ in 0..3 {
            sessions
                .record_failure(&Email::parse("viejo@x.mx").unwrap())
                .await
                .unwrap();
        }
        sqlx::query("UPDATE failed_logins SET at = now() - interval '16 minutes'")
            .execute(&pool)
            .await
            .unwrap();

        let _ = usuarios::sessions::login(
            &PgUsuarios::new(pool.clone()),
            &sessions,
            "nuevo@x.mx",
            "no-es-la-password",
        )
        .await;

        let quedan: Vec<String> = sqlx::query_scalar("SELECT email FROM failed_logins")
            .fetch_all(&pool)
            .await
            .unwrap();
        assert_eq!(quedan, ["nuevo@x.mx"]);
    }

    #[sqlx::test]
    async fn los_fallos_de_hace_mas_de_15_minutos_no_cuentan(pool: PgPool) {
        let sessions = PgSessions::new(pool.clone());
        let ana = Email::parse("ana@x.mx").unwrap();
        for _ in 0..3 {
            sessions.record_failure(&ana).await.unwrap();
        }
        sqlx::query("UPDATE failed_logins SET at = now() - interval '16 minutes'")
            .execute(&pool)
            .await
            .unwrap();
        sessions.record_failure(&ana).await.unwrap();

        assert_eq!(
            sessions.count_recent_failures(&ana, LOCKOUT).await.unwrap(),
            1
        );
    }
}
