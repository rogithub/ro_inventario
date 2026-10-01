use inventario::categorias::{
    Categoria, CategoriaError, CategoriaId, CategoriasRepo, NewCategoria,
};
use kernel::RepoError;
use sqlx::PgPool;

#[derive(Clone)]
pub struct PgCategorias {
    pool: PgPool,
}

impl PgCategorias {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

impl CategoriasRepo for PgCategorias {
    async fn list(&self) -> Result<Vec<Categoria>, RepoError> {
        let rows = sqlx::query!("SELECT id, nombre FROM categorias ORDER BY lower(nombre)")
            .fetch_all(&self.pool)
            .await
            .map_err(|e| RepoError(e.to_string()))?;
        Ok(rows
            .into_iter()
            .map(|r| Categoria {
                id: CategoriaId(r.id),
                nombre: r.nombre,
            })
            .collect())
    }

    async fn add(&self, new_categoria: NewCategoria) -> Result<Categoria, CategoriaError> {
        let row = sqlx::query!(
            "INSERT INTO categorias (nombre) VALUES ($1) RETURNING id, nombre",
            new_categoria.nombre()
        )
        .fetch_one(&self.pool)
        .await
        .map_err(
            |e| match e.as_database_error().and_then(|d| d.constraint()) {
                Some("categorias_nombre_unique") => {
                    CategoriaError::DuplicateNombre(new_categoria.nombre().to_string())
                }
                _ => CategoriaError::Repo(RepoError(e.to_string())),
            },
        )?;
        Ok(Categoria {
            id: CategoriaId(row.id),
            nombre: row.nombre,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use inventario::categorias::contract;

    #[sqlx::test]
    async fn cumple_el_contrato_agregada_aparece_en_la_lista(pool: PgPool) {
        contract::agregada_aparece_en_la_lista(&PgCategorias::new(pool)).await;
    }

    #[sqlx::test]
    async fn cumple_el_contrato_trae_su_id(pool: PgPool) {
        contract::agregada_trae_su_id_y_la_lista_lo_conserva(&PgCategorias::new(pool)).await;
    }

    #[sqlx::test]
    async fn cumple_el_contrato_nombre_repetido(pool: PgPool) {
        contract::nombre_repetido_se_rechaza_sin_importar_mayusculas(&PgCategorias::new(pool))
            .await;
    }

    #[sqlx::test]
    async fn cumple_el_contrato_orden_como_postgres(pool: PgPool) {
        contract::la_lista_ordena_como_postgres_acentos_espacios_y_signos(&PgCategorias::new(pool))
            .await;
    }

    #[sqlx::test]
    async fn cumple_el_contrato_orden_alfabetico(pool: PgPool) {
        contract::la_lista_va_en_orden_alfabetico(&PgCategorias::new(pool)).await;
    }

    #[sqlx::test]
    async fn la_migracion_no_trae_categorias(pool: PgPool) {
        assert!(PgCategorias::new(pool).list().await.unwrap().is_empty());
    }
}
