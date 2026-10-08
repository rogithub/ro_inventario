//! Descontinuar un producto y volver a manejarlo.

use super::*;

/// En una transacción, con el renglón bloqueado desde que se lee si ya estaba descontinuado: dos
/// a la vez no pisan la fecha del primero.
pub(super) async fn discontinue(
    pool: &PgPool,
    id: ArticuloId,
    motivo: MotivoDescontinuado,
    by: &Email,
) -> Result<Producto, ProductoError> {
    let mut tx = pool.begin().await.map_err(repo_error)?;
    let usuario_id = active_usuario_id(&mut tx, by).await?;
    let (nid, is_descontinuado) = lock(&mut tx, id).await?;
    if is_descontinuado {
        return Err(ProductoError::AlreadyDescontinuado);
    }

    sqlx::query!(
        "UPDATE articulos
         SET descontinuado_at = now(), motivo_descontinuado = $2,
             updated_at = now(), updated_by = $3
         WHERE id = $1",
        id.0,
        motivo.as_str(),
        usuario_id
    )
    .execute(&mut *tx)
    .await
    .map_err(repo_error)?;

    let producto = stored(&mut tx, nid).await?;
    tx.commit().await.map_err(repo_error)?;
    Ok(producto)
}

/// Si ya se manejaba, no escribe nada (ni quién).
pub(super) async fn resume(
    pool: &PgPool,
    id: ArticuloId,
    by: &Email,
) -> Result<Producto, ProductoError> {
    let mut tx = pool.begin().await.map_err(repo_error)?;
    let usuario_id = active_usuario_id(&mut tx, by).await?;
    let (nid, is_descontinuado) = lock(&mut tx, id).await?;

    if is_descontinuado {
        sqlx::query!(
            "UPDATE articulos
             SET descontinuado_at = NULL, motivo_descontinuado = NULL,
                 updated_at = now(), updated_by = $2
             WHERE id = $1",
            id.0,
            usuario_id
        )
        .execute(&mut *tx)
        .await
        .map_err(repo_error)?;
    }

    let producto = stored(&mut tx, nid).await?;
    tx.commit().await.map_err(repo_error)?;
    Ok(producto)
}

/// Bloquea el renglón del producto; regresa su NID y si ya estaba descontinuado.
async fn lock(
    tx: &mut sqlx::Transaction<'_, sqlx::Postgres>,
    id: ArticuloId,
) -> Result<(Nid, bool), ProductoError> {
    let row = sqlx::query!(
        r#"SELECT a.nid, a.descontinuado_at IS NOT NULL AS "is_descontinuado!"
           FROM articulos a
           JOIN productos p ON p.articulo_id = a.id
           WHERE a.id = $1
           FOR UPDATE OF a"#,
        id.0
    )
    .fetch_optional(&mut **tx)
    .await
    .map_err(repo_error)?
    .ok_or(ProductoError::NotFound)?;
    Ok((Nid(row.nid), row.is_descontinuado))
}

/// Como quedó, leído en la misma transacción: la fecha es la que puso la base.
async fn stored(
    tx: &mut sqlx::Transaction<'_, sqlx::Postgres>,
    nid: Nid,
) -> Result<Producto, ProductoError> {
    update::get_by_nid(&mut **tx, nid)
        .await?
        .ok_or(ProductoError::NotFound)
}
