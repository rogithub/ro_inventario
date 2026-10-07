//! Cómo se escriben las fechas en las pantallas. Un instante se guarda en UTC; aquí se pasa a la
//! zona del negocio (negocio.toml), que se eligió al arrancar.

use chrono::Datelike;
use chrono_tz::Tz;
use kernel::{DateTime, Utc};

/// Abreviaturas de los meses, como se escriben en México.
const MONTHS: [&str; 12] = [
    "ene", "feb", "mar", "abr", "may", "jun", "jul", "ago", "sep", "oct", "nov", "dic",
];

/// El día en la zona del negocio: "3 oct 2026".
#[cfg_attr(
    not(test),
    expect(dead_code, reason = "la usa la franja del descontinuado (paso 7c2d)")
)]
pub fn short_date(at: DateTime<Utc>, time_zone: Tz) -> String {
    let local = at.with_timezone(&time_zone);
    let month = MONTHS[local.month0() as usize];
    format!("{} {month} {}", local.day(), local.year())
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::TimeZone;
    use chrono_tz::America::Cancun;

    #[test]
    fn escribe_dia_mes_abreviado_y_anio() {
        let at = Utc.with_ymd_and_hms(2026, 10, 3, 18, 0, 0).unwrap();
        assert_eq!(short_date(at, Cancun), "3 oct 2026");
    }

    #[test]
    fn a_las_dos_de_la_manana_utc_en_cancun_todavia_es_el_dia_anterior() {
        // Cancún está en UTC−5 todo el año: las 2:00 UTC son las 21:00 del día anterior.
        let at = Utc.with_ymd_and_hms(2026, 1, 1, 2, 0, 0).unwrap();
        assert_eq!(short_date(at, Cancun), "31 dic 2025");
    }

    #[test]
    fn cada_mes_tiene_su_abreviatura() {
        let months: Vec<String> = (1..=12)
            .map(|month| {
                let at = Utc.with_ymd_and_hms(2026, month, 15, 18, 0, 0).unwrap();
                short_date(at, Cancun)
                    .replace(" 2026", "")
                    .replace("15 ", "")
            })
            .collect();
        assert_eq!(
            months.join(" "),
            "ene feb mar abr may jun jul ago sep oct nov dic"
        );
    }
}
