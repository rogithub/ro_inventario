//! Descontinuar un producto: ya no se maneja, pero su historia se conserva (diseño 01).

use kernel::{DateTime, Utc};

use super::ProductoError;

/// Largo máximo del motivo, en caracteres (decidido por el dueño el 2026-10-07). La base lo
/// repite con un `CHECK`.
pub const MAX_MOTIVO_DESCONTINUADO: usize = 300;

/// Desde cuándo y por qué ya no se maneja.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Descontinuado {
    pub at: DateTime<Utc>,
    pub motivo: String,
}

/// Por qué se descontinúa, ya validado: solo se construye con `parse`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MotivoDescontinuado(String);

impl MotivoDescontinuado {
    /// Quita los espacios de las orillas. Es obligatorio y de hasta 300 caracteres.
    pub fn parse(text: &str) -> Result<Self, ProductoError> {
        let motivo = text.trim();
        if motivo.is_empty() {
            return Err(ProductoError::EmptyMotivo);
        }
        if motivo.chars().count() > MAX_MOTIVO_DESCONTINUADO {
            return Err(ProductoError::LongMotivo);
        }
        Ok(Self(motivo.to_string()))
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn el_motivo_se_guarda_sin_espacios_en_las_orillas() {
        let motivo = MotivoDescontinuado::parse("  el proveedor ya no lo vende ").unwrap();

        assert_eq!(motivo.as_str(), "el proveedor ya no lo vende");
    }

    #[test]
    fn sin_motivo_no_se_puede_descontinuar() {
        assert_eq!(
            MotivoDescontinuado::parse(""),
            Err(ProductoError::EmptyMotivo)
        );
        assert_eq!(
            MotivoDescontinuado::parse("   "),
            Err(ProductoError::EmptyMotivo)
        );
    }

    #[test]
    fn un_motivo_de_300_caracteres_se_acepta_y_de_301_no() {
        assert!(MotivoDescontinuado::parse(&"ñ".repeat(300)).is_ok());
        assert_eq!(
            MotivoDescontinuado::parse(&"ñ".repeat(301)),
            Err(ProductoError::LongMotivo)
        );
    }
}
