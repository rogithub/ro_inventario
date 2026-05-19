use std::fmt::Display;

/// Formato monetario: dos decimales fijos. Uso en template: {{ valor|pesos }}
/// Genérico porque Askama pasa &Decimal para campos y Decimal para resultados de métodos.
pub fn pesos<T: Display>(value: T) -> askama::Result<String> {
    Ok(format!("{:.2}", value))
}
