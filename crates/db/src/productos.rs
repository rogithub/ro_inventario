use inventario::articulos::{ArticuloId, Nid, PrecioVenta};
use inventario::categorias::CategoriaId;
use inventario::productos::search::{SearchQuery, SearchResults};
use inventario::productos::{NewProducto, Producto, ProductoError, ProductosRepo};
use inventario::unidades_medida::UnidadMedidaId;
use kernel::{Decimal, RepoError};
use sqlx::PgPool;
use usuarios::usuarios::Email;

#[derive(Clone)]
pub struct PgProductos {
    pool: PgPool,
}

impl PgProductos {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

fn repo_error(error: sqlx::Error) -> RepoError {
    RepoError(error.to_string())
}

/// Un precio de la base ya cumple el `CHECK`; si no se puede construir, algo escribió sin pasar
/// por el área.
fn precio_from_db(precio: Option<Decimal>) -> Result<Option<PrecioVenta>, RepoError> {
    precio
        .map(|p| PrecioVenta::new(p).map_err(|e| RepoError(format!("precio {p} en la base: {e}"))))
        .transpose()
}

/// Lo escrito se busca tal cual: `%`, `_` y `\` no son comodines de `LIKE` (su escape es `\`).
fn escape_like(text: &str) -> String {
    let mut escaped = String::with_capacity(text.len());
    for c in text.chars() {
        if matches!(c, '%' | '_' | '\\') {
            escaped.push('\\');
        }
        escaped.push(c);
    }
    escaped
}

impl ProductosRepo for PgProductos {
    async fn list(&self) -> Result<Vec<Producto>, RepoError> {
        let rows = sqlx::query!(
            "SELECT a.id, a.nid, a.nombre, a.categoria_id, a.unidad_medida_id, a.precio_venta,
                    a.descripcion, p.marca, p.modelo, p.color
             FROM articulos a
             JOIN productos p ON p.articulo_id = a.id
             ORDER BY lower(a.nombre), a.nid"
        )
        .fetch_all(&self.pool)
        .await
        .map_err(repo_error)?;
        rows.into_iter()
            .map(|r| {
                Ok(Producto {
                    id: ArticuloId(r.id),
                    nid: Nid(r.nid),
                    nombre: r.nombre,
                    categoria_id: CategoriaId(r.categoria_id),
                    unidad_medida_id: UnidadMedidaId(r.unidad_medida_id),
                    precio_venta: precio_from_db(r.precio_venta)?,
                    descripcion: r.descripcion,
                    marca: r.marca,
                    modelo: r.modelo,
                    color: r.color,
                })
            })
            .collect()
    }

    /// Una sola consulta: cada palabra debe aparecer en `unaccent(lower(nombre))`; el NID buscado
    /// coincide aunque el nombre no, y va primero. El total sale de la misma consulta (`OVER ()`).
    async fn search(
        &self,
        search_query: &SearchQuery,
        categoria: Option<CategoriaId>,
        limit: u32,
    ) -> Result<SearchResults, RepoError> {
        let patterns: Vec<String> = search_query
            .words()
            .iter()
            .map(|p| escape_like(p))
            .collect();
        let rows = sqlx::query!(
            r#"SELECT a.id, a.nid, a.nombre, a.categoria_id, a.unidad_medida_id, a.precio_venta,
                      a.descripcion, p.marca, p.modelo, p.color, count(*) OVER () AS "total!"
               FROM articulos a
               JOIN productos p ON p.articulo_id = a.id
               WHERE ($2::uuid IS NULL OR a.categoria_id = $2)
                 AND (a.nid = $3
                      OR NOT EXISTS (
                          SELECT 1 FROM unnest($1::text[]) AS w(patron)
                          WHERE unaccent(lower(a.nombre))
                                NOT LIKE '%' || unaccent(lower(w.patron)) || '%'))
               ORDER BY a.nid IS NOT DISTINCT FROM $3 DESC, lower(a.nombre), a.nid
               LIMIT $4"#,
            &patterns,
            categoria.map(|c| c.0),
            search_query.nid().map(|n| n.0),
            i64::from(limit)
        )
        .fetch_all(&self.pool)
        .await
        .map_err(repo_error)?;
        // Sin rows no hay de dónde leer el total: ninguno coincidió.
        let total = rows.first().map_or(0, |r| r.total.unsigned_abs());
        let productos = rows
            .into_iter()
            .map(|r| {
                Ok(Producto {
                    id: ArticuloId(r.id),
                    nid: Nid(r.nid),
                    nombre: r.nombre,
                    categoria_id: CategoriaId(r.categoria_id),
                    unidad_medida_id: UnidadMedidaId(r.unidad_medida_id),
                    precio_venta: precio_from_db(r.precio_venta)?,
                    descripcion: r.descripcion,
                    marca: r.marca,
                    modelo: r.modelo,
                    color: r.color,
                })
            })
            .collect::<Result<_, RepoError>>()?;
        Ok(SearchResults { productos, total })
    }

