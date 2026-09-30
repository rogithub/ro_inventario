use std::time::Duration;

use kernel::RepoError;
use sqlx::PgPool;
use usuarios::sesiones::{HuellaToken, SesionesRepo};
use usuarios::usuarios::Email;

#[derive(Clone)]
pub struct PgSesiones {
    pool: PgPool,
}

impl PgSesiones {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

fn seconds(duracion: Duration) -> i64 {
    i64::try_from(duracion.as_secs()).unwrap_or(i64::MAX)
}

fn repo_error(error: sqlx::Error) -> RepoError {
    RepoError(error.to_string())
}

impl SesionesRepo for PgSesiones {
    async fn create(&self, huella: &HuellaToken, email: &Email) -> Result<(), RepoError> {
        let insertadas = sqlx::query!(
            "INSERT INTO sesiones (huella, usuario_id) SELECT $1, id FROM usuarios WHERE email = $2",
            &huella.0[..],
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
        huella: &HuellaToken,
        duracion: Duration,
    ) -> Result<Option<Email>, RepoError> {
        // Encontrarla y renovarla en la misma consulta.
        let email = sqlx::query_scalar!(
            "UPDATE sesiones s SET last_used_at = now()
             FROM usuarios u
             WHERE s.huella = $1
               AND s.last_used_at > now() - ($2::bigint * interval '1 second')
               AND u.id = s.usuario_id
             RETURNING u.email",
            &huella.0[..],
            seconds(duracion)
        )
        .fetch_optional(&self.pool)
        .await
        .map_err(repo_error)?;
        email
            .map(|e| Email::parse(&e).map_err(|error| RepoError(error.to_string())))
            .transpose()
    }

    async fn delete(&self, huella: &HuellaToken) -> Result<(), RepoError> {
        sqlx::query!("DELETE FROM sesiones WHERE huella = $1", &huella.0[..])
            .execute(&self.pool)
            .await
            .map_err(repo_error)?;
        Ok(())
    }

    async fn record_failure(&self, email: &Email) -> Result<(), RepoError> {
        sqlx::query!(
            "INSERT INTO intentos_fallidos (email) VALUES ($1)",
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
        ventana: Duration,
    ) -> Result<u32, RepoError> {
        let total = sqlx::query_scalar!(
            r#"SELECT count(*) AS "total!" FROM intentos_fallidos
               WHERE email = $1 AND at > now() - ($2::bigint * interval '1 second')"#,
            email.as_str(),
            seconds(ventana)
        )
        .fetch_one(&self.pool)
        .await
        .map_err(repo_error)?;
        Ok(u32::try_from(total).unwrap_or(u32::MAX))
    }

    async fn clear_failures(&self, email: &Email) -> Result<(), RepoError> {
        sqlx::query!(
            "DELETE FROM intentos_fallidos WHERE email = $1",
            email.as_str()
        )
        .execute(&self.pool)
        .await
        .map_err(repo_error)?;
        Ok(())
    }

    async fn purge_expired(
        &self,
        duracion_sesion: Duration,
        ventana_fallos: Duration,
    ) -> Result<(), RepoError> {
        sqlx::query!(
            "DELETE FROM sesiones WHERE last_used_at <= now() - ($1::bigint * interval '1 second')",
            seconds(duracion_sesion)
        )
        .execute(&self.pool)
        .await
        .map_err(repo_error)?;
        sqlx::query!(
            "DELETE FROM intentos_fallidos WHERE at <= now() - ($1::bigint * interval '1 second')",
            seconds(ventana_fallos)
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
    use usuarios::sesiones::{BLOQUEO, DURACION_SESION, TokenSesion, contrato};
    use usuarios::usuarios::{NuevoUsuario, UsuariosRepo};

    #[sqlx::test]
    async fn cumple_el_contrato_sesion_creada(pool: PgPool) {
        contrato::sesion_creada_se_encuentra_y_borrada_ya_no(
            &PgUsuarios::new(pool.clone()),
            &PgSesiones::new(pool),
        )
        .await;
    }

    #[sqlx::test]
    async fn cumple_el_contrato_huella_desconocida(pool: PgPool) {
        contrato::huella_desconocida_no_se_encuentra(&PgSesiones::new(pool)).await;
    }

    #[sqlx::test]
    async fn cumple_el_contrato_fallos(pool: PgPool) {
        contrato::los_fallos_se_cuentan_por_email_y_se_borran(&PgSesiones::new(pool)).await;
    }

    #[sqlx::test]
    async fn cumple_el_contrato_purgar(pool: PgPool) {
        contrato::purgar_no_borra_lo_vigente(
            &PgUsuarios::new(pool.clone()),
            &PgSesiones::new(pool),
        )
        .await;
    }

    async fn sesion_de_ana(pool: &PgPool) -> (Email, HuellaToken) {
        PgUsuarios::new(pool.clone())
            .add(NuevoUsuario::new("ana@x.mx", "Ana", "Cajero", "caja-de-lapices").unwrap())
            .await
            .unwrap();
        let email = Email::parse("ana@x.mx").unwrap();
        let huella = TokenSesion::generate().unwrap().huella();
        PgSesiones::new(pool.clone())
            .create(&huella, &email)
            .await
            .unwrap();
        (email, huella)
    }

    #[sqlx::test]
    async fn una_sesion_sin_usar_7_dias_ya_no_sirve_y_se_purga(pool: PgPool) {
        let (_, huella) = sesion_de_ana(&pool).await;
        sqlx::query("UPDATE sesiones SET last_used_at = now() - interval '7 days 1 minute'")
            .execute(&pool)
            .await
            .unwrap();
        let sesiones = PgSesiones::new(pool.clone());

        assert_eq!(
            sesiones.find_email(&huella, DURACION_SESION).await.unwrap(),
            None
        );

        sesiones
            .purge_expired(DURACION_SESION, BLOQUEO)
            .await
            .unwrap();
        let quedan: i64 = sqlx::query_scalar("SELECT count(*) FROM sesiones")
            .fetch_one(&pool)
            .await
            .unwrap();
        assert_eq!(quedan, 0);
    }

    #[sqlx::test]
    async fn usar_la_sesion_la_renueva(pool: PgPool) {
        let (_, huella) = sesion_de_ana(&pool).await;
        sqlx::query("UPDATE sesiones SET last_used_at = now() - interval '6 days'")
            .execute(&pool)
            .await
            .unwrap();

        PgSesiones::new(pool.clone())
            .find_email(&huella, DURACION_SESION)
            .await
            .unwrap()
            .unwrap();

        let recien: bool =
            sqlx::query_scalar("SELECT last_used_at > now() - interval '1 minute' FROM sesiones")
                .fetch_one(&pool)
                .await
                .unwrap();
        assert!(recien);
    }

    /// Sin esto, alguien sin sesión podría llenar la tabla con emails inventados mientras nadie
    /// entre bien.
    #[sqlx::test]
    async fn un_intento_fallido_tambien_limpia_los_intentos_viejos(pool: PgPool) {
        let sesiones = PgSesiones::new(pool.clone());
        for _ in 0..3 {
            sesiones
                .record_failure(&Email::parse("viejo@x.mx").unwrap())
                .await
                .unwrap();
        }
        sqlx::query("UPDATE intentos_fallidos SET at = now() - interval '16 minutes'")
            .execute(&pool)
            .await
            .unwrap();

        let _ = usuarios::sesiones::login(
            &PgUsuarios::new(pool.clone()),
            &sesiones,
            "nuevo@x.mx",
            "no-es-la-contrasena",
        )
        .await;

        let quedan: Vec<String> = sqlx::query_scalar("SELECT email FROM intentos_fallidos")
            .fetch_all(&pool)
            .await
            .unwrap();
        assert_eq!(quedan, ["nuevo@x.mx"]);
    }

    #[sqlx::test]
    async fn los_fallos_de_hace_mas_de_15_minutos_no_cuentan(pool: PgPool) {
        let sesiones = PgSesiones::new(pool.clone());
        let ana = Email::parse("ana@x.mx").unwrap();
        for _ in 0..3 {
            sesiones.record_failure(&ana).await.unwrap();
        }
        sqlx::query("UPDATE intentos_fallidos SET at = now() - interval '16 minutes'")
            .execute(&pool)
            .await
            .unwrap();
        sesiones.record_failure(&ana).await.unwrap();

        assert_eq!(
            sesiones.count_recent_failures(&ana, BLOQUEO).await.unwrap(),
            1
        );
    }
}
