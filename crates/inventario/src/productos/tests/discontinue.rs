//! Descontinuar y volver a manejar, en memoria: el contrato.

use super::*;

#[tokio::test]
async fn en_memoria_cumple_el_contrato_descontinuar_anota_la_fecha_y_el_motivo_y_se_ve_al_buscarlo()
{
    let (repo, setup) = in_memory_repo();
    contract::discontinue::descontinuar_anota_la_fecha_y_el_motivo_y_se_ve_al_buscarlo(
        &repo, &setup,
    )
    .await;
}

#[tokio::test]
async fn en_memoria_cumple_el_contrato_uno_ya_descontinuado_no_se_vuelve_a_descontinuar_y_conserva_su_fecha()
 {
    let (repo, setup) = in_memory_repo();
    contract::discontinue::uno_ya_descontinuado_no_se_vuelve_a_descontinuar_y_conserva_su_fecha(
        &repo, &setup,
    )
    .await;
}

#[tokio::test]
async fn en_memoria_cumple_el_contrato_volver_a_manejar_borra_la_fecha_y_el_motivo() {
    let (repo, setup) = in_memory_repo();
    contract::discontinue::volver_a_manejar_borra_la_fecha_y_el_motivo(&repo, &setup).await;
}

#[tokio::test]
async fn en_memoria_cumple_el_contrato_volver_a_manejar_uno_que_se_maneja_no_cambia_nada() {
    let (repo, setup) = in_memory_repo();
    contract::discontinue::volver_a_manejar_uno_que_se_maneja_no_cambia_nada(&repo, &setup).await;
}

#[tokio::test]
async fn en_memoria_cumple_el_contrato_descontinuar_o_volver_a_manejar_uno_que_no_existe_se_rechaza()
 {
    let (repo, setup) = in_memory_repo();
    contract::discontinue::descontinuar_o_volver_a_manejar_uno_que_no_existe_se_rechaza(
        &repo, &setup,
    )
    .await;
}

#[tokio::test]
async fn en_memoria_cumple_el_contrato_con_un_usuario_que_no_existe_se_rechaza_sin_cambiar_nada() {
    let (repo, setup) = in_memory_repo();
    contract::discontinue::con_un_usuario_que_no_existe_se_rechaza_sin_cambiar_nada(&repo, &setup)
        .await;
}

#[tokio::test]
async fn en_memoria_cumple_el_contrato_editar_un_descontinuado_lo_deja_descontinuado() {
    let (repo, setup) = in_memory_repo();
    contract::discontinue::editar_un_descontinuado_lo_deja_descontinuado(&repo, &setup).await;
}
