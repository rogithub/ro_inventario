//! Categorías del catálogo: planas, sin jerarquía; productos y servicios las comparten.

use std::fmt;
use std::future::Future;

use kernel::nombres::MAX_NOMBRE_CATALOGO;
use kernel::{RepoError, Uuid};

/// El id de una categoría: no se confunde con el de otra tabla.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct CategoriaId(pub Uuid);

impl CategoriaId {
    /// El id escrito en una ruta o un formulario; `None` si no es un uuid.
    pub fn parse(text: &str) -> Option<Self> {
        Uuid::parse_str(text).ok().map(Self)
    }
}

impl fmt::Display for CategoriaId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.0.fmt(f)
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Categoria {
    pub id: CategoriaId,
    pub nombre: String,
}

/// Una categoría por agregar, ya validada: solo se construye con `new`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NewCategoria {
    nombre: String,
}

impl NewCategoria {
    /// Quita los espacios de las orillas del nombre; vacío o de más de 150 caracteres no se acepta.
    pub fn new(nombre: &str) -> Result<Self, CategoriaError> {
        let nombre = nombre.trim();
        if nombre.is_empty() {
            return Err(CategoriaError::EmptyNombre);
        }
        if nombre.chars().count() > MAX_NOMBRE_CATALOGO {
            return Err(CategoriaError::LongNombre);
        }
        Ok(Self {
            nombre: nombre.to_string(),
        })
    }

    pub fn nombre(&self) -> &str {
        &self.nombre
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CategoriaError {
    EmptyNombre,
    /// Más de `MAX_NOMBRE_CATALOGO` caracteres.
    LongNombre,
    /// Ya hay una con ese nombre, sin importar mayúsculas ("cuadernos" choca con "Cuadernos").
    DuplicateNombre(String),
    /// Al renombrar: ya no existe (alguien la cambió mientras tanto, o el id no es de ninguna).
    NotFound,
    Repo(RepoError),
}

impl fmt::Display for CategoriaError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::EmptyNombre => write!(f, "Escribe el nombre de la categoría."),
            Self::LongNombre => write!(
                f,
                "El nombre no puede pasar de {MAX_NOMBRE_CATALOGO} caracteres."
            ),
            Self::DuplicateNombre(nombre) => write!(f, "Ya existe la categoría «{nombre}»."),
            Self::NotFound => write!(f, "Esa categoría ya no existe. Recarga la página."),
            Self::Repo(_) => write!(f, "No se pudo guardar la categoría."),
        }
    }
}

impl From<RepoError> for CategoriaError {
    fn from(error: RepoError) -> Self {
        Self::Repo(error)
    }
}

/// Dónde viven las categorías. Lo implementa `db`; para pruebas, `in_memory`.
pub trait CategoriasRepo {
    /// Todas, en orden alfabético.
    fn list(&self) -> impl Future<Output = Result<Vec<Categoria>, RepoError>> + Send;

    fn add(
        &self,
        new_categoria: NewCategoria,
    ) -> impl Future<Output = Result<Categoria, CategoriaError>> + Send;

    /// Le pone otro nombre; el id no cambia. Cambiar solo mayúsculas ("plumas" → "Plumas") se vale.
    fn rename(
        &self,
        id: CategoriaId,
        new_categoria: NewCategoria,
    ) -> impl Future<Output = Result<Categoria, CategoriaError>> + Send;
}

#[cfg(any(test, feature = "test-support"))]
pub mod in_memory;

/// Lo que toda implementación de `CategoriasRepo` debe cumplir. Corre contra la de memoria
/// (aquí) y contra la de Postgres (en `db`).
#[cfg(any(test, feature = "test-support"))]
#[allow(clippy::unwrap_used)] // código de pruebas
pub mod contract;

#[cfg(test)]
mod tests;
