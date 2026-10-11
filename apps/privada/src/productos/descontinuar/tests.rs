use crate::productos::editar::tests::{pluma, post, stored};
use crate::test_support::*;
use axum::http::{StatusCode, header};
use db::{PgPool, PgProductos};
use inventario::productos::ProductosRepo;
use inventario::productos::discontinue::MotivoDescontinuado;
use usuarios::usuarios::Email;

/// Lo descontinúa directo en la base, como si lo hubiera hecho otra pestaña.
async fn discontinue(pool: &PgPool, pluma: &inventario::productos::Producto, motivo: &str) {
    PgProductos::new(pool.clone())
        .discontinue(
            pluma.id,
            MotivoDescontinuado::parse(motivo).unwrap(),
            &Email::parse("ana@x.mx").unwrap(),
        )
        .await
        .unwrap();
}

#[sqlx::test(migrator = "db::MIGRATOR")]
async fn descontinuar_con_motivo_muestra_la_franja_con_la_fecha_y_el_motivo(pool: PgPool) {
    let cookie = cookie(&pool, "ana@x.mx", "Dueño").await;
    let pluma = pluma(&pool).await;
    let uri = format!("/productos/{}", pluma.nid);

    let response = send(
        pool.clone(),
        post(
            &format!("{uri}/descontinuar"),
            &cookie,
            "motivo=+el+proveedor+ya+no+lo+vende+",
            false,
        ),
    )
    .await;

    assert_eq!(response.status(), StatusCode::SEE_OTHER);
    assert_eq!(response.headers()[header::LOCATION], uri.as_str());
    let saved = stored(&pool, &pluma).await.descontinuado.unwrap();
    assert_eq!(saved.motivo, "el proveedor ya no lo vende");
    let today = crate::dates::short_date(saved.at, chrono_tz::America::Cancun);
    let html = body_text(get_with_cookie(pool, &uri, &cookie).await).await;
    assert!(html.contains(&format!(
        "Descontinuado el {today}: el proveedor ya no lo vende"
    )));
    assert!(html.contains(&format!(r#"action="{uri}/volver-a-manejar""#)));
    assert!(!html.contains(r#"name="motivo""#));
}

#[sqlx::test(migrator = "db::MIGRATOR")]
async fn sin_motivo_no_se_descontinua_y_lo_dice_junto_al_motivo(pool: PgPool) {
    let cookie = cookie(&pool, "ana@x.mx", "Dueño").await;
    let pluma = pluma(&pool).await;

    let response = send(
        pool.clone(),
        post(
            &format!("/productos/{}/descontinuar", pluma.nid),
            &cookie,
            "motivo=+++",
            false,
        ),
    )
    .await;

    assert_eq!(response.status(), StatusCode::UNPROCESSABLE_ENTITY);
    let html = body_text(response).await;
    assert!(html.contains(
        r#"<div id="motivo-error" class="invalid-feedback">Escribe por qué se descontinúa.</div>"#
    ));
    // La página completa, con el producto como está.
    assert!(html.contains(r#"name="nombre" value="PLUMA AZUL""#));
    assert_eq!(stored(&pool, &pluma).await.descontinuado, None);
}

#[sqlx::test(migrator = "db::MIGRATOR")]
async fn un_motivo_de_301_caracteres_no_se_descontinua_y_conserva_lo_escrito(pool: PgPool) {
    let cookie = cookie(&pool, "ana@x.mx", "Dueño").await;
    let pluma = pluma(&pool).await;
    let motivo = "ñ".repeat(301);

    let response = send(
        pool.clone(),
        post(
            &format!("/productos/{}/descontinuar", pluma.nid),
            &cookie,
            &format!("motivo={motivo}"),
            false,
        ),
    )
    .await;

    assert_eq!(response.status(), StatusCode::UNPROCESSABLE_ENTITY);
    let html = body_text(response).await;
    assert!(html.contains("El motivo no puede pasar de 300 caracteres."));
    assert!(html.contains(&format!("{motivo}</textarea>")));
    assert_eq!(stored(&pool, &pluma).await.descontinuado, None);
}

#[sqlx::test(migrator = "db::MIGRATOR")]
async fn volver_a_manejar_quita_la_franja_y_regresa_el_motivo_y_el_boton(pool: PgPool) {
    let cookie = cookie(&pool, "ana@x.mx", "Dueño").await;
    let pluma = pluma(&pool).await;
    discontinue(&pool, &pluma, "el proveedor ya no lo vende").await;
    let uri = format!("/productos/{}", pluma.nid);

    let response = send(
        pool.clone(),
        post(&format!("{uri}/volver-a-manejar"), &cookie, "", false),
    )
    .await;

    assert_eq!(response.status(), StatusCode::SEE_OTHER);
    assert_eq!(stored(&pool, &pluma).await.descontinuado, None);
    let html = body_text(get_with_cookie(pool, &uri, &cookie).await).await;
    assert!(!html.contains("Descontinuado el"));
    assert!(html.contains(r#"name="motivo""#));
    assert!(html.contains(&format!(r#"action="{uri}/descontinuar""#)));
}

#[sqlx::test(migrator = "db::MIGRATOR")]
async fn descontinuar_y_volver_a_manejar_regresan_a_la_edicion_con_la_busqueda(pool: PgPool) {
    let cookie = cookie(&pool, "ana@x.mx", "Dueño").await;
    let pluma = pluma(&pool).await;
    let uri = format!("/productos/{}", pluma.nid);
    let search = "?q=pluma%20az&descontinuados=1";

    let page =
        body_text(get_with_cookie(pool.clone(), &format!("{uri}{search}"), &cookie).await).await;
    let discontinued = send(
        pool.clone(),
        post(
            &format!("{uri}/descontinuar{search}"),
            &cookie,
            "motivo=ya+no+se+vende",
            false,
        ),
    )
    .await;
    let resumed = send(
        pool.clone(),
        post(
            &format!("{uri}/volver-a-manejar{search}"),
            &cookie,
            "",
            false,
        ),
    )
    .await;

    let action = format!("{uri}/descontinuar{search}").replace('&', "&#38;");
    assert!(page.contains(&format!(r#"action="{action}""#)), "{page}");
    assert_eq!(
        discontinued.headers()[header::LOCATION],
        format!("{uri}{search}").as_str()
    );
    assert_eq!(
        resumed.headers()[header::LOCATION],
        format!("{uri}{search}").as_str()
    );
}

#[sqlx::test(migrator = "db::MIGRATOR")]
async fn sin_editar_catalogo_no_se_puede_descontinuar_ni_volver_a_manejar(pool: PgPool) {
    cookie(&pool, "ana@x.mx", "Dueño").await;
    let cajero = cookie(&pool, "beto@x.mx", "Cajero").await;
    let pluma = pluma(&pool).await;
    let uri = format!("/productos/{}", pluma.nid);

    let discontinued = send(
        pool.clone(),
        post(&format!("{uri}/descontinuar"), &cajero, "motivo=x", false),
    )
    .await;
    assert_eq!(discontinued.status(), StatusCode::FORBIDDEN);
    assert_eq!(stored(&pool, &pluma).await.descontinuado, None);

    discontinue(&pool, &pluma, "el proveedor ya no lo vende").await;
    let resumed = send(
        pool.clone(),
        post(&format!("{uri}/volver-a-manejar"), &cajero, "", false),
    )
    .await;
    assert_eq!(resumed.status(), StatusCode::FORBIDDEN);
    assert!(stored(&pool, &pluma).await.descontinuado.is_some());
}

#[sqlx::test(migrator = "db::MIGRATOR")]
async fn descontinuar_uno_ya_descontinuado_conserva_su_fecha_y_lo_avisa(pool: PgPool) {
    let cookie = cookie(&pool, "ana@x.mx", "Dueño").await;
    let pluma = pluma(&pool).await;
    discontinue(&pool, &pluma, "el proveedor ya no lo vende").await;
    let before = stored(&pool, &pluma).await.descontinuado;

    let response = send(
        pool.clone(),
        post(
            &format!("/productos/{}/descontinuar", pluma.nid),
            &cookie,
            "motivo=otro+motivo",
            false,
        ),
    )
    .await;

    assert_eq!(response.status(), StatusCode::UNPROCESSABLE_ENTITY);
    let html = body_text(response).await;
    assert!(html.contains("Ese producto ya estaba descontinuado."));
    assert!(html.contains(": el proveedor ya no lo vende"));
    assert_eq!(stored(&pool, &pluma).await.descontinuado, before);
}

#[sqlx::test(migrator = "db::MIGRATOR")]
async fn descontinuar_un_nid_que_no_existe_no_se_encuentra(pool: PgPool) {
    let cookie = cookie(&pool, "ana@x.mx", "Dueño").await;

    for uri in [
        "/productos/999/descontinuar",
        "/productos/abc/descontinuar",
        "/productos/999/volver-a-manejar",
    ] {
        let response = send(pool.clone(), post(uri, &cookie, "motivo=x", false)).await;
        assert_eq!(response.status(), StatusCode::NOT_FOUND, "{uri}");
    }
}

#[sqlx::test(migrator = "db::MIGRATOR")]
async fn sin_motivo_en_uno_ya_descontinuado_avisa_que_ya_lo_estaba(pool: PgPool) {
    let cookie = cookie(&pool, "ana@x.mx", "Dueño").await;
    let pluma = pluma(&pool).await;
    discontinue(&pool, &pluma, "el proveedor ya no lo vende").await;

    let response = send(
        pool.clone(),
        post(
            &format!("/productos/{}/descontinuar", pluma.nid),
            &cookie,
            "motivo=+++",
            false,
        ),
    )
    .await;

    assert_eq!(response.status(), StatusCode::UNPROCESSABLE_ENTITY);
    let html = body_text(response).await;
    assert!(html.contains("Ese producto ya estaba descontinuado."));
    assert!(html.contains(": el proveedor ya no lo vende"));
}
