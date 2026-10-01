//! Reglas comunes a los nombres del catálogo (categorías, unidades, artículos…).

/// Largo máximo de un nombre del catálogo, en caracteres (no bytes). El nombre más largo de la v1
/// tiene 68 (medido en la copia de la v1, 2026-10-01). La base lo repite con un `CHECK`.
pub const MAX_NOMBRE_CATALOGO: usize = 150;

/// Llave para ordenar nombres como Postgres con la collation `en_US.utf8` (la de desarrollo, CI y
/// producción): primero sin mayúsculas, acentos, espacios ni signos ("Hojas a" antes que
/// "Hoja z"; la ñ como n); si empatan, el texto en minúsculas ("albumes" antes que "Álbumes").
/// Es una aproximación para las implementaciones en memoria: solo conoce los acentos del español
/// (y unos pocos más); otras letras, como ã o ø, quedan al final. La verdad es Postgres, y el
/// contrato de cada repositorio compara las dos.
pub fn sort_key(nombre: &str) -> (String, String) {
    let lower = nombre.to_lowercase();
    let base = lower
        .chars()
        .filter(|c| c.is_alphanumeric())
        .map(without_accent)
        .collect();
    (base, lower)
}

/// El texto como lo compara una búsqueda: en minúsculas y sin acentos («Piña» queda «pina»).
/// Misma aproximación que `sort_key`, para las implementaciones en memoria: en Postgres la
/// búsqueda usa `unaccent(lower(…))`, que es la verdad.
pub fn fold_for_search(text: &str) -> String {
    text.to_lowercase().chars().map(without_accent).collect()
}

fn without_accent(c: char) -> char {
    match c {
        'á' | 'à' | 'ä' | 'â' => 'a',
        'é' | 'è' | 'ë' | 'ê' => 'e',
        'í' | 'ì' | 'ï' | 'î' => 'i',
        'ó' | 'ò' | 'ö' | 'ô' => 'o',
        'ú' | 'ù' | 'ü' | 'û' => 'u',
        'ñ' => 'n',
        'ç' => 'c',
        _ => c,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sorted(nombres: &[&str]) -> Vec<String> {
        let mut nombres: Vec<String> = nombres.iter().map(|n| n.to_string()).collect();
        nombres.sort_by_key(|n| sort_key(n));
        nombres
    }

    #[test]
    fn los_acentos_y_las_mayusculas_no_cuentan_para_el_orden() {
        assert_eq!(
            sorted(&["Zeta", "Útiles", "alfa"]),
            ["alfa", "Útiles", "Zeta"]
        );
    }

    #[test]
    fn la_enie_se_ordena_como_n() {
        assert_eq!(sorted(&["nube", "ñandú"]), ["ñandú", "nube"]);
    }

    #[test]
    fn los_espacios_y_signos_no_cuentan_para_el_orden() {
        assert_eq!(
            sorted(&["Hoja z", "coop", "Hojas a", "co-op", "cob"]),
            ["cob", "co-op", "coop", "Hojas a", "Hoja z"]
        );
    }

    #[test]
    fn para_buscar_se_quitan_mayusculas_y_acentos_y_la_enie_queda_como_n() {
        assert_eq!(fold_for_search("PIÑA Lápiz Ü"), "pina lapiz u");
    }

    #[test]
    fn si_empatan_va_primero_el_que_no_lleva_acento() {
        assert_eq!(sorted(&["Álbumes", "albumes"]), ["albumes", "Álbumes"]);
    }
}
