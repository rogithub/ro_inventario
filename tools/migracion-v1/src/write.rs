//! Escritura en la v2, en una sola transacción: o guarda todo y cuadrado, o no guarda nada.
//!
//! Solo agrega: no lleva DELETE, TRUNCATE ni DROP. Si la base destino ya tiene datos, se niega
//! sin tocarla; para correrla otra vez en desarrollo se recrea la base con herramientas/dev-db.sh.

use std::fmt;

use sqlx::PgPool;

use crate::read::RecordCounts;
use crate::transform::Categorias;
use crate::transform::productos::MigratedProducto;

#[derive(Debug)]
pub enum WriteError {
    /// La base destino ya tiene categorías o artículos.
    NotEmpty {
        categorias: i64,
        articulos: i64,
    },
    /// Lo escrito no coincide con lo contado en la v1. No quedó nada.
    CountMismatch {
        expected: RecordCounts,
        written: RecordCounts,
    },
    /// Un producto migrado tiene un NID más alto que el más alto de la v1: la secuencia quedaría
    /// por debajo y la siguiente alta chocaría. No quedó nada.
    NidAboveV1 {
        max_nid_v1: i32,
        max_nid_written: i32,
    },
    Db(sqlx::Error),
}

impl fmt::Display for WriteError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::NotEmpty {
                categorias,
                articulos,
            } => write!(
                f,
                "La base destino ya tiene datos ({categorias} categorías, {articulos} artículos): \
                 la migración solo escribe en una base vacía. No se tocó nada."
            ),
            Self::CountMismatch { expected, written } => write!(
                f,
                "Los conteos no cuadran (esperados en la v1: {expected:?}; escritos: {written:?}). \
                 No quedó nada."
            ),
            Self::NidAboveV1 {
                max_nid_v1,
                max_nid_written,
            } => write!(
                f,
                "Se escribió el NID {max_nid_written}, más alto que el más alto de la v1 \
                 ({max_nid_v1}): las altas nuevas chocarían. No quedó nada."
            ),
            Self::Db(e) => write!(f, "Falla de la base: {e}"),
        }
    }
}

impl From<sqlx::Error> for WriteError {
    fn from(error: sqlx::Error) -> Self {
        Self::Db(error)
    }
}

/// Escribe las categorías, los productos y su historial de precios, y confirma solo si los
/// conteos de la v2 cuadran con los expected. Al final adelanta la secuencia del NID a
/// `max_nid_v1`, para que la primera alta reciba el siguiente.
pub async fn write(
    v2: &PgPool,
    categorias: &Categorias,
    productos: &[MigratedProducto],
    expected: RecordCounts,
    max_nid_v1: i32,
) -> Result<RecordCounts, WriteError> {
    let mut tx = v2.begin().await?;

    let existing = sqlx::query!(
        r#"SELECT (SELECT count(*) FROM categorias) AS "categorias!",
                  (SELECT count(*) FROM articulos) AS "articulos!""#
    )
    .fetch_one(&mut *tx)
    .await?;
    if existing.categorias > 0 || existing.articulos > 0 {
        return Err(WriteError::NotEmpty {
            categorias: existing.categorias,
            articulos: existing.articulos,
        });
    }

    for (id, categoria) in &categorias.to_insert {
        sqlx::query!(
            "INSERT INTO categorias (id, nombre) VALUES ($1, $2)",
            id.0,
            categoria.nombre()
        )
        .execute(&mut *tx)
        .await?;
    }

    // Sin `updated_by`: vacío quiere decir migrado de la v1 (migración 0007).
    for migrated in productos {
        let producto = &migrated.producto;
        sqlx::query!(
            "INSERT INTO articulos (id, nid, kind, nombre, categoria_id, unidad_medida_id,
                                    precio_venta, descripcion, created_at, updated_at)
             VALUES ($1, $2, 'producto', $3, $4, $5, $6, $7, $8, $9)",
            migrated.id.0,
            migrated.nid.0,
            producto.nombre(),
            producto.categoria_id().0,
            producto.unidad_medida_id().0,
            producto.precio_venta().map(|p| p.value()),
            producto.descripcion(),
            migrated.created_at,
            migrated.updated_at
        )
        .execute(&mut *tx)
        .await?;
        sqlx::query!(
            "INSERT INTO productos (articulo_id, marca, modelo, color) VALUES ($1, $2, $3, $4)",
            migrated.id.0,
            producto.marca(),
            producto.modelo(),
            producto.color()
        )
        .execute(&mut *tx)
        .await?;
        for precio in &migrated.historial {
            sqlx::query!(
                "INSERT INTO precios_historial (articulo_id, precio_venta, desde)
                 VALUES ($1, $2, $3)",
                migrated.id.0,
                precio.precio_venta.value(),
                precio.desde
            )
            .execute(&mut *tx)
            .await?;
        }
    }

    // Un producto cuenta solo si tiene su artículo y su renglón en `productos`.
    let written = sqlx::query!(
        r#"SELECT (SELECT count(*) FROM categorias) AS "categorias!",
                  (SELECT count(*) FROM articulos a JOIN productos p ON p.articulo_id = a.id
                   WHERE a.kind = 'producto') AS "productos!",
                  (SELECT count(*) FROM precios_historial) AS "precios!",
                  (SELECT coalesce(sum(precio_venta), 0) FROM precios_historial)
                      AS "precios_sum!""#
    )
    .fetch_one(&mut *tx)
    .await?;
    let written = RecordCounts {
        categorias: written.categorias,
        productos: written.productos,
        precios: written.precios,
        precios_sum: written.precios_sum,
    };
    if written != expected {
        // Al soltar `tx` sin confirmar, Postgres deshace todo lo escrito.
        return Err(WriteError::CountMismatch { expected, written });
    }

    let max_nid_written =
        sqlx::query_scalar!(r#"SELECT coalesce(max(nid), 0) AS "n!" FROM articulos"#)
            .fetch_one(&mut *tx)
            .await?;
    if max_nid_written > max_nid_v1 {
        return Err(WriteError::NidAboveV1 {
            max_nid_v1,
            max_nid_written,
        });
    }

    // Al último, ya cuadrado: `setval` no se deshace con un rollback. Si el commit fallara, la
    // secuencia quedaría adelantada en una base sin datos, que para repetir se recrea de todos modos.
    // Una v1 sin artículos (0) deja la secuencia sin usar: la siguiente alta recibe el 1.
    sqlx::query_scalar!(
        "SELECT setval(pg_get_serial_sequence('articulos', 'nid'), GREATEST($1, 1), $1 > 0)",
        max_nid_v1
    )
    .fetch_one(&mut *tx)
    .await?;

    tx.commit().await?;
    Ok(written)
}

#[cfg(test)]
mod tests;
