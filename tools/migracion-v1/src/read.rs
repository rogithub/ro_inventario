//! Lectura de la copia de la v1. Con `sqlx::query` sin macros: el esquema de la v1 no es el
//! nuestro, y las macros se revisan contra la base de la v2.
//!
//! En la v1, `productos` guarda también los servicios (`esservicio`) y los kits (`escompuesto`).
//! Por ahora solo se migran los productos; servicios y kits esperan al diseño 03. Cada consulta
//! repite ese filtro: `NOT p.esservicio AND NOT p.escompuesto`.

use std::fmt;

use chrono::NaiveDateTime;
use kernel::{Decimal, Uuid};
use sqlx::{PgPool, Row};

/// Una categoría de la v1, tal como viene (con sus espacios).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CategoriaV1 {
    pub id: Uuid,
    pub nombre: String,
}

/// Un producto de la v1, tal como viene. Las fechas de la v1 no llevan zona: son hora de Cancún.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ProductoV1 {
    pub id: Uuid,
    pub nid: i32,
    pub nombre: String,
    /// El nombre de su unidad en la v1 ("PIEZA"); `None` si no tiene.
    pub unidad: Option<String>,
    /// Sus categorías distintas: en la v1 un producto puede repetir la misma (el NID 5).
    pub categoria_ids: Vec<Uuid>,
    pub marca: Option<String>,
    pub modelo: Option<String>,
    pub color: Option<String>,
    pub descripcion: Option<String>,
    /// Su última edición en la v1 (`datestamp`), sin contar los cambios de precio. En la v1 puede
    /// venir vacío (hoy ninguno): se lee para que las reglas digan cuál es, no aquí con un error de
    /// la base.
    pub datestamp: Option<NaiveDateTime>,
}

/// Un renglón del historial de precios de la v1 (`preciosproductos`).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PrecioV1 {
    pub producto_id: Uuid,
    /// Puede venir vacío en la v1 (hoy ninguno), como `datestamp`.
    pub precio_venta: Option<Decimal>,
    /// `fechacreado`: desde cuándo tuvo ese precio.
    pub created_at: NaiveDateTime,
}

/// Cuántos registros debe tener la v2 al terminar, contados en la v1 con su propio SQL (no con
/// lo que salió de `transform`), para que el cuadre no se revise contra sí mismo.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct RecordCounts {
    /// Categorías distintas sin contar mayúsculas ni espacios de más ("CINTA " es "CINTA").
    pub categorias: i64,
    pub productos: i64,
    /// Renglones del historial de precios de los productos.
    pub precios: i64,
    /// La suma de los precios del historial: cuadra también los montos, no solo los renglones.
    pub precios_sum: Decimal,
}

/// Lo que se lee de la v1.
#[derive(Debug)]
pub struct V1 {
    pub categorias: Vec<CategoriaV1>,
    pub productos: Vec<ProductoV1>,
    pub precios: Vec<PrecioV1>,
    pub expected: RecordCounts,
    /// El NID más alto de toda la v1, también de servicios y kits: las altas nuevas siguen de
    /// ahí, para no tomar los NID que los servicios necesitarán al migrarse (dueño, 2026-10-11).
    pub max_nid: i32,
}

#[derive(Debug)]
pub enum ReadError {
    /// Tantos artículos tienen `esservicio` o `escompuesto` vacío: el filtro los dejaría fuera sin
    /// avisar, y el cuadre no lo notaría porque los conteos esperados repiten el filtro.
    UnknownKind(i64),
    Db(sqlx::Error),
}

impl fmt::Display for ReadError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::UnknownKind(n) => write!(
                f,
                "{n} artículos de la v1 no dicen si son servicio o kit (esservicio o escompuesto \
                 vacío): no se sabe si migrarlos como productos."
            ),
            Self::Db(e) => e.fmt(f),
        }
    }
}

