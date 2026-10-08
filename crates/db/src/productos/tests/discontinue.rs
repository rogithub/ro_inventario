//! Descontinuar y volver a manejar, contra Postgres: el contrato y lo que solo se ve en la base.

use inventario::productos::discontinue::MotivoDescontinuado;
use kernel::{DateTime, Utc};

use super::*;

#[sqlx::test]
async fn cumple_el_contrato_descontinuar_anota_la_fecha_y_el_motivo_y_se_ve_al_buscarlo(
    pool: PgPool,
) {
    let setup = setup(&pool).await;
    contract::discontinue::descontinuar_anota_la_fecha_y_el_motivo_y_se_ve_al_buscarlo(
        &PgProductos::new(pool),
        &setup,
    )
    .await;
}

#[sqlx::test]
async fn cumple_el_contrato_uno_ya_descontinuado_no_se_vuelve_a_descontinuar_y_conserva_su_fecha(
    pool: PgPool,
) {
    let setup = setup(&pool).await;
    contract::discontinue::uno_ya_descontinuado_no_se_vuelve_a_descontinuar_y_conserva_su_fecha(
        &PgProductos::new(pool),
        &setup,
    )
    .await;
}

#[sqlx::test]
async fn cumple_el_contrato_volver_a_manejar_borra_la_fecha_y_el_motivo(pool: PgPool) {
    let setup = setup(&pool).await;
    contract::discontinue::volver_a_manejar_borra_la_fecha_y_el_motivo(
        &PgProductos::new(pool),
        &setup,
    )
    .await;
}

#[sqlx::test]
async fn cumple_el_contrato_volver_a_manejar_uno_que_se_maneja_no_cambia_nada(pool: PgPool) {
    let setup = setup(&pool).await;
    contract::discontinue::volver_a_manejar_uno_que_se_maneja_no_cambia_nada(
        &PgProductos::new(pool),
        &setup,
    )
    .await;
}

#[sqlx::test]
async fn cumple_el_contrato_descontinuar_o_volver_a_manejar_uno_que_no_existe_se_rechaza(
    pool: PgPool,
) {
    let setup = setup(&pool).await;
    contract::discontinue::descontinuar_o_volver_a_manejar_uno_que_no_existe_se_rechaza(
        &PgProductos::new(pool),
        &setup,
    )
    .await;
}

#[sqlx::test]
async fn cumple_el_contrato_con_un_usuario_que_no_existe_se_rechaza_sin_cambiar_nada(pool: PgPool) {
    let setup = setup(&pool).await;
    contract::discontinue::con_un_usuario_que_no_existe_se_rechaza_sin_cambiar_nada(
        &PgProductos::new(pool),
        &setup,
    )
    .await;
}

#[sqlx::test]
async fn cumple_el_contrato_editar_un_descontinuado_lo_deja_descontinuado(pool: PgPool) {
    let setup = setup(&pool).await;
    contract::discontinue::editar_un_descontinuado_lo_deja_descontinuado(
        &PgProductos::new(pool),
        &setup,
    )
    .await;
}

#[sqlx::test]
async fn cumple_el_contrato_sin_ver_descontinuados_no_salen_ni_cuentan_en_el_total(pool: PgPool) {
    let setup = setup(&pool).await;
    contract::discontinue::sin_ver_descontinuados_no_salen_ni_cuentan_en_el_total(
        &PgProductos::new(pool),
        &setup,
    )
    .await;
}

#[sqlx::test]
async fn cumple_el_contrato_con_ver_descontinuados_salen_tambien_los_descontinuados(pool: PgPool) {
    let setup = setup(&pool).await;
    contract::discontinue::con_ver_descontinuados_salen_tambien_los_descontinuados(
        &PgProductos::new(pool),
        &setup,
    )
    .await;
}

#[sqlx::test]
async fn cumple_el_contrato_el_nid_exacto_encuentra_un_descontinuado_pero_no_los_descontinuados_con_ese_numero(
    pool: PgPool,
) {
    let setup = setup(&pool).await;
    contract::discontinue::el_nid_exacto_encuentra_un_descontinuado_pero_no_los_descontinuados_con_ese_numero(&PgProductos::new(pool), &setup).await;
}

/// Quién lo cambió por última vez, y cuándo.
async fn updated(pool: &PgPool, id: ArticuloId) -> (String, DateTime<Utc>) {
    sqlx::query_as(
        "SELECT u.email, a.updated_at
         FROM articulos a JOIN usuarios u ON u.id = a.updated_by WHERE a.id = $1",
    )
    .bind(id.0)
    .fetch_one(pool)
    .await
    .unwrap()
}

fn motivo() -> MotivoDescontinuado {
    MotivoDescontinuado::parse("ya no se vende").unwrap()
}

#[sqlx::test]
async fn descontinuar_y_volver_a_manejar_anotan_quien_y_cuando(pool: PgPool) {
    let setup = setup(&pool).await;
    let beto = update::beto(&pool).await;
    let repo = PgProductos::new(pool.clone());
    let added = repo.add(pluma(&setup, "12"), &setup.by).await.unwrap();
    let (_, added_at) = updated(&pool, added.id).await;

    repo.discontinue(added.id, motivo(), &beto).await.unwrap();
    let (discontinued_by, discontinued_at) = updated(&pool, added.id).await;
    repo.resume(added.id, &setup.by).await.unwrap();
    let (resumed_by, resumed_at) = updated(&pool, added.id).await;

    assert_eq!(
        (discontinued_by.as_str(), resumed_by.as_str()),
        ("beto@x.mx", "ana@x.mx")
    );
    assert!(added_at < discontinued_at && discontinued_at < resumed_at);
}

#[sqlx::test]
async fn volver_a_manejar_uno_que_se_maneja_no_anota_quien_ni_cuando(pool: PgPool) {
    let setup = setup(&pool).await;
    let beto = update::beto(&pool).await;
    let repo = PgProductos::new(pool.clone());
    let added = repo.add(pluma(&setup, "12"), &setup.by).await.unwrap();
    let before = updated(&pool, added.id).await;

    repo.resume(added.id, &beto).await.unwrap();

    assert_eq!(updated(&pool, added.id).await, before);
}

#[sqlx::test]
async fn un_usuario_desactivado_no_puede_descontinuar_ni_volver_a_manejar(pool: PgPool) {
    let setup = setup(&pool).await;
    let repo = PgProductos::new(pool.clone());
    let added = repo.add(pluma(&setup, "12"), &setup.by).await.unwrap();
    sqlx::query("UPDATE usuarios SET deactivated_at = now()")
        .execute(&pool)
        .await
        .unwrap();

    let discontinued = repo.discontinue(added.id, motivo(), &setup.by).await;
    let resumed = repo.resume(added.id, &setup.by).await;

    assert_eq!(discontinued, Err(ProductoError::UsuarioNotFound));
    assert_eq!(resumed, Err(ProductoError::UsuarioNotFound));
}
