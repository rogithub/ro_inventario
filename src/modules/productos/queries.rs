use sqlx::PgPool;
use uuid::Uuid;

use super::models::ProductoBuscado;
use crate::error::AppError;

const COLS: &str = "nid, id, nombre, esservicio, unidadmedida,
    ultimoprecioventa, preciocomprapromedio,
    COALESCE(stock, 0) AS stock,
    COALESCE(codigobarrasitem, '') AS codigobarrasitem,
    COALESCE(codigobarrascaja, '') AS codigobarrascaja";

pub async fn buscar(
    pool: &PgPool,
    q: &str,
    solo_con_stock: bool,
) -> Result<Vec<ProductoBuscado>, AppError> {
    let stock_cond = if solo_con_stock { "AND stock > 0" } else { "" };

    // 1. Entero → NID
    if let Ok(nid) = q.trim().parse::<i32>() {
        return Ok(sqlx::query_as(&format!(
            "SELECT {COLS} FROM v_inventario WHERE nid = $1 {stock_cond} LIMIT 10"
        ))
        .bind(nid)
        .fetch_all(pool)
        .await?);
    }

    // 2. UUID → ID propio (QR code generado por el sistema)
    if let Ok(id) = q.trim().parse::<Uuid>() {
        return Ok(sqlx::query_as(&format!(
            "SELECT {COLS} FROM v_inventario WHERE id = $1 {stock_cond} LIMIT 1"
        ))
        .bind(id)
        .fetch_all(pool)
        .await?);
    }

    // 3. Código de barras exacto (lectores de barcode, incluyendo no-numéricos)
    let barcode: Vec<ProductoBuscado> = sqlx::query_as(&format!(
        "SELECT {COLS} FROM v_inventario
         WHERE (codigobarrasitem = $1 OR codigobarrascaja = $1) {stock_cond}
         LIMIT 1"
    ))
    .bind(q)
    .fetch_all(pool)
    .await?;
    if !barcode.is_empty() {
        return Ok(barcode);
    }

    // 4. FTS (search_vector) + ILIKE — rápido, tolera acentos
    let pattern_like = format!("%{}%", q);
    let fts: Vec<ProductoBuscado> = sqlx::query_as(&format!(
        "SELECT {COLS} FROM v_inventario v
         WHERE (
             v.nid IN (
                 SELECT DISTINCT nid FROM productos
                 WHERE search_vector @@ websearch_to_tsquery('spanish', $1)
             )
             OR unaccent(v.nombre) ILIKE unaccent($2)
         )
         {stock_cond}
         ORDER BY similarity(lower(unaccent(v.nombre)), lower(unaccent($3))) DESC,
                  v.nombre
         LIMIT 20"
    ))
    .bind(q)
    .bind(&pattern_like)
    .bind(q)
    .fetch_all(pool)
    .await?;
    if !fts.is_empty() {
        return Ok(fts);
    }

    // 5. Trigramas — más lento, tolera typos
    Ok(sqlx::query_as(&format!(
        "SELECT {COLS} FROM v_inventario v
         WHERE v.nid IN (
             SELECT nid FROM productos
             WHERE nombre % $1
             ORDER BY similarity(nombre, $1) DESC
             LIMIT 20
         )
         {stock_cond}
         ORDER BY similarity(lower(unaccent(v.nombre)), lower(unaccent($1))) DESC
         LIMIT 20"
    ))
    .bind(q)
    .fetch_all(pool)
    .await?)
}
