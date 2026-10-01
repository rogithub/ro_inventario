use inventario::unidades_medida::{
    NewUnidadMedida, UnidadMedida, UnidadMedidaError, UnidadMedidaId, UnidadesMedidaRepo,
};
use kernel::RepoError;
use sqlx::PgPool;

#[derive(Clone)]
pub struct PgUnidadesMedida {
    pool: PgPool,
}

impl PgUnidadesMedida {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

impl UnidadesMedidaRepo for PgUnidadesMedida {
    async fn list(&self) -> Result<Vec<UnidadMedida>, RepoError> {
        let rows = sqlx::query!(
            "SELECT id, nombre, allows_fraction FROM unidades_medida ORDER BY lower(nombre)"
        )
        .fetch_all(&self.pool)
        .await
        .map_err(|e| RepoError(e.to_string()))?;
        Ok(rows
            .into_iter()
            .map(|r| UnidadMedida {
                id: UnidadMedidaId(r.id),
                nombre: r.nombre,
                allows_fraction: r.allows_fraction,
            })
            .collect())
    }

    async fn add(&self, new_unidad: NewUnidadMedida) -> Result<UnidadMedida, UnidadMedidaError> {
        let row = sqlx::query!(
            "INSERT INTO unidades_medida (nombre, allows_fraction) VALUES ($1, $2)
             RETURNING id, nombre, allows_fraction",
            new_unidad.nombre(),
            new_unidad.allows_fraction()
        )
        .fetch_one(&self.pool)
        .await
        .map_err(
            |e| match e.as_database_error().and_then(|d| d.constraint()) {
                Some("unidades_medida_nombre_unique") => {
                    UnidadMedidaError::DuplicateNombre(new_unidad.nombre().to_string())
                }
                _ => UnidadMedidaError::Repo(RepoError(e.to_string())),
            },
        )?;
        Ok(UnidadMedida {
            id: UnidadMedidaId(row.id),
            nombre: row.nombre,
            allows_fraction: row.allows_fraction,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use inventario::unidades_medida::contract;

    #[sqlx::test]
    async fn cumple_el_contrato_agregada_aparece_en_la_lista(pool: PgPool) {
        contract::agregada_aparece_en_la_lista(&PgUnidadesMedida::new(pool)).await;
    }

    #[sqlx::test]
    async fn cumple_el_contrato_trae_su_id(pool: PgPool) {
        contract::agregada_trae_su_id_y_la_lista_lo_conserva(&PgUnidadesMedida::new(pool)).await;
    }

    #[sqlx::test]
    async fn cumple_el_contrato_nombre_repetido(pool: PgPool) {
        contract::nombre_repetido_se_rechaza_sin_importar_mayusculas(&PgUnidadesMedida::new(pool))
            .await;
    }

    #[sqlx::test]
    async fn cumple_el_contrato_orden_como_postgres(pool: PgPool) {
        contract::la_lista_ordena_como_postgres_acentos_espacios_y_signos(&PgUnidadesMedida::new(
            pool,
        ))
        .await;
    }

    #[sqlx::test]
    async fn cumple_el_contrato_orden_alfabetico(pool: PgPool) {
        contract::la_lista_va_en_orden_alfabetico(&PgUnidadesMedida::new(pool)).await;
    }

    #[sqlx::test]
    async fn trae_las_unidades_base_de_la_migracion(pool: PgPool) {
        let nombres: Vec<String> = PgUnidadesMedida::new(pool)
            .list()
            .await
            .unwrap()
            .into_iter()
            .map(|u| u.nombre)
            .collect();
        assert_eq!(nombres, ["Gramo", "Hora", "Metro", "Pieza"]);
    }
}