    /// Todo en una transacción: el artículo, su renglón de producto y su primer precio.
    async fn add(&self, new: NewProducto, by: &Email) -> Result<Producto, ProductoError> {
        let mut tx = self.pool.begin().await.map_err(repo_error)?;
        let usuario_id = sqlx::query_scalar!(
            // Un desactivado ya no puede hacer nada, aunque su sesión siguiera abierta.
            "SELECT id FROM usuarios WHERE email = $1 AND deactivated_at IS NULL",
            by.as_str()
        )
        .fetch_optional(&mut *tx)
        .await
        .map_err(repo_error)?
        .ok_or(ProductoError::UsuarioNotFound)?;
        let precio = new.precio_venta().map(|p| p.value());

        let row = sqlx::query!(
            "INSERT INTO articulos
                 (kind, nombre, categoria_id, unidad_medida_id, precio_venta, descripcion, updated_by)
             VALUES ('producto', $1, $2, $3, $4, $5, $6)
             RETURNING id, nid",
            new.nombre(),
            new.categoria_id().0,
            new.unidad_medida_id().0,
            precio,
            new.descripcion(),
            usuario_id
        )
        .fetch_one(&mut *tx)
        .await
        .map_err(
            |e| match e.as_database_error().and_then(|d| d.constraint()) {
                Some("articulos_categoria_id_fkey") => ProductoError::CategoriaNotFound,
                Some("articulos_unidad_medida_id_fkey") => ProductoError::UnidadMedidaNotFound,
                _ => ProductoError::Repo(repo_error(e)),
            },
        )?;

        sqlx::query!(
            "INSERT INTO productos (articulo_id, marca, modelo, color) VALUES ($1, $2, $3, $4)",
            row.id,
            new.marca(),
            new.modelo(),
            new.color()
        )
        .execute(&mut *tx)
        .await
        .map_err(repo_error)?;

        if let Some(precio) = precio {
            sqlx::query!(
                "INSERT INTO precios_historial (articulo_id, precio_venta, updated_by)
                 VALUES ($1, $2, $3)",
                row.id,
                precio,
                usuario_id
            )
            .execute(&mut *tx)
            .await
            .map_err(repo_error)?;
        }

        tx.commit().await.map_err(repo_error)?;
        Ok(Producto {
            id: ArticuloId(row.id),
            nid: Nid(row.nid),
            nombre: new.nombre().to_string(),
            categoria_id: new.categoria_id(),
            unidad_medida_id: new.unidad_medida_id(),
            precio_venta: new.precio_venta(),
            descripcion: new.descripcion().map(str::to_string),
            marca: new.marca().map(str::to_string),
            modelo: new.modelo().map(str::to_string),
            color: new.color().map(str::to_string),
        })
    }
}

#[cfg(test)]
mod tests;
