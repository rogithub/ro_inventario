//! Productos: los artículos físicos, que se compran, llevan stock y se revenden o se consumen en
//! un servicio (diseño 01). Lo común con servicios y kits está en `articulos`.

pub mod search;

use std::fmt;
use std::future::Future;

use kernel::RepoError;
use kernel::nombres::MAX_NOMBRE_CATALOGO;
use usuarios::usuarios::Email;

use crate::articulos::{ArticuloId, Nid, PrecioError, PrecioVenta};
use crate::categorias::CategoriaId;
use crate::unidades_medida::UnidadMedidaId;
use search::{SearchQuery, SearchResults};

/// Largos máximos, en caracteres, de los textos opcionales (decididos por el dueño el 2026-10-01;
/// en la v1 lo más largo es 92 en color y 286 en descripción). La base los repite con un `CHECK`.
pub const MAX_DESCRIPCION: usize = 1000;
pub const MAX_MARCA: usize = 150;
pub const MAX_MODELO: usize = 200;
pub const MAX_COLOR: usize = 150;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Producto {
    pub id: ArticuloId,
    pub nid: Nid,
    pub nombre: String,
    pub categoria_id: CategoriaId,
    /// Su unidad base: en ella se cuentan el stock y el costo.
    pub unidad_medida_id: UnidadMedidaId,
    /// `None` = todavía no tiene precio de venta final (un producto por llegar).
    pub precio_venta: Option<PrecioVenta>,
    pub descripcion: Option<String>,
    pub marca: Option<String>,
    pub modelo: Option<String>,
    pub color: Option<String>,
}

/// Lo que se captura de un producto, tal como viene del formulario.
#[derive(Debug, Clone, Copy)]
pub struct ProductoFields<'a> {
    pub nombre: &'a str,
    pub categoria_id: CategoriaId,
    pub unidad_medida_id: UnidadMedidaId,
    /// Vacío = sin precio todavía.
    pub precio_venta: &'a str,
    pub descripcion: &'a str,
    pub marca: &'a str,
    pub modelo: &'a str,
    pub color: &'a str,
}

/// Un producto por agregar, ya validado: solo se construye con `new`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NewProducto {
    nombre: String,
    categoria_id: CategoriaId,
    unidad_medida_id: UnidadMedidaId,
    precio_venta: Option<PrecioVenta>,
    descripcion: Option<String>,
    marca: Option<String>,
    modelo: Option<String>,
    color: Option<String>,
}

impl NewProducto {
    /// Quita los espacios de las orillas de cada texto; un texto opcional vacío queda en `None`.
    /// El nombre es obligatorio y de hasta 150 caracteres. El nombre se guarda como se escribió,
    /// sin cambiar mayúsculas (la v1 los tiene en mayúsculas, y así se quedan).
    pub fn new(fields: ProductoFields<'_>) -> Result<Self, ProductoError> {
        let nombre = fields.nombre.trim();
        if nombre.is_empty() {
            return Err(ProductoError::EmptyNombre);
        }
        if nombre.chars().count() > MAX_NOMBRE_CATALOGO {
            return Err(ProductoError::LongNombre);
        }
        let precio_venta = match fields.precio_venta.trim() {
            "" => None,
            text => Some(PrecioVenta::parse(text).map_err(ProductoError::Precio)?),
        };
        Ok(Self {
            nombre: nombre.to_string(),
            categoria_id: fields.categoria_id,
            unidad_medida_id: fields.unidad_medida_id,
            precio_venta,
            descripcion: optional(
                fields.descripcion,
                MAX_DESCRIPCION,
                ProductoError::LongDescripcion,
            )?,
            marca: optional(fields.marca, MAX_MARCA, ProductoError::LongMarca)?,
            modelo: optional(fields.modelo, MAX_MODELO, ProductoError::LongModelo)?,
            color: optional(fields.color, MAX_COLOR, ProductoError::LongColor)?,
        })
    }

    pub fn nombre(&self) -> &str {
        &self.nombre
    }

    pub fn categoria_id(&self) -> CategoriaId {
        self.categoria_id
    }

    pub fn unidad_medida_id(&self) -> UnidadMedidaId {
        self.unidad_medida_id
    }

    pub fn precio_venta(&self) -> Option<PrecioVenta> {
        self.precio_venta
    }

