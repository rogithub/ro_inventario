use super::*;

#[test]
fn lo_escrito_se_separa_en_palabras_sin_espacios_de_mas() {
    let search_query = SearchQuery::new("  cinta   verde ");
    assert_eq!(search_query.words(), ["cinta", "verde"]);
    assert_eq!(search_query.nid(), None);
}

#[test]
fn solo_espacios_es_una_busqueda_vacia() {
    assert!(SearchQuery::new("   ").is_empty());
    assert!(SearchQuery::default().is_empty());
}

#[test]
fn un_numero_se_busca_como_nid_y_como_palabra() {
    let search_query = SearchQuery::new(" 40 ");
    assert_eq!(search_query.nid(), Some(Nid(40)));
    assert_eq!(search_query.words(), ["40"]);
}

#[test]
fn un_cero_o_un_negativo_solo_se_busca_como_palabra() {
    for text in ["0", "-5"] {
        let search_query = SearchQuery::new(text);
        assert_eq!(search_query.nid(), None, "«{text}»");
        assert_eq!(search_query.words(), [text]);
    }
}

#[test]
fn varias_palabras_no_son_nid_aunque_una_sea_numero() {
    assert_eq!(SearchQuery::new("cuaderno 100").nid(), None);
}

#[test]
fn de_mas_de_25_palabras_solo_cuentan_las_primeras_25() {
    let text: Vec<String> = (1..=30).map(|i| format!("p{i}")).collect();
    let search_query = SearchQuery::new(&text.join(" "));
    assert_eq!(search_query.words(), &text[..25]);
}
