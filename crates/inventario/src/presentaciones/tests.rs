use super::*;

fn factor(text: &str) -> Result<Factor, FactorError> {
    Factor::new(Decimal::from_str_exact(text).unwrap())
}

fn paquete_de_20() -> Factor {
    factor("20").unwrap()
}

#[test]
fn un_factor_de_mas_de_una_unidad_se_acepta() {
    assert_eq!(factor("20").unwrap().value(), Decimal::new(20, 0));
    assert_eq!(factor("1.5").unwrap().value(), Decimal::new(15, 1));
}

#[test]
fn un_factor_de_una_unidad_o_menos_se_rechaza_porque_seria_la_pieza_suelta() {
    for text in ["1", "1.000", "0.5", "0", "-20"] {
        assert_eq!(factor(text), Err(FactorError::NotMoreThanOne), "{text}");
    }
}

#[test]
fn un_factor_con_mas_de_tres_decimales_se_rechaza_en_lugar_de_redondearlo() {
    assert_eq!(factor("2.5001"), Err(FactorError::TooManyDecimals));
}

#[test]
fn los_ceros_de_mas_a_la_derecha_no_cuentan_como_decimales_del_factor() {
    assert_eq!(factor("100.0000").unwrap().value(), Decimal::new(100, 0));
}

#[test]
fn el_factor_mayor_es_el_que_cabe_en_la_base() {
    assert_eq!(Factor::MAX.to_string(), "999999999.999");
    assert!(factor("999999999.999").is_ok());
    assert_eq!(factor("1000000000"), Err(FactorError::TooBig));
}

#[test]
fn una_presentacion_guarda_el_nombre_sin_espacios_en_las_orillas() {
    let precio = PrecioVenta::parse("10").unwrap();

    let presentacion =
        NewPresentacion::new("  PAQUETE DE 20 HOJAS ", paquete_de_20(), Some(precio)).unwrap();

    assert_eq!(presentacion.nombre(), "PAQUETE DE 20 HOJAS");
    assert_eq!(presentacion.factor(), paquete_de_20());
    assert_eq!(presentacion.precio_venta(), Some(precio));
}

#[test]
fn una_presentacion_sin_precio_se_acepta_porque_puede_ser_solo_para_comprar() {
    let presentacion = NewPresentacion::new("Caja con 5000", paquete_de_20(), None).unwrap();

    assert_eq!(presentacion.precio_venta(), None);
}

#[test]
fn una_presentacion_sin_nombre_se_rechaza() {
    assert_eq!(
        NewPresentacion::new("   ", paquete_de_20(), None),
        Err(PresentacionError::EmptyNombre)
    );
}

#[test]
fn el_nombre_de_una_presentacion_no_pasa_de_150_caracteres() {
    let de_150 = "ñ".repeat(150);
    let de_151 = "ñ".repeat(151);

    assert!(NewPresentacion::new(&de_150, paquete_de_20(), None).is_ok());
    assert_eq!(
        NewPresentacion::new(&de_151, paquete_de_20(), None),
        Err(PresentacionError::LongNombre)
    );
}
