//! El formulario de un producto (plantilla parcial `productos/form.html`): lo usa el alta del
//! catálogo y lo va a usar el alta desde compras, con otra `action`.

use inventario::categorias::{Categoria, CategoriaId};
use inventario::productos::{ProductoError, ProductoFields};
use inventario::unidades_medida::{UnidadMedida, UnidadMedidaId};
use serde::Deserialize;

/// Lo escrito en el formulario, tal cual llega: para validarlo y para no perderlo si hay error.
#[derive(Deserialize, Default)]
#[serde(default)]
pub struct ProductoForm {
    pub nombre: String,
    /// El id de la categoría; vacío = no se eligió.
    pub categoria: String,
    /// El id de la unidad.
    pub unidad: String,
    /// Vacío = sin precio todavía.
    pub precio: String,
    pub descripcion: String,
    pub marca: String,
    pub modelo: String,
    pub color: String,
}

impl ProductoForm {
    /// Los campos para el área. Un id que no es id cuenta como una categoría o unidad que ya no
    /// existe (el formulario solo manda ids de la lista).
    pub fn fields(&self) -> Result<ProductoFields<'_>, ProductoError> {
        let categoria_id = match self.categoria.trim() {
            "" => None,
            text => Some(CategoriaId::parse(text).ok_or(ProductoError::CategoriaNotFound)?),
        };
        let unidad_medida_id =
            UnidadMedidaId::parse(&self.unidad).ok_or(ProductoError::UnidadMedidaNotFound)?;
        Ok(ProductoFields {
            nombre: &self.nombre,
            categoria_id,
            unidad_medida_id,
            precio_venta: &self.precio,
            descripcion: &self.descripcion,
            marca: &self.marca,
            modelo: &self.modelo,
            color: &self.color,
        })
    }

    /// Después de un alta: vacío para el siguiente, con la misma categoría y unidad (se suelen
    /// capturar varios parecidos seguidos).
    pub fn next(self) -> Self {
        Self {
            categoria: self.categoria,
            unidad: self.unidad,
            ..Self::default()
        }
    }
}

/// El mensaje junto al campo que lo causó; el que no es de un campo va arriba (`general`).
#[derive(Default)]
pub struct FieldErrors {
    pub nombre: Option<String>,
    pub categoria: Option<String>,
    pub unidad: Option<String>,
    pub precio: Option<String>,
    pub descripcion: Option<String>,
    pub marca: Option<String>,
    pub modelo: Option<String>,
    pub color: Option<String>,
    pub general: Option<String>,
}

impl FieldErrors {
    pub fn from_error(error: &ProductoError) -> Self {
        let message = Some(error.to_string());
        match error {
            ProductoError::EmptyNombre | ProductoError::LongNombre => Self {
                nombre: message,
                ..Self::default()
            },
            ProductoError::MissingCategoria | ProductoError::CategoriaNotFound => Self {
                categoria: message,
                ..Self::default()
            },
            ProductoError::UnidadMedidaNotFound => Self {
                unidad: message,
                ..Self::default()
            },
            ProductoError::Precio(_) => Self {
                precio: message,
                ..Self::default()
            },
            ProductoError::LongDescripcion => Self {
                descripcion: message,
                ..Self::default()
            },
            ProductoError::LongMarca => Self {
                marca: message,
                ..Self::default()
            },
            ProductoError::LongModelo => Self {
                modelo: message,
                ..Self::default()
            },
            ProductoError::LongColor => Self {
                color: message,
                ..Self::default()
            },
            ProductoError::UsuarioNotFound | ProductoError::Repo(_) => Self {
                general: message,
                ..Self::default()
            },
        }
    }
}

/// Lo que pinta el parcial.
pub struct ProductoFormView {
    /// A dónde se manda.
    pub action: &'static str,
    pub categorias: Vec<Categoria>,
    pub unidades: Vec<UnidadMedida>,
    pub values: ProductoForm,
    pub errors: FieldErrors,
    /// El aviso del alta que se acaba de hacer.
    pub added: Option<String>,
}

impl ProductoFormView {
    /// «Más datos» se abre si algo de adentro trae valor o error: que un error no quede escondido.
    pub fn is_more_open(&self) -> bool {
        let v = &self.values;
        let e = &self.errors;
        [&v.descripcion, &v.marca, &v.modelo, &v.color]
            .iter()
            .any(|text| !text.trim().is_empty())
            || [&e.descripcion, &e.marca, &e.modelo, &e.color]
                .iter()
                .any(|error| error.is_some())
    }

    pub fn is_categoria(&self, id: &CategoriaId) -> bool {
        self.values.categoria == id.to_string()
    }

    pub fn is_unidad(&self, id: &UnidadMedidaId) -> bool {
        self.values.unidad == id.to_string()
    }
}
