use inventario::categorias::{Categoria, CategoriaError, CategoriasRepo, NewCategoria};
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
        sqlx::query_as!(
            Categoria,
            "SELECT nombre FROM categorias ORDER BY lower(nombre)"
        )
        .fetch_all(&self.pool)
        .await
        .map_err(|e| RepoError(e.to_string()))
    }

    async fn add(&self, new_categoria: NewCategoria) -> Result<Categoria, CategoriaError> {
        sqlx::query_as!(
            Categoria,
            "INSERT INTO categorias (nombre) VALUES ($1) RETURNING nombre",
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
        )
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
    async fn cumple_el_contrato_nombre_repetido(pool: PgPool) {
        contract::nombre_repetido_se_rechaza_sin_importar_mayusculas(&PgCategorias::new(pool))
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
