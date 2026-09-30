use inventario::unidades_medida::{
    NuevaUnidadMedida, UnidadMedida, UnidadMedidaError, UnidadesMedidaRepo,
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
        sqlx::query_as!(
            UnidadMedida,
            "SELECT nombre, allows_fraction FROM unidades_medida ORDER BY lower(nombre)"
        )
        .fetch_all(&self.pool)
        .await
        .map_err(|e| RepoError(e.to_string()))
    }

    async fn add(&self, nueva: NuevaUnidadMedida) -> Result<UnidadMedida, UnidadMedidaError> {
        sqlx::query_as!(
            UnidadMedida,
            "INSERT INTO unidades_medida (nombre, allows_fraction) VALUES ($1, $2)
             RETURNING nombre, allows_fraction",
            nueva.nombre(),
            nueva.allows_fraction()
        )
        .fetch_one(&self.pool)
        .await
        .map_err(
            |e| match e.as_database_error().and_then(|d| d.constraint()) {
                Some("unidades_medida_nombre_unico") => {
                    UnidadMedidaError::NombreRepetido(nueva.nombre().to_string())
                }
                _ => UnidadMedidaError::Repo(RepoError(e.to_string())),
            },
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use inventario::unidades_medida::contrato;

    #[sqlx::test]
    async fn cumple_el_contrato_agregada_aparece_en_la_lista(pool: PgPool) {
        contrato::agregada_aparece_en_la_lista(&PgUnidadesMedida::new(pool)).await;
    }

    #[sqlx::test]
    async fn cumple_el_contrato_nombre_repetido(pool: PgPool) {
        contrato::nombre_repetido_se_rechaza_sin_importar_mayusculas(&PgUnidadesMedida::new(pool))
            .await;
    }

    #[sqlx::test]
    async fn cumple_el_contrato_orden_alfabetico(pool: PgPool) {
        contrato::la_lista_va_en_orden_alfabetico(&PgUnidadesMedida::new(pool)).await;
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
