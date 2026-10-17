//! Códigos de barras: los impresos en un producto o en su empaque (diseño 01). Un producto puede
//! tener varios; cada código es de un solo producto, o de una de sus presentaciones. El código
//! generado del NID no se guarda: la etiqueta imprime el NID y al escanearlo se busca por NID.

use std::fmt;

/// Largo máximo de un código, en caracteres. En la v1 el más largo tiene 24 (medido en la copia de
/// la v1, 2026-10-17). La base lo repite con un `CHECK`.
pub const MAX_CODIGO_BARRAS: usize = 50;

/// Un código de barras ya validado: sin espacios y de hasta 50 caracteres. Se guarda exacto, con
/// mayúsculas y minúsculas, como lo manda el escáner (dueño, 2026-10-17). No se exige que sea
/// numérico: los proveedores traen códigos como «QGJ01», y en la v1 hay capturados a mano como
/// «perm0#?rojo». Solo se construye con `parse`.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct CodigoBarras(String);

impl CodigoBarras {
    /// Quita los espacios de las orillas (en la v1 hay dos con un espacio al final); uno en medio
    /// se rechaza: el escáner no los manda, así que es un error de captura.
    pub fn parse(text: &str) -> Result<Self, CodigoBarrasError> {
        let text = text.trim();
        if text.is_empty() {
            return Err(CodigoBarrasError::Empty);
        }
        if text.chars().any(char::is_whitespace) {
            return Err(CodigoBarrasError::HasSpaces);
        }
        if text.chars().count() > MAX_CODIGO_BARRAS {
            return Err(CodigoBarrasError::TooLong);
        }
        Ok(Self(text.to_string()))
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl fmt::Display for CodigoBarras {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.0.fmt(f)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CodigoBarrasError {
    Empty,
    /// Lleva espacios en medio.
    HasSpaces,
    /// Más de `MAX_CODIGO_BARRAS` caracteres.
    TooLong,
}

impl fmt::Display for CodigoBarrasError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Empty => write!(f, "Escribe o escanea el código de barras."),
            Self::HasSpaces => write!(f, "El código de barras no lleva espacios."),
            Self::TooLong => write!(
                f,
                "El código de barras no puede pasar de {MAX_CODIGO_BARRAS} caracteres."
            ),
        }
    }
}

#[cfg(test)]
mod tests;
