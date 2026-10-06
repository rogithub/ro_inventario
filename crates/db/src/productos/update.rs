//! Buscar un producto por su NID y editarlo.

use super::*;

pub(super) async fn get_by_nid(pool: &PgPool, nid: Nid) -> Result<Option<Producto>, RepoError> {
    let row = sqlx::query!(
        "SELECT a.id, a.nid, a.nombre, a.categoria_id, a.unidad_medida_id, a.precio_venta,
                a.descripcion, p.marca, p.modelo, p.color
         FROM articulos a
         JOIN productos p ON p.articulo_id = a.id
         WHERE a.nid = $1",
        nid.0
    )
    .fetch_optional(pool)
    .await
    .map_err(repo_error)?;
    row.map(|r| {
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
    .transpose()
}

/// Todo en una transacción: el artículo, su renglón de producto y, si el precio cambió, su
/// historial. El renglón queda bloqueado desde que se lee el precio anterior: dos ediciones a la
/// vez no comparan contra el mismo precio.
pub(super) async fn update(
    pool: &PgPool,
    id: ArticuloId,
    new: NewProducto,
    by: &Email,
) -> Result<Producto, ProductoError> {
    let mut tx = pool.begin().await.map_err(repo_error)?;
    let usuario_id = active_usuario_id(&mut tx, by).await?;
    let previous = sqlx::query_scalar!(
        "SELECT a.precio_venta
         FROM articulos a
         JOIN productos p ON p.articulo_id = a.id
         WHERE a.id = $1
         FOR UPDATE OF a",
        id.0
    )
    .fetch_optional(&mut *tx)
    .await
    .map_err(repo_error)?
    .ok_or(ProductoError::NotFound)?;
    let precio = new.precio_venta().map(|p| p.value());
    if previous.is_some() && precio.is_none() {
        return Err(ProductoError::CannotRemovePrecio);
    }

    let nid = sqlx::query_scalar!(
        "UPDATE articulos
         SET nombre = $2, categoria_id = $3, unidad_medida_id = $4, precio_venta = $5,
             descripcion = $6, updated_at = now(), updated_by = $7
         WHERE id = $1
         RETURNING nid",
        id.0,
        new.nombre(),
        new.categoria_id().0,
        new.unidad_medida_id().0,
        precio,
        new.descripcion(),
        usuario_id
    )
    .fetch_one(&mut *tx)
    .await
    .map_err(articulo_error)?;

    sqlx::query!(
        "UPDATE productos SET marca = $2, modelo = $3, color = $4 WHERE articulo_id = $1",
        id.0,
        new.marca(),
        new.modelo(),
        new.color()
    )
    .execute(&mut *tx)
    .await
    .map_err(repo_error)?;

    if let Some(precio) = precio
        && Some(precio) != previous
    {
        sqlx::query!(
            "INSERT INTO precios_historial (articulo_id, precio_venta, updated_by)
             VALUES ($1, $2, $3)",
            id.0,
            precio,
            usuario_id
        )
        .execute(&mut *tx)
        .await
        .map_err(repo_error)?;
    }

    tx.commit().await.map_err(repo_error)?;
    Ok(Producto {
        id,
        nid: Nid(nid),
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
