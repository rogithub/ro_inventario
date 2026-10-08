//! Buscar por NID y editar, contra Postgres: el contrato y lo que solo se ve en la base.

use super::*;

#[sqlx::test]
async fn cumple_el_contrato_se_encuentra_por_su_nid(pool: PgPool) {
    let setup = setup(&pool).await;
    contract::update::se_encuentra_por_su_nid(&PgProductos::new(pool), &setup).await;
}

#[sqlx::test]
async fn cumple_el_contrato_editar_cambia_todos_sus_datos_y_conserva_id_y_nid(pool: PgPool) {
    let setup = setup(&pool).await;
    contract::update::editar_cambia_todos_sus_datos_y_conserva_id_y_nid(
        &PgProductos::new(pool),
        &setup,
    )
    .await;
}

#[sqlx::test]
async fn cumple_el_contrato_a_uno_sin_precio_se_le_puede_poner_precio(pool: PgPool) {
    let setup = setup(&pool).await;
    contract::update::a_uno_sin_precio_se_le_puede_poner_precio(&PgProductos::new(pool), &setup)
        .await;
}

#[sqlx::test]
async fn cumple_el_contrato_uno_con_precio_no_puede_quedar_sin_precio(pool: PgPool) {
    let setup = setup(&pool).await;
    contract::update::uno_con_precio_no_puede_quedar_sin_precio(&PgProductos::new(pool), &setup)
        .await;
}

#[sqlx::test]
async fn cumple_el_contrato_editar_uno_que_no_existe_se_rechaza(pool: PgPool) {
    let setup = setup(&pool).await;
    contract::update::editar_uno_que_no_existe_se_rechaza(&PgProductos::new(pool), &setup).await;
}

#[sqlx::test]
async fn cumple_el_contrato_editar_con_una_categoria_que_no_existe_se_rechaza_sin_cambiar_nada(
    pool: PgPool,
) {
    let setup = setup(&pool).await;
    contract::update::editar_con_una_categoria_que_no_existe_se_rechaza_sin_cambiar_nada(
        &PgProductos::new(pool),
        &setup,
    )
    .await;
}

#[sqlx::test]
async fn cumple_el_contrato_editar_con_un_usuario_que_no_existe_se_rechaza_sin_cambiar_nada(
    pool: PgPool,
) {
    let setup = setup(&pool).await;
    contract::update::editar_con_un_usuario_que_no_existe_se_rechaza_sin_cambiar_nada(
        &PgProductos::new(pool),
        &setup,
    )
    .await;
}

#[sqlx::test]
async fn cumple_el_contrato_editar_con_una_unidad_que_no_existe_se_rechaza_sin_cambiar_nada(
    pool: PgPool,
) {
    let setup = setup(&pool).await;
    contract::update::editar_con_una_unidad_que_no_existe_se_rechaza_sin_cambiar_nada(
        &PgProductos::new(pool),
        &setup,
    )
    .await;
}

/// Los precios de su historial, del más viejo al más nuevo, con el correo de quien los puso.
async fn historial(pool: &PgPool, id: ArticuloId) -> Vec<(Decimal, String)> {
    sqlx::query_as(
        "SELECT h.precio_venta, u.email FROM precios_historial h
         JOIN usuarios u ON u.id = h.updated_by WHERE h.articulo_id = $1 ORDER BY h.desde, h.id",
    )
    .bind(id.0)
    .fetch_all(pool)
    .await
    .unwrap()
}

/// Beto, otro usuario, para ver que se anota quién editó.
pub(super) async fn beto(pool: &PgPool) -> Email {
    PgUsuarios::new(pool.clone())
        .add(NewUsuario::new("beto@x.mx", "Beto", "Encargado", "caja-de-lapices").unwrap())
        .await
        .unwrap()
        .email
}

