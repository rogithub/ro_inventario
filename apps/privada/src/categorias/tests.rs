use crate::test_support::*;
use axum::body::Body;
use axum::http::{Request, StatusCode, header};
use db::PgPool;

fn post(cookie: &str, form: &str, htmx: bool) -> Request<Body> {
    let mut request = Request::post("/categorias")
        .header("cookie", cookie)
        .header(header::CONTENT_TYPE, "application/x-www-form-urlencoded");
    if htmx {
        request = request.header("hx-request", "true");
    }
    request.body(Body::from(form.to_string())).unwrap()
}

#[sqlx::test(migrator = "db::MIGRATOR")]
async fn la_pagina_muestra_las_categorias_agregadas(pool: PgPool) {
    let cookie = cookie(&pool, "ana@x.mx", "Dueño").await;
    send(pool.clone(), post(&cookie, "nombre=Plumas", false)).await;

    let response = get_with_cookie(pool, "/categorias", &cookie).await;

    assert_eq!(response.status(), StatusCode::OK);
    let html = body_text(response).await;
    assert!(html.contains("<title>Categorías · Papelería de prueba</title>"));
    assert!(html.contains("<td>Plumas</td>"));
}

#[sqlx::test(migrator = "db::MIGRATOR")]
async fn sin_categorias_la_pagina_lo_dice(pool: PgPool) {
    let cookie = cookie(&pool, "ana@x.mx", "Dueño").await;
    let html = body_text(get_with_cookie(pool, "/categorias", &cookie).await).await;
    assert!(html.contains("Todavía no hay categorías."));
}

#[sqlx::test(migrator = "db::MIGRATOR")]
async fn agregar_con_htmx_regresa_solo_la_seccion_con_la_nueva(pool: PgPool) {
    let cookie = cookie(&pool, "ana@x.mx", "Dueño").await;
    let response = send(pool, post(&cookie, "nombre=Cuadernos", true)).await;
    assert_eq!(response.status(), StatusCode::OK);

    let html = body_text(response).await;
    assert!(html.contains("<td>Cuadernos</td>"));
    assert!(!html.contains("<html"), "debe ser solo la sección");
    assert!(html.contains(r#"value="""#), "el formulario queda vacío");
}

#[sqlx::test(migrator = "db::MIGRATOR")]
async fn agregar_sin_htmx_regresa_a_la_lista(pool: PgPool) {
    let cookie = cookie(&pool, "ana@x.mx", "Dueño").await;
    let response = send(pool.clone(), post(&cookie, "nombre=Cuadernos", false)).await;
    assert_eq!(response.status(), StatusCode::SEE_OTHER);
    assert_eq!(response.headers()[header::LOCATION], "/categorias");

    let html = body_text(get_with_cookie(pool, "/categorias", &cookie).await).await;
    assert!(html.contains("<td>Cuadernos</td>"));
}

#[sqlx::test(migrator = "db::MIGRATOR")]
async fn un_nombre_vacio_regresa_422_con_el_mensaje(pool: PgPool) {
    let cookie = cookie(&pool, "ana@x.mx", "Dueño").await;
    let response = send(pool, post(&cookie, "nombre=+++", true)).await;
    assert_eq!(response.status(), StatusCode::UNPROCESSABLE_ENTITY);
    assert!(
        body_text(response)
            .await
            .contains("Escribe el nombre de la categoría.")
    );
}

#[sqlx::test(migrator = "db::MIGRATOR")]
async fn un_nombre_de_mas_de_150_caracteres_regresa_422_con_el_mensaje(pool: PgPool) {
    let cookie = cookie(&pool, "ana@x.mx", "Dueño").await;
    let form = format!("nombre={}", "a".repeat(151));

    let response = send(pool, post(&cookie, &form, true)).await;

    assert_eq!(response.status(), StatusCode::UNPROCESSABLE_ENTITY);
    assert!(
        body_text(response)
            .await
            .contains("El nombre no puede pasar de 150 caracteres.")
    );
}

#[sqlx::test(migrator = "db::MIGRATOR")]
async fn un_nombre_repetido_regresa_422_y_conserva_lo_escrito(pool: PgPool) {
    let cookie = cookie(&pool, "ana@x.mx", "Dueño").await;
    send(pool.clone(), post(&cookie, "nombre=Plumas", true)).await;

    let response = send(pool, post(&cookie, "nombre=plumas", true)).await;

    assert_eq!(response.status(), StatusCode::UNPROCESSABLE_ENTITY);
    let html = body_text(response).await;
    assert!(html.contains("Ya existe la categoría «plumas»."));
    assert!(html.contains(r#"value="plumas""#));
}

#[sqlx::test(migrator = "db::MIGRATOR")]
async fn un_cajero_ve_la_lista_pero_no_el_formulario(pool: PgPool) {
    let dueno = cookie(&pool, "ana@x.mx", "Dueño").await;
    send(pool.clone(), post(&dueno, "nombre=Plumas", true)).await;
    let cajero = cookie(&pool, "beto@x.mx", "Cajero").await;

    let html = body_text(get_with_cookie(pool, "/categorias", &cajero).await).await;

    assert!(html.contains("<td>Plumas</td>"));
    assert!(!html.contains(r#"action="/categorias""#));
}

#[sqlx::test(migrator = "db::MIGRATOR")]
async fn un_cajero_no_puede_agregar_aunque_mande_el_formulario(pool: PgPool) {
    let cookie = cookie(&pool, "ana@x.mx", "Cajero").await;

    let response = send(pool.clone(), post(&cookie, "nombre=Plumas", true)).await;

    assert_eq!(response.status(), StatusCode::FORBIDDEN);
    let html = body_text(get_with_cookie(pool, "/categorias", &cookie).await).await;
    assert!(!html.contains("<td>Plumas</td>"));
}

#[sqlx::test(migrator = "db::MIGRATOR")]
async fn si_falla_la_lista_se_ve_el_id_para_reportarlo(pool: PgPool) {
    let cookie = cookie(&pool, "ana@x.mx", "Dueño").await;
    sqlx::query("DROP TABLE categorias CASCADE")
        .execute(&pool)
        .await
        .unwrap();

    let response = get_with_cookie(pool, "/categorias", &cookie).await;

    assert_eq!(response.status(), StatusCode::INTERNAL_SERVER_ERROR);
    let id = response.headers()["x-request-id"]
        .to_str()
        .unwrap()
        .to_string();
    assert!(body_text(response).await.contains(&id));
}
