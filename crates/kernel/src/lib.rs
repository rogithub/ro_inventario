//! Tipos que comparten todas las áreas del negocio: dinero, cantidades, identificadores, errores.
//!
//! No depende de ningún otro crate: una consulta SQL o una respuesta HTTP aquí no compila.

// El dinero y las cantidades son `Decimal`, nunca flotantes (docs/decisiones/2026-09-29-formato-y-linters.md).
#![deny(clippy::float_arithmetic)]
