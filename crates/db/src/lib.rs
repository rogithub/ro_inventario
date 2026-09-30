//! Implementación en PostgreSQL de los repositorios que definen las áreas del negocio, y las migraciones.
//!
//! Depende de las áreas, nunca al revés: docs/decisiones/2026-09-29-estructura-de-crates.md
//! Migraciones: docs/decisiones/2026-09-29-base-de-datos-y-ambientes.md

pub use sqlx::PgPool;

/// Las migraciones de `crates/db/migrations/`, incluidas en el binario al compilar.
pub static MIGRATOR: sqlx::migrate::Migrator = sqlx::migrate!();

/// Abre el pool de conexiones. Se conecta una vez para fallar al arrancar si la base no está,
/// en vez de a la primera venta.
pub async fn connect(database_url: &str) -> Result<PgPool, sqlx::Error> {
    sqlx::postgres::PgPoolOptions::new()
        .acquire_timeout(std::time::Duration::from_secs(5))
        .connect(database_url)
        .await
}

/// Aplica las migraciones pendientes. Cada una corre en su propia transacción;
/// las ya aplicadas no se repiten (la base registra cuáles tiene).
pub async fn run_migrations(pool: &PgPool) -> Result<(), sqlx::migrate::MigrateError> {
    MIGRATOR.run(pool).await
}

/// `true` si la base responde. Para `/health`.
pub async fn is_db_alive(pool: &PgPool) -> bool {
    sqlx::query("SELECT 1").execute(pool).await.is_ok()
}

#[cfg(test)]
mod tests {
    use super::*;

    // `migrations = false`: la base de cada prueba empieza vacía, para probar `run_migrations`.
    #[sqlx::test(migrations = false)]
    async fn las_migraciones_crean_las_unidades_base(pool: PgPool) {
        run_migrations(&pool).await.unwrap();

        let unidades: Vec<(String, bool)> =
            sqlx::query_as("SELECT nombre, allows_fraction FROM unidades_medida ORDER BY nombre")
                .fetch_all(&pool)
                .await
                .unwrap();
        assert_eq!(
            unidades,
            vec![
                ("Gramo".to_string(), true),
                ("Hora".to_string(), true),
                ("Metro".to_string(), true),
                ("Pieza".to_string(), false),
            ]
        );
    }

    #[sqlx::test(migrations = false)]
    async fn correr_las_migraciones_otra_vez_no_cambia_nada(pool: PgPool) {
        run_migrations(&pool).await.unwrap();
        run_migrations(&pool).await.unwrap();

        let total: i64 = sqlx::query_scalar("SELECT count(*) FROM unidades_medida")
            .fetch_one(&pool)
            .await
            .unwrap();
        assert_eq!(total, 4);
    }

    #[sqlx::test(migrations = false)]
    async fn una_base_viva_responde(pool: PgPool) {
        assert!(is_db_alive(&pool).await);
    }
}
