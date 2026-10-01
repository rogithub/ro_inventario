use super::in_memory::InMemoryCategorias;
use super::*;

#[test]
fn el_nombre_se_guarda_sin_espacios_en_las_orillas() {
    let new_categoria = NewCategoria::new("  Hojas de color  ").unwrap();
    assert_eq!(new_categoria.nombre(), "Hojas de color");
}

#[test]
fn un_nombre_vacio_o_solo_espacios_no_se_acepta() {
    assert_eq!(NewCategoria::new(""), Err(CategoriaError::EmptyNombre));
    assert_eq!(NewCategoria::new("   "), Err(CategoriaError::EmptyNombre));
}

#[test]
fn un_nombre_de_150_caracteres_se_acepta_aunque_lleve_enies() {
    let nombre = "ñ".repeat(150);
    assert_eq!(NewCategoria::new(&nombre).unwrap().nombre(), nombre);
}

#[test]
fn un_nombre_de_151_caracteres_no_se_acepta() {
    assert_eq!(
        NewCategoria::new(&"a".repeat(151)),
        Err(CategoriaError::LongNombre)
    );
}

#[test]
fn los_espacios_de_las_orillas_no_cuentan_para_el_largo() {
    let nombre = format!("  {}  ", "a".repeat(150));
    assert!(NewCategoria::new(&nombre).is_ok());
}

#[tokio::test]
async fn en_memoria_cumple_el_contrato_agregada_aparece_en_la_lista() {
    contract::agregada_aparece_en_la_lista(&InMemoryCategorias::default()).await;
}

#[tokio::test]
async fn en_memoria_cumple_el_contrato_nombre_repetido() {
    contract::nombre_repetido_se_rechaza_sin_importar_mayusculas(&InMemoryCategorias::default())
        .await;
}

#[tokio::test]
async fn en_memoria_cumple_el_contrato_orden_alfabetico() {
    contract::la_lista_va_en_orden_alfabetico(&InMemoryCategorias::default()).await;
}

#[tokio::test]
async fn en_memoria_cumple_el_contrato_orden_como_postgres() {
    contract::la_lista_ordena_como_postgres_acentos_espacios_y_signos(
        &InMemoryCategorias::default(),
    )
    .await;
}

#[tokio::test]
async fn en_memoria_cumple_el_contrato_trae_su_id() {
    contract::agregada_trae_su_id_y_la_lista_lo_conserva(&InMemoryCategorias::default()).await;
}

#[tokio::test]
async fn en_memoria_cumple_el_contrato_renombrar_cambia_el_nombre() {
    contract::renombrar_cambia_el_nombre_y_conserva_el_id(&InMemoryCategorias::default()).await;
}

#[tokio::test]
async fn en_memoria_cumple_el_contrato_renombrar_a_uno_que_ya_existe() {
    contract::renombrar_a_un_nombre_que_ya_existe_se_rechaza(&InMemoryCategorias::default()).await;
}

#[tokio::test]
async fn en_memoria_cumple_el_contrato_renombrar_solo_mayusculas() {
    contract::renombrar_cambiando_solo_mayusculas_se_permite(&InMemoryCategorias::default()).await;
}

#[tokio::test]
async fn en_memoria_cumple_el_contrato_renombrar_una_que_no_existe() {
    contract::renombrar_una_que_no_existe_avisa(&InMemoryCategorias::default()).await;
}

#[tokio::test]
async fn en_memoria_cumple_el_contrato_renombrar_una_que_no_existe_con_nombre_ocupado() {
    contract::renombrar_una_que_no_existe_avisa_aunque_el_nombre_este_ocupado(
        &InMemoryCategorias::default(),
    )
    .await;
}

#[test]
fn el_id_se_lee_de_su_texto_y_un_texto_que_no_es_id_no() {
    let id = CategoriaId(Uuid::from_u128(7));
    assert_eq!(CategoriaId::parse(&id.to_string()), Some(id));
    assert_eq!(CategoriaId::parse("plumas"), None);
}
