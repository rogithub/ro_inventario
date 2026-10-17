use super::*;

#[test]
fn un_codigo_se_guarda_exacto_con_mayusculas_y_minusculas() {
    assert_eq!(
        CodigoBarras::parse("EDaq89393").unwrap().as_str(),
        "EDaq89393"
    );
}

#[test]
fn un_codigo_no_tiene_que_ser_numerico() {
    for text in ["7501015200964", "QGJ01", "perm0#?rojo", "GF2411L'78"] {
        assert_eq!(CodigoBarras::parse(text).unwrap().as_str(), text);
    }
}

#[test]
fn los_espacios_de_las_orillas_se_quitan() {
    assert_eq!(
        CodigoBarras::parse(" oaNi38386 ").unwrap().as_str(),
        "oaNi38386"
    );
}

#[test]
fn un_codigo_vacio_se_rechaza() {
    for text in ["", "   "] {
        assert_eq!(CodigoBarras::parse(text), Err(CodigoBarrasError::Empty));
    }
}

#[test]
fn un_codigo_con_espacios_en_medio_se_rechaza() {
    assert_eq!(
        CodigoBarras::parse("750 1015"),
        Err(CodigoBarrasError::HasSpaces)
    );
}

#[test]
fn un_codigo_no_pasa_de_50_caracteres() {
    assert!(CodigoBarras::parse(&"7".repeat(50)).is_ok());
    assert_eq!(
        CodigoBarras::parse(&"7".repeat(51)),
        Err(CodigoBarrasError::TooLong)
    );
}
