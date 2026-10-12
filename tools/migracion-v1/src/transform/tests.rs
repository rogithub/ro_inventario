use super::*;

fn categoria(id: u128, nombre: &str) -> CategoriaV1 {
    CategoriaV1 {
        id: Uuid::from_u128(id),
        nombre: nombre.to_string(),
    }
}

fn nombres(categorias: &Categorias) -> Vec<&str> {
    categorias
        .to_insert
        .iter()
        .map(|(_, c)| c.nombre())
        .collect()
}

#[test]
fn los_espacios_de_mas_se_quitan_y_las_mayusculas_se_quedan() {
    assert_eq!(
        normalize_nombre("  FLORATAPE  VERDE   LIMON "),
        "FLORATAPE VERDE LIMON"
    );
}

#[test]
fn los_espacios_de_las_orillas_no_hacen_otra_categoria() {
    let v1 = [
        categoria(1, "CINTA "),
        categoria(2, "CINTA"),
        categoria(3, "GOMA"),
    ];

    let categorias = categorias(&v1).unwrap();

    assert_eq!(nombres(&categorias), ["CINTA", "GOMA"]);
}

#[test]
fn la_categoria_fusionada_conserva_el_uuid_de_la_que_venia_limpia() {
    let v1 = [categoria(1, "CINTA "), categoria(2, "CINTA")];

    let categorias = categorias(&v1).unwrap();

    let clean = CategoriaId(Uuid::from_u128(2));
    assert_eq!(categorias.to_insert[0].0, clean);
    assert_eq!(categorias.by_v1_id[&Uuid::from_u128(1)], clean);
    assert_eq!(categorias.by_v1_id[&Uuid::from_u128(2)], clean);
}

#[test]
fn una_categoria_sola_con_espacios_conserva_su_uuid_con_el_nombre_limpio() {
    let v1 = [categoria(7, " HOJA ")];

    let categorias = categorias(&v1).unwrap();

    assert_eq!(categorias.to_insert[0].0, CategoriaId(Uuid::from_u128(7)));
    assert_eq!(nombres(&categorias), ["HOJA"]);
}

#[test]
fn si_dos_categorias_iguales_vienen_limpias_no_adivina_cual_conservar() {
    let v1 = [categoria(1, "Cinta"), categoria(2, "CINTA")];

    let errors = categorias(&v1).unwrap_err();

    assert_eq!(
        errors,
        ["Las categorías «Cinta», «CINTA» de la v1 son la misma y no se sabe cuál conservar."]
    );
}

#[test]
fn junta_todos_los_errores_de_las_categorias_antes_de_detenerse() {
    let long_nombre = "Ñ".repeat(151);
    let v1 = [
        categoria(1, "Cinta"),
        categoria(2, "CINTA"),
        categoria(3, &long_nombre),
        categoria(4, "GOMA"),
    ];

    let errors = categorias(&v1).unwrap_err();

    assert_eq!(errors.len(), 2, "{errors:?}");
}
