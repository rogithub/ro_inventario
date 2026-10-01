//! Tipos que comparten todas las áreas del negocio: dinero, cantidades, identificadores, errores.
//!
//! Solo depende de `uuid`: una consulta SQL o una respuesta HTTP aquí no compila.

// El dinero y las cantidades son `Decimal`, nunca flotantes (docs/decisiones/2026-09-29-formato-y-linters.md).
#![deny(clippy::float_arithmetic)]

pub mod nombres;

/// Los ids de las tablas. Cada área define su tipo propio (`CategoriaId(Uuid)`), para que el
/// compilador no deje pasar el id de una cosa donde va el de otra.
pub use uuid::Uuid;

/// Falla de la infraestructura detrás de un repositorio (la base no responde, se cortó la conexión).
/// No es culpa del usuario: el detalle va al log y la pantalla muestra el id de la petición.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RepoError(pub String);

impl std::fmt::Display for RepoError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "falla del repositorio: {}", self.0)
    }
}

impl std::error::Error for RepoError {}
