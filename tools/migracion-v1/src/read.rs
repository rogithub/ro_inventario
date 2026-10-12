//! Lectura de la copia de la v1. Con `sqlx::query` sin macros: el esquema de la v1 no es el
//! nuestro, y las macros se revisan contra la base de la v2.
//!
//! En la v1, `productos` guarda también los servicios (`esservicio`) y los kits (`escompuesto`).
//! Por ahora solo se migran los productos; servicios y kits esperan al diseño 03. Cada consulta
//! repite ese filtro: `NOT p.esservicio AND NOT p.escompuesto`.

use kernel::Uuid;
use sqlx::{PgPool, Row};

/// Una categoría de la v1, tal como viene (con sus espacios).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CategoriaV1 {
    pub id: Uuid,
    pub nombre: String,
}

/// Cuántos registros debe tener la v2 al terminar, contados en la v1 con su propio SQL (no con
/// lo que salió de `transform`), para que el cuadre no se revise contra sí mismo.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct RecordCounts {
    /// Categorías distintas sin contar mayúsculas ni espacios de más ("CINTA " es "CINTA").
    pub categorias: i64,
}

/// Lo que se lee de la v1.
#[derive(Debug)]
pub struct V1 {
    pub categorias: Vec<CategoriaV1>,
    pub expected: RecordCounts,
}

/// Lee todo en una transacción de solo lectura y con una sola foto de la base: si la v1 cambiara
/// a media lectura, los datos y los conteos seguirían siendo del mismo momento.
pub async fn read(v1: &PgPool) -> Result<V1, sqlx::Error> {
    let mut tx = v1.begin().await?;
    sqlx::query("SET TRANSACTION ISOLATION LEVEL REPEATABLE READ, READ ONLY")
        .execute(&mut *tx)
        .await?;

    // Solo las categorías que usa algún producto: las vacías y las de servicios no se migran.
    let categorias = sqlx::query(
        "SELECT c.id, c.nombre FROM categorias c
         WHERE EXISTS (SELECT 1 FROM categoriasproductos cp JOIN productos p ON p.id = cp.productoid
                       WHERE cp.categoriaid = c.id AND NOT p.esservicio AND NOT p.escompuesto)
         ORDER BY c.nombre, c.id",
    )
    .fetch_all(&mut *tx)
    .await?
    .into_iter()
    .map(|row| {
        Ok(CategoriaV1 {
            id: row.try_get("id")?,
            nombre: row.try_get("nombre")?,
        })
    })
    .collect::<Result<_, sqlx::Error>>()?;

    let expected_categorias: i64 = sqlx::query_scalar(
        r"SELECT count(DISTINCT lower(btrim(regexp_replace(c.nombre, '\s+', ' ', 'g'))))
          FROM categorias c
          JOIN categoriasproductos cp ON cp.categoriaid = c.id
          JOIN productos p ON p.id = cp.productoid
          WHERE NOT p.esservicio AND NOT p.escompuesto",
    )
    .fetch_one(&mut *tx)
    .await?;

    tx.commit().await?;
    Ok(V1 {
        categorias,
        expected: RecordCounts {
            categorias: expected_categorias,
        },
    })
}

#[cfg(test)]
mod tests;
