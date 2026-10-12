//! Escritura en la v2, en una sola transacción: o guarda todo y cuadrado, o no guarda nada.
//!
//! Solo agrega: no lleva DELETE, TRUNCATE ni DROP. Si la base destino ya tiene datos, se niega
//! sin tocarla; para correrla otra vez en desarrollo se recrea la base con herramientas/dev-db.sh.

use std::fmt;

use sqlx::PgPool;

use crate::read::RecordCounts;
use crate::transform::Categorias;

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
            Self::Db(e) => write!(f, "Falla de la base: {e}"),
        }
    }
}

impl From<sqlx::Error> for WriteError {
    fn from(error: sqlx::Error) -> Self {
        Self::Db(error)
    }
}

/// Escribe las categorías y confirma solo si los conteos de la v2 cuadran con los expected.
pub async fn write(
    v2: &PgPool,
    categorias: &Categorias,
    expected: RecordCounts,
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

    let written = RecordCounts {
        categorias: sqlx::query_scalar!(r#"SELECT count(*) AS "n!" FROM categorias"#)
            .fetch_one(&mut *tx)
            .await?,
    };
    if written != expected {
        // Al soltar `tx` sin confirmar, Postgres deshace todo lo escrito.
        return Err(WriteError::CountMismatch { expected, written });
    }

    tx.commit().await?;
    Ok(written)
}

#[cfg(test)]
mod tests;