#[sqlx::test]
async fn cambiar_el_precio_lo_agrega_al_historial_con_quien_lo_puso(pool: PgPool) {
    let setup = setup(&pool).await;
    let beto = beto(&pool).await;
    let repo = PgProductos::new(pool.clone());
    let added = repo.add(pluma(&setup, "12.50"), &setup.by).await.unwrap();

    repo.update(added.id, pluma(&setup, "13"), &beto)
        .await
        .unwrap();

    assert_eq!(
        historial(&pool, added.id).await,
        [
            (Decimal::new(1250, 2), "ana@x.mx".to_string()),
            (Decimal::new(1300, 2), "beto@x.mx".to_string()),
        ]
    );
}

#[sqlx::test]
async fn guardar_sin_cambiar_el_precio_no_escribe_historial(pool: PgPool) {
    let setup = setup(&pool).await;
    let repo = PgProductos::new(pool.clone());
    let added = repo.add(pluma(&setup, "12.50"), &setup.by).await.unwrap();
    let renamed = NewProducto::new(ProductoFields {
        nombre: "PLUMA AZUL BIC",
        categoria_id: Some(setup.categoria),
        unidad_medida_id: setup.unidad,
        // Escrito distinto: el área lo deja en 12.50, y no cuenta como cambio.
        precio_venta: "12.5",
        descripcion: "",
        marca: "",
        modelo: "",
        color: "",
    })
    .unwrap();

    repo.update(added.id, renamed, &setup.by).await.unwrap();

    assert_eq!(historial(&pool, added.id).await.len(), 1);
}

#[sqlx::test]
async fn poner_precio_a_uno_sin_precio_lo_agrega_al_historial(pool: PgPool) {
    let setup = setup(&pool).await;
    let repo = PgProductos::new(pool.clone());
    let added = repo.add(pluma(&setup, ""), &setup.by).await.unwrap();

    repo.update(added.id, pluma(&setup, "12"), &setup.by)
        .await
        .unwrap();

    assert_eq!(
        historial(&pool, added.id).await,
        [(Decimal::new(1200, 2), "ana@x.mx".to_string())]
    );
}

#[sqlx::test]
async fn editar_anota_quien_lo_cambio_y_cuando(pool: PgPool) {
    let setup = setup(&pool).await;
    let beto = beto(&pool).await;
    let repo = PgProductos::new(pool.clone());
    let added = repo.add(pluma(&setup, "12"), &setup.by).await.unwrap();

    repo.update(added.id, pluma(&setup, "12"), &beto)
        .await
        .unwrap();

    let (email, updated_later): (String, bool) = sqlx::query_as(
        "SELECT u.email, a.updated_at > a.created_at
         FROM articulos a JOIN usuarios u ON u.id = a.updated_by WHERE a.id = $1",
    )
    .bind(added.id.0)
    .fetch_one(&pool)
    .await
    .unwrap();
    assert_eq!((email.as_str(), updated_later), ("beto@x.mx", true));
}

#[sqlx::test]
async fn un_usuario_desactivado_no_puede_editar(pool: PgPool) {
    let setup = setup(&pool).await;
    let repo = PgProductos::new(pool.clone());
    let added = repo.add(pluma(&setup, "12"), &setup.by).await.unwrap();
    sqlx::query("UPDATE usuarios SET deactivated_at = now()")
        .execute(&pool)
        .await
        .unwrap();

    let result = repo.update(added.id, pluma(&setup, "13"), &setup.by).await;

    assert_eq!(result, Err(ProductoError::UsuarioNotFound));
}

#[sqlx::test]
async fn un_servicio_no_se_encuentra_ni_se_edita_como_producto(pool: PgPool) {
    let setup = setup(&pool).await;
    let (id, nid): (kernel::Uuid, i32) = sqlx::query_as(
        "INSERT INTO articulos (kind, nombre, categoria_id, unidad_medida_id)
         VALUES ('servicio', 'COPIA BN', $1, $2) RETURNING id, nid",
    )
    .bind(setup.categoria.0)
    .bind(setup.unidad.0)
    .fetch_one(&pool)
    .await
    .unwrap();
    let repo = PgProductos::new(pool);

    assert_eq!(repo.get_by_nid(Nid(nid)).await.unwrap(), None);
    assert_eq!(
        repo.update(ArticuloId(id), pluma(&setup, "1"), &setup.by)
            .await,
        Err(ProductoError::NotFound)
    );
}