impl From<sqlx::Error> for ReadError {
    fn from(error: sqlx::Error) -> Self {
        Self::Db(error)
    }
}

/// Lee todo en una transacción de solo lectura y con una sola foto de la base: si la v1 cambiara
/// a media lectura, los datos y los conteos seguirían siendo del mismo momento.
pub async fn read(v1: &PgPool) -> Result<V1, ReadError> {
    let mut tx = v1.begin().await?;
    sqlx::query("SET TRANSACTION ISOLATION LEVEL REPEATABLE READ, READ ONLY")
        .execute(&mut *tx)
        .await?;

    let unknown_kind: i64 = sqlx::query_scalar(
        "SELECT count(*) FROM productos WHERE esservicio IS NULL OR escompuesto IS NULL",
    )
    .fetch_one(&mut *tx)
    .await?;
    if unknown_kind > 0 {
        return Err(ReadError::UnknownKind(unknown_kind));
    }

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

    let productos = sqlx::query(
        "SELECT p.id, p.nid, p.nombre, u.nombre AS unidad, p.marca, p.modelo, p.color,
                p.descripcion, p.datestamp,
                ARRAY(SELECT DISTINCT cp.categoriaid FROM categoriasproductos cp
                      WHERE cp.productoid = p.id ORDER BY 1) AS categoria_ids
         FROM productos p
         LEFT JOIN unidadesmedida u ON u.id = p.unidadmedidaid
         WHERE NOT p.esservicio AND NOT p.escompuesto
         ORDER BY p.nid",
    )
    .fetch_all(&mut *tx)
    .await?
    .into_iter()
    .map(|row| {
        Ok(ProductoV1 {
            id: row.try_get("id")?,
            nid: row.try_get("nid")?,
            nombre: row.try_get("nombre")?,
            unidad: row.try_get("unidad")?,
            categoria_ids: row.try_get("categoria_ids")?,
            marca: row.try_get("marca")?,
            modelo: row.try_get("modelo")?,
            color: row.try_get("color")?,
            descripcion: row.try_get("descripcion")?,
            datestamp: row.try_get("datestamp")?,
        })
    })
    .collect::<Result<_, sqlx::Error>>()?;

    let precios = sqlx::query(
        "SELECT pp.productoid, pp.precioventa, pp.fechacreado
         FROM preciosproductos pp JOIN productos p ON p.id = pp.productoid
         WHERE NOT p.esservicio AND NOT p.escompuesto
         ORDER BY pp.productoid, pp.fechacreado",
    )
    .fetch_all(&mut *tx)
    .await?
    .into_iter()
    .map(|row| {
        Ok(PrecioV1 {
            producto_id: row.try_get("productoid")?,
            precio_venta: row.try_get("precioventa")?,
            created_at: row.try_get("fechacreado")?,
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
    let expected_productos: i64 = sqlx::query_scalar(
        "SELECT count(*) FROM productos WHERE NOT esservicio AND NOT escompuesto",
    )
    .fetch_one(&mut *tx)
    .await?;
    let (expected_precios, expected_precios_sum): (i64, Decimal) = sqlx::query_as(
        "SELECT count(*), coalesce(sum(pp.precioventa), 0)
         FROM preciosproductos pp JOIN productos p ON p.id = pp.productoid
         WHERE NOT p.esservicio AND NOT p.escompuesto",
    )
    .fetch_one(&mut *tx)
    .await?;
    // Sin el filtro de productos, a propósito: es el más alto de toda la v1.
    let max_nid: i32 = sqlx::query_scalar("SELECT coalesce(max(nid), 0) FROM productos")
        .fetch_one(&mut *tx)
        .await?;

    tx.commit().await?;
    Ok(V1 {
        categorias,
        productos,
        precios,
        expected: RecordCounts {
            categorias: expected_categorias,
            productos: expected_productos,
            precios: expected_precios,
            precios_sum: expected_precios_sum,
        },
        max_nid,
    })
}

#[cfg(test)]
mod tests;
