//! Presentaciones: formas de vender (o comprar) un producto en grupo, sobre todo para el mayoreo
//! ("Paquete de 100 hojas", diseño 01). El precio y el stock se cuentan en la unidad base del
//! producto; el factor dice cuántas trae.

use std::fmt;

use kernel::nombres::MAX_NOMBRE_CATALOGO;
use kernel::{Decimal, Uuid};

use crate::articulos::PrecioVenta;

/// El id de una presentación: no se confunde con el de otra tabla.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct PresentacionId(pub Uuid);

impl fmt::Display for PresentacionId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.0.fmt(f)
    }
}

/// Cuántas unidades base trae una presentación, ya validado: más de una (una presentación de 1
/// sería la pieza suelta; dueño, 2026-10-17), con 3 decimales como máximo y que cabe en la
/// columna (`numeric(12,3)`). Solo se construye con `new`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct Factor(Decimal);

impl Factor {
    /// El mayor que cabe en `numeric(12,3)`: 999,999,999.999.
    pub const MAX: Decimal = Decimal::from_parts(3_567_587_327, 232, 0, false, 3);

    /// Rechaza en lugar de redondear, como el precio.
    pub fn new(factor: Decimal) -> Result<Self, FactorError> {
        if factor <= Decimal::ONE {
            return Err(FactorError::NotMoreThanOne);
        }
        if factor.normalize().scale() > 3 {
            return Err(FactorError::TooManyDecimals);
        }
        if factor > Self::MAX {
            return Err(FactorError::TooBig);
        }
        Ok(Self(factor.normalize()))
    }

    pub fn value(&self) -> Decimal {
        self.0
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FactorError {
    /// 1 o menos.
    NotMoreThanOne,
    /// Más de 3 decimales.
    TooManyDecimals,
    /// Más de `Factor::MAX`.
    TooBig,
}

impl fmt::Display for FactorError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::NotMoreThanOne => write!(f, "Una presentación trae más de una unidad."),
            Self::TooManyDecimals => {
                write!(
                    f,
                    "Las unidades que trae llevan como máximo tres decimales."
                )
            }
            Self::TooBig => write!(f, "Son demasiadas unidades para una presentación."),
        }
    }
}

/// Los datos de una presentación para darla de alta, ya validados: solo se construye con `new`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NewPresentacion {
    nombre: String,
    factor: Factor,
    precio_venta: Option<PrecioVenta>,
}

impl NewPresentacion {
    /// Quita los espacios de las orillas del nombre, obligatorio y de hasta 150 caracteres.
    /// Sin precio = solo se usa para comprar.
    pub fn new(
        nombre: &str,
        factor: Factor,
        precio_venta: Option<PrecioVenta>,
    ) -> Result<Self, PresentacionError> {
        let nombre = nombre.trim();
        if nombre.is_empty() {
            return Err(PresentacionError::EmptyNombre);
        }
        if nombre.chars().count() > MAX_NOMBRE_CATALOGO {
            return Err(PresentacionError::LongNombre);
        }
        Ok(Self {
            nombre: nombre.to_string(),
            factor,
            precio_venta,
        })
    }

    pub fn nombre(&self) -> &str {
        &self.nombre
    }

    pub fn factor(&self) -> Factor {
        self.factor
    }

    pub fn precio_venta(&self) -> Option<PrecioVenta> {
        self.precio_venta
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PresentacionError {
    EmptyNombre,
    /// Más de `MAX_NOMBRE_CATALOGO` caracteres.
    LongNombre,
}

impl fmt::Display for PresentacionError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::EmptyNombre => write!(f, "Escribe el nombre de la presentación."),
            Self::LongNombre => write!(
                f,
                "El nombre no puede pasar de {MAX_NOMBRE_CATALOGO} caracteres."
            ),
        }
    }
}

#[cfg(test)]
mod tests;
