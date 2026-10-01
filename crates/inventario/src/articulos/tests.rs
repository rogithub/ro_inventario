use super::*;

fn precio(text: &str) -> Result<PrecioVenta, PrecioError> {
    PrecioVenta::parse(text)
}

#[test]
fn un_precio_con_centavos_se_acepta_tal_cual() {
    assert_eq!(precio("12.50").unwrap().value(), Decimal::new(1250, 2));
    assert_eq!(precio(" 12 ").unwrap().value(), Decimal::new(12, 0));
}

#[test]
fn un_precio_con_fracciones_de_centavo_se_rechaza_en_lugar_de_redondearlo() {
    assert_eq!(precio("12.345"), Err(PrecioError::TooManyDecimals));
}

#[test]
fn los_ceros_de_mas_a_la_derecha_no_cuentan_como_decimales() {
    assert_eq!(precio("12.5000").unwrap().value(), Decimal::new(125, 1));
}

#[test]
fn un_precio_en_cero_o_negativo_se_rechaza() {
    assert_eq!(precio("0"), Err(PrecioError::NotPositive));
    assert_eq!(precio("0.00"), Err(PrecioError::NotPositive));
    assert_eq!(precio("-5"), Err(PrecioError::NotPositive));
}

#[test]
fn un_precio_que_no_es_numero_se_rechaza() {
    for text in ["", "doce", "1,200", "$12"] {
        assert_eq!(precio(text), Err(PrecioError::NotANumber), "«{text}»");
    }
}

#[test]
fn el_precio_mayor_es_el_que_cabe_en_la_base() {
    assert_eq!(PrecioVenta::MAX.to_string(), "9999999999.99");
    assert!(precio("9999999999.99").is_ok());
    assert_eq!(precio("10000000000"), Err(PrecioError::TooBig));
}

#[test]
fn el_id_se_lee_de_su_texto_y_un_texto_que_no_es_id_no() {
    let id = ArticuloId(Uuid::from_u128(7));
    assert_eq!(ArticuloId::parse(&id.to_string()), Some(id));
    assert_eq!(ArticuloId::parse("40"), None);
}

#[test]
fn un_precio_con_demasiados_decimales_se_rechaza_aunque_no_quepan_en_un_decimal() {
    assert_eq!(
        precio("12.3399999999999999999999999999999"),
        Err(PrecioError::TooManyDecimals)
    );
}

#[test]
fn un_precio_con_guion_bajo_o_notacion_cientifica_se_rechaza() {
    for text in ["1_200", "1e3", "1E3", "12.5.0", "."] {
        assert_eq!(precio(text), Err(PrecioError::NotANumber), "«{text}»");
    }
}

#[test]
fn el_precio_se_guarda_con_dos_decimales() {
    assert_eq!(precio("12.5000").unwrap().value().to_string(), "12.50");
    assert_eq!(precio("12").unwrap().value().to_string(), "12.00");
}

#[test]
fn un_nid_escrito_con_digitos_se_lee() {
    assert_eq!(Nid::parse("40"), Some(Nid(40)));
    assert_eq!(Nid::parse(" 2337 "), Some(Nid(2337)));
}

#[test]
fn un_nid_en_cero_o_negativo_no_es_nid() {
    for text in ["0", "000", "-5", "+5"] {
        assert_eq!(Nid::parse(text), None, "«{text}»");
    }
}

#[test]
fn un_texto_o_un_numero_que_no_cabe_no_es_nid() {
    for text in ["", "lapiz", "40a", "4.0", "2147483648"] {
        assert_eq!(Nid::parse(text), None, "«{text}»");
    }
}
