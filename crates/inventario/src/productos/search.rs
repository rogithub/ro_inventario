//! Lo que se escribe en el buscador de productos, ya limpio. Cómo se compara con los nombres lo
//! decide cada repositorio (en Postgres, `unaccent`); aquí solo se separa en words y se ve si
//! es un NID.

use super::Producto;
use crate::articulos::Nid;

/// Cuántos productos muestra una búsqueda como máximo; para ver otros, se afina lo escrito.
pub const MAX_RESULTS: u32 = 100;

/// Cuántas palabras cuentan como máximo; las demás se ignoran. Ningún nombre las necesita, y
/// cada una es una comparación más por producto en la base (decidido por el dueño, 2026-10-01).
pub const MAX_WORDS: usize = 25;

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct SearchQuery {
    words: Vec<String>,
    nid: Option<Nid>,
}

impl SearchQuery {
    /// Separa lo escrito en words (sin espacios de más). Si todo es un NID, también se busca
    /// por él: "40" encuentra el NID 40 y los nombres que dicen "40".
    pub fn new(text: &str) -> Self {
        Self {
            words: text
                .split_whitespace()
                .take(MAX_WORDS)
                .map(str::to_string)
                .collect(),
            nid: Nid::parse(text),
        }
    }

    /// Un producto coincide si su nombre contiene todas, en cualquier orden. Sin words,
    /// coinciden todos.
    pub fn words(&self) -> &[String] {
        &self.words
    }

    /// Un producto con este NID coincide aunque su nombre no, y va primero.
    pub fn nid(&self) -> Option<Nid> {
        self.nid
    }

    pub fn is_empty(&self) -> bool {
        self.words.is_empty()
    }
}

/// Los primeros productos que coinciden y cuántos coinciden en total.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SearchResults {
    pub productos: Vec<Producto>,
    pub total: u64,
}

#[cfg(test)]
mod tests;
