//! Los descontinuados en la lista: no salen, salvo con «Ver descontinuados» o por su NID.

use inventario::articulos::Nid;
use inventario::productos::discontinue::MotivoDescontinuado;

use super::*;

/// Da de alta un producto y lo descontinúa; regresa su NID.
async fn descontinuado(pool: &PgPool, nombre: &str) -> i32 {
    let nid = producto(pool, nombre, "Plumas", "12").await;
    let productos = PgProductos::new(pool.clone());
    let added = productos.get_by_nid(Nid(nid)).await.unwrap().unwrap();
    productos
        .discontinue(
            added.id,
            MotivoDescontinuado::parse("ya no se vende").unwrap(),
            &Email::parse("ana@x.mx").unwrap(),
        )
        .await
        .unwrap();
    nid
}

#[sqlx::test(migrator = "db::MIGRATOR")]
async fn un_descontinuado_no_sale_en_la_lista_y_la_casilla_va_sin_marcar(pool: PgPool) {
    let cookie = cookie(&pool, "ana@x.mx", "Dueño").await;
    producto(&pool, "PLUMA AZUL", "Plumas", "12").await;
    descontinuado(&pool, "PLUMA ROJA").await;

    let html = body_text(get_with_cookie(pool, "/productos", &cookie).await).await;

    assert!(html.contains("PLUMA AZUL"));
    assert!(!html.contains("PLUMA ROJA"));
    assert!(html.contains(r#"type="checkbox" value="1" class="form-check-input">"#));
}

#[sqlx::test(migrator = "db::MIGRATOR")]
async fn con_ver_descontinuados_sale_en_gris_con_la_insignia_y_la_casilla_marcada(pool: PgPool) {
    let cookie = cookie(&pool, "ana@x.mx", "Dueño").await;
    let nid = descontinuado(&pool, "PLUMA ROJA").await;

    let html =
        body_text(get_with_cookie(pool, "/productos?q=pluma&descontinuados=1", &cookie).await)
            .await;

    assert!(html.contains(&format!(
        r#"<td class="text-end font-monospace text-body-secondary">{nid}</td>"#
    )));
    assert!(html.contains(
        r#"class="link-secondary">PLUMA ROJA</a> <span class="badge text-bg-secondary">Descontinuado</span>"#
    ));
    assert!(html.contains(r#"class="form-check-input" checked>"#));
}

#[sqlx::test(migrator = "db::MIGRATOR")]
async fn marcar_la_casilla_con_htmx_trae_los_descontinuados(pool: PgPool) {
    let cookie = cookie(&pool, "ana@x.mx", "Dueño").await;
    descontinuado(&pool, "PLUMA ROJA").await;

    let response = send(
        pool,
        get_htmx("/productos?q=pluma&descontinuados=1", &cookie),
    )
    .await;

    let html = body_text(response).await;
    assert!(html.contains("Descontinuado</span>"));
    assert!(!html.contains("<form"), "el buscador no se reemplaza");
}

#[sqlx::test(migrator = "db::MIGRATOR")]
async fn por_su_nid_un_descontinuado_sale_aunque_la_casilla_no_este_marcada(pool: PgPool) {
    let cookie = cookie(&pool, "ana@x.mx", "Dueño").await;
    let nid = descontinuado(&pool, "PLUMA ROJA").await;

    let html =
        body_text(get_with_cookie(pool, &format!("/productos?q={nid}"), &cookie).await).await;

    assert!(html.contains("PLUMA ROJA"));
    assert!(html.contains("Descontinuado</span>"));
}

#[sqlx::test(migrator = "db::MIGRATOR")]
async fn el_enlace_a_la_edicion_lleva_ver_descontinuados(pool: PgPool) {
    let cookie = cookie(&pool, "ana@x.mx", "Dueño").await;
    let nid = descontinuado(&pool, "PLUMA ROJA").await;

    let html =
        body_text(get_with_cookie(pool, "/productos?q=pluma&descontinuados=1", &cookie).await)
            .await;

    assert!(html.contains(&format!(
        r#"<a href="/productos/{nid}?q=pluma&#38;descontinuados=1""#
    )));
}

#[sqlx::test(migrator = "db::MIGRATOR")]
async fn la_flecha_de_la_edicion_regresa_con_ver_descontinuados(pool: PgPool) {
    let cookie = cookie(&pool, "ana@x.mx", "Dueño").await;
    let nid = descontinuado(&pool, "PLUMA ROJA").await;

    let html = body_text(
        get_with_cookie(
            pool,
            &format!("/productos/{nid}?q=pluma&descontinuados=1"),
            &cookie,
        )
        .await,
    )
    .await;

    assert!(html.contains(r#"<a href="/productos?q=pluma&#38;descontinuados=1""#));
}
