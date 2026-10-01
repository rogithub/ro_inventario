//! Lo común de todo lo que se vende en caja (producto, servicio o kit): su id, su NID y su precio.

use std::fmt;

use kernel::{Decimal, Uuid};

/// El id de un artículo: no se confunde con el de otra tabla.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct ArticuloId(pub Uuid);

impl ArticuloId {
    /// El id escrito en una ruta o un formulario; `None` si no es un uuid.
    pub fn parse(text: &str) -> Option<Self> {
        Uuid::parse_str(text).ok().map(Self)
    }
}

impl fmt::Display for ArticuloId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.0.fmt(f)
    }
}

/// Número corto del artículo: se teclea en caja y se imprime en la etiqueta. Lo asigna la base.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct Nid(pub i32);

impl fmt::Display for Nid {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.0.fmt(f)
    }
}

/// Precio de venta de una unidad, ya validado: mayor que cero, con centavos como máximo y que
/// cabe en la columna (`numeric(12,2)`). Solo se construye con `new`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct PrecioVenta(Decimal);

impl PrecioVenta {
    /// El mayor que cabe en `numeric(12,2)`: 9,999,999,999.99.
    pub const MAX: Decimal = Decimal::from_parts(3_567_587_327, 232, 0, false, 2);

    /// Rechaza en lugar de redondear: un 12.345 es un error de captura, y lo que se cobra se
    /// redondea una sola vez, en el servidor (no aquí ni en la base).
    pub fn new(precio: Decimal) -> Result<Self, PrecioError> {
        if precio <= Decimal::ZERO {
            return Err(PrecioError::NotPositive);
        }
        if precio.normalize().scale() > 2 {
            return Err(PrecioError::TooManyDecimals);
        }
        if precio > Self::MAX {
            return Err(PrecioError::TooBig);
        }
        // Siempre con centavos ("12.5000" y "12.5" quedan "12.50"); no redondea: ya tiene 2 o menos.
        let mut precio = precio.normalize();
        precio.rescale(2);
        Ok(Self(precio))
    }

    /// El precio escrito en un formulario ("12.50", "12"), sin redondear. Solo dígitos, un punto
    /// y quizá un signo: el `parse` de `Decimal` acepta "1_200" y "1e3", y redondea en silencio
    /// lo que no le cabe.
    pub fn parse(text: &str) -> Result<Self, PrecioError> {
        let text = text.trim();
        let unsigned = text.strip_prefix('-').unwrap_or(text);
        let (whole, fraction) = unsigned.split_once('.').unwrap_or((unsigned, ""));
        let is_number = !(whole.is_empty() && fraction.is_empty())
            && whole.bytes().all(|b| b.is_ascii_digit())
            && fraction.bytes().all(|b| b.is_ascii_digit());
        if !is_number {
            return Err(PrecioError::NotANumber);
        }
        if fraction.trim_end_matches('0').len() > 2 {
            return Err(PrecioError::TooManyDecimals);
        }
        // Con 2 decimales o menos, solo falla si la parte entera no cabe en un `Decimal`.
        let precio = Decimal::from_str_exact(text).map_err(|_| PrecioError::TooBig)?;
        Self::new(precio)
    }

    pub fn value(&self) -> Decimal {
        self.0
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PrecioError {
    /// No es un número ("doce", "1,200", "1e3").
    NotANumber,
    /// Cero o negativo.
    NotPositive,
    /// Fracciones de centavo (12.345).
    TooManyDecimals,
    /// Más de `PrecioVenta::MAX`.
    TooBig,
}

impl fmt::Display for PrecioError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::NotANumber => write!(f, "Escribe el precio con números, por ejemplo 12.50."),
            Self::NotPositive => write!(f, "El precio debe ser mayor que cero."),
            Self::TooManyDecimals => write!(f, "El precio lleva como máximo dos decimales."),
            Self::TooBig => write!(f, "El precio es demasiado grande."),
        }
    }
}

#[cfg(test)]
mod tests;
