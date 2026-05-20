use sqlx::PgPool;

use super::models::ClienteBuscado;
use crate::error::AppError;

/// Quita todo lo que no sea dígito y devuelve los últimos 10.
/// Igual que el .NET: `Regex.Replace(telefono, @"\D", "")` + `digits[^10..]`
fn normalizar_telefono(input: &str) -> String {
    let digits: String = input.chars().filter(|c| c.is_ascii_digit()).collect();
    if digits.len() > 10 {
        digits[digits.len() - 10..].to_string()
    } else {
        digits
    }
}

/// Heurística: si ≥ 70 % de los caracteres son dígitos, tratamos el input como teléfono.
fn parece_telefono(input: &str) -> bool {
    let total = input.chars().count();
    if total == 0 {
        return false;
    }
    let digits = input.chars().filter(|c| c.is_ascii_digit()).count();
    digits * 10 >= total * 7 // digits/total >= 0.7
}

pub async fn buscar(pool: &PgPool, q: &str) -> Result<Vec<ClienteBuscado>, AppError> {
    if parece_telefono(q) {
        // Busca por teléfono normalizado: strip no-dígitos en la BD y compara
        let normalized = normalizar_telefono(q);
        return Ok(sqlx::query_as(
            "SELECT id,
                    COALESCE(nombre, '') AS nombre,
                    COALESCE(telefono, '') AS telefono,
                    COALESCE(email, '') AS email
             FROM contactos
             WHERE tipo = 0
               AND REGEXP_REPLACE(COALESCE(telefono,''), '[^0-9]', '', 'g') ILIKE $1
             ORDER BY nombre
             LIMIT 10",
        )
        .bind(format!("%{normalized}%"))
        .fetch_all(pool)
        .await?);
    }

    // Búsqueda por nombre, empresa, email o teléfono con ILIKE
    let pattern = format!("%{q}%");
    Ok(sqlx::query_as(
        "SELECT id,
                COALESCE(nombre, '') AS nombre,
                COALESCE(telefono, '') AS telefono,
                COALESCE(email, '') AS email
         FROM contactos
         WHERE tipo = 0
           AND (
               unaccent(nombre)  ILIKE unaccent($1) OR
               unaccent(empresa) ILIKE unaccent($1) OR
               unaccent(email)   ILIKE unaccent($1) OR
               telefono          ILIKE $1
           )
         ORDER BY nombre
         LIMIT 10",
    )
    .bind(&pattern)
    .fetch_all(pool)
    .await?)
}
