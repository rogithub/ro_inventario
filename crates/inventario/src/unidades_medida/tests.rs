use super::in_memory::InMemoryUnidadesMedida;
use super::*;

#[test]
fn el_nombre_se_guarda_sin_espacios_en_las_orillas() {
    let new_unidad = NewUnidadMedida::new("  Hoja carta  ", false).unwrap();
    assert_eq!(new_unidad.nombre(), "Hoja carta");
    assert!(!new_unidad.allows_fraction());
}

#[test]
fn un_nombre_vacio_o_solo_espacios_no_se_acepta() {
    assert_eq!(
        NewUnidadMedida::new("", true),
        Err(UnidadMedidaError::EmptyNombre)
    );
    assert_eq!(
        NewUnidadMedida::new("   ", true),
        Err(UnidadMedidaError::EmptyNombre)
    );
}

#[test]
fn un_nombre_de_150_caracteres_se_acepta_aunque_lleve_enies() {
    let nombre = "ñ".repeat(150);
    assert_eq!(
        NewUnidadMedida::new(&nombre, false).unwrap().nombre(),
        nombre
    );
}

#[test]
fn un_nombre_de_151_caracteres_no_se_acepta() {
    assert_eq!(
        NewUnidadMedida::new(&"a".repeat(151), false),
        Err(UnidadMedidaError::LongNombre)
    );
}

#[test]
fn los_espacios_de_las_orillas_no_cuentan_para_el_largo() {
    let nombre = format!("  {}  ", "a".repeat(150));
    assert!(NewUnidadMedida::new(&nombre, false).is_ok());
}

#[tokio::test]
async fn en_memoria_cumple_el_contrato_agregada_aparece_en_la_lista() {
    contract::agregada_aparece_en_la_lista(&InMemoryUnidadesMedida::default()).await;
}

#[tokio::test]
async fn en_memoria_cumple_el_contrato_nombre_repetido() {
    contract::nombre_repetido_se_rechaza_sin_importar_mayusculas(
        &InMemoryUnidadesMedida::default(),
    )
    .await;
}

#[tokio::test]
async fn en_memoria_cumple_el_contrato_orden_alfabetico() {
    contract::la_lista_va_en_orden_alfabetico(&InMemoryUnidadesMedida::default()).await;
}

#[tokio::test]
async fn en_memoria_cumple_el_contrato_orden_como_postgres() {
    contract::la_lista_ordena_como_postgres_acentos_espacios_y_signos(
        &InMemoryUnidadesMedida::default(),
    )
    .await;
}

#[tokio::test]
async fn en_memoria_cumple_el_contrato_trae_su_id() {
    contract::agregada_trae_su_id_y_la_lista_lo_conserva(&InMemoryUnidadesMedida::default()).await;
}