    pub fn descripcion(&self) -> Option<&str> {
        self.descripcion.as_deref()
    }

    pub fn marca(&self) -> Option<&str> {
        self.marca.as_deref()
    }

    pub fn modelo(&self) -> Option<&str> {
        self.modelo.as_deref()
    }

    pub fn color(&self) -> Option<&str> {
        self.color.as_deref()
    }
}

fn optional(
    text: &str,
    max: usize,
    too_long: ProductoError,
) -> Result<Option<String>, ProductoError> {
    match text.trim() {
        "" => Ok(None),
        text if text.chars().count() > max => Err(too_long),
        text => Ok(Some(text.to_string())),
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ProductoError {
    EmptyNombre,
    /// Más de `MAX_NOMBRE_CATALOGO` caracteres.
    LongNombre,
    /// Más de `MAX_DESCRIPCION` caracteres; igual las tres siguientes con su máximo.
    LongDescripcion,
    LongMarca,
    LongModelo,
    LongColor,
    Precio(PrecioError),
    /// La categoría ya no existe (alguien la cambió mientras tanto, o el id no es de ninguna).
    CategoriaNotFound,
    /// La unidad ya no existe.
    UnidadMedidaNotFound,
    /// Quien lo da de alta ya no existe (su sesión es de un usuario que no está en la base).
    UsuarioNotFound,
    Repo(RepoError),
}

impl fmt::Display for ProductoError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::EmptyNombre => write!(f, "Escribe el nombre del producto."),
            Self::LongNombre => write!(
                f,
                "El nombre no puede pasar de {MAX_NOMBRE_CATALOGO} caracteres."
            ),
            Self::LongDescripcion => write!(
                f,
                "La descripción no puede pasar de {MAX_DESCRIPCION} caracteres."
            ),
            Self::LongMarca => write!(f, "La marca no puede pasar de {MAX_MARCA} caracteres."),
            Self::LongModelo => write!(f, "El modelo no puede pasar de {MAX_MODELO} caracteres."),
            Self::LongColor => write!(f, "El color no puede pasar de {MAX_COLOR} caracteres."),
            Self::Precio(error) => error.fmt(f),
            Self::CategoriaNotFound => write!(f, "Esa categoría ya no existe. Recarga la página."),
            Self::UnidadMedidaNotFound => write!(f, "Esa unidad ya no existe. Recarga la página."),
            Self::UsuarioNotFound => write!(f, "Tu sesión ya no es válida. Vuelve a entrar."),
            Self::Repo(_) => write!(f, "No se pudo guardar el producto."),
        }
    }
}

impl From<RepoError> for ProductoError {
    fn from(error: RepoError) -> Self {
        Self::Repo(error)
    }
}

/// Dónde viven los productos. Lo implementa `db`; para pruebas, `in_memory`.
pub trait ProductosRepo {
    /// Todos, en orden alfabético; los de nombre repetido, por NID.
    fn list(&self) -> impl Future<Output = Result<Vec<Producto>, RepoError>> + Send;

    /// Los que coinciden con la búsqueda (sin importar mayúsculas ni acentos; la ñ cuenta como n)
    /// y, si se da, son de esa categoría. Primero el del NID buscado; luego, en el orden de
    /// `list`. Regresa hasta `limit` y cuántos coinciden en total.
    fn search(
        &self,
        search_query: &SearchQuery,
        categoria: Option<CategoriaId>,
        limit: u32,
    ) -> impl Future<Output = Result<SearchResults, RepoError>> + Send;

    /// Lo da de alta con un NID mayor que los anteriores (puede saltarse números: un alta que
    /// falla en Postgres gasta el suyo) y anota quién. Si trae precio, ese precio también
    /// queda en su historial.
    fn add(
        &self,
        new_producto: NewProducto,
        by: &Email,
    ) -> impl Future<Output = Result<Producto, ProductoError>> + Send;
}

#[cfg(any(test, feature = "test-support"))]
pub mod in_memory;

/// Lo que toda implementación de `ProductosRepo` debe cumplir. Corre contra la de memoria
/// (aquí) y contra la de Postgres (en `db`).
#[cfg(any(test, feature = "test-support"))]
#[allow(clippy::unwrap_used)] // código de pruebas
pub mod contract;

#[cfg(test)]
mod tests;
