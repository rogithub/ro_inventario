//! Unidades en que se cuenta el stock y se cobra: Pieza, Metro, Hoja…

use std::fmt;
use std::future::Future;

use kernel::nombres::MAX_NOMBRE_CATALOGO;
use kernel::{RepoError, Uuid};

/// El id de una unidad de medida: no se confunde con el de otra tabla.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct UnidadMedidaId(pub Uuid);

impl UnidadMedidaId {
    /// El id escrito en un formulario; `None` si no es un uuid.
    pub fn parse(text: &str) -> Option<Self> {
        Uuid::parse_str(text).ok().map(Self)
    }
}

impl fmt::Display for UnidadMedidaId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.0.fmt(f)
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct UnidadMedida {
    pub id: UnidadMedidaId,
    pub nombre: String,
    /// Si se puede vender en fracciones (1.5 metros) o solo en enteros (piezas).
    pub allows_fraction: bool,
}

/// Una unidad por agregar, ya validada: solo se construye con `new`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NewUnidadMedida {
    nombre: String,
    allows_fraction: bool,
}

impl NewUnidadMedida {
    /// Quita los espacios de las orillas del nombre; vacío o de más de 150 caracteres no se acepta.
    pub fn new(nombre: &str, allows_fraction: bool) -> Result<Self, UnidadMedidaError> {
        let nombre = nombre.trim();
        if nombre.is_empty() {
            return Err(UnidadMedidaError::EmptyNombre);
        }
        if nombre.chars().count() > MAX_NOMBRE_CATALOGO {
            return Err(UnidadMedidaError::LongNombre);
        }
        Ok(Self {
            nombre: nombre.to_string(),
            allows_fraction,
        })
    }

    pub fn nombre(&self) -> &str {
        &self.nombre
    }

    pub fn allows_fraction(&self) -> bool {
        self.allows_fraction
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum UnidadMedidaError {
    EmptyNombre,
    /// Más de `MAX_NOMBRE_CATALOGO` caracteres.
    LongNombre,
    /// Ya hay una con ese nombre, sin importar mayúsculas ("hoja" choca con "Hoja").
    DuplicateNombre(String),
    Repo(RepoError),
}

impl fmt::Display for UnidadMedidaError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::EmptyNombre => write!(f, "Escribe el nombre de la unidad."),
            Self::LongNombre => write!(
                f,
                "El nombre no puede pasar de {MAX_NOMBRE_CATALOGO} caracteres."
            ),
            Self::DuplicateNombre(nombre) => write!(f, "Ya existe la unidad «{nombre}»."),
            Self::Repo(_) => write!(f, "No se pudo guardar la unidad."),
        }
    }
}

impl From<RepoError> for UnidadMedidaError {
    fn from(error: RepoError) -> Self {
        Self::Repo(error)
    }
}

/// Dónde viven las unidades. Lo implementa `db`; para pruebas, `in_memory`.
pub trait UnidadesMedidaRepo {
    /// Todas, en orden alfabético.
    fn list(&self) -> impl Future<Output = Result<Vec<UnidadMedida>, RepoError>> + Send;

    fn add(
        &self,
        new_unidad: NewUnidadMedida,
    ) -> impl Future<Output = Result<UnidadMedida, UnidadMedidaError>> + Send;
}

#[cfg(any(test, feature = "test-support"))]
pub mod in_memory;

/// Lo que toda implementación de `UnidadesMedidaRepo` debe cumplir. Corre contra la de memoria
/// (aquí) y contra la de Postgres (en `db`). Cada función recibe un repo que puede traer
/// unidades de antes (las base de la migración); usa nombres que no existen.
#[cfg(any(test, feature = "test-support"))]
#[allow(clippy::unwrap_used)] // código de pruebas
pub mod contract;

#[cfg(test)]
mod tests;
