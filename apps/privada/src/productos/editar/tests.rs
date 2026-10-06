use crate::test_support::*;
use axum::body::Body;
use axum::http::{Request, StatusCode, header};
use db::{PgCategorias, PgPool, PgProductos, PgUnidadesMedida};
use inventario::categorias::{CategoriasRepo, NewCategoria};
use inventario::productos::{NewProducto, Producto, ProductoFields, ProductosRepo};
use inventario::unidades_medida::UnidadesMedidaRepo;
use usuarios::usuarios::Email;

fn post(uri: &str, cookie: &str, form: &str, htmx: bool) -> Request<Body> {
    let mut request = Request::post(uri)
        .header("cookie", cookie)
        .header(header::CONTENT_TYPE, "application/x-www-form-urlencoded");
    if htmx {
        request = request.header("hx-request", "true");
    }
    request.body(Body::from(form.to_string())).unwrap()
}

/// «PLUMA AZUL» a $12.50, marca Bic, en Plumas y por pieza, dada de alta por ana@x.mx (la que
/// crea `cookie`, que va antes).
async fn pluma(pool: &PgPool) -> Producto {
    let plumas = PgCategorias::new(pool.clone())
        .add(NewCategoria::new("Plumas").unwrap())
        .await
        .unwrap();
    let pieza = PgUnidadesMedida::new(pool.clone())
        .list()
        .await
        .unwrap()
        .into_iter()
        .find(|u| u.nombre == "Pieza")
        .unwrap();
    let new = NewProducto::new(ProductoFields {
        nombre: "PLUMA AZUL",
        categoria_id: Some(plumas.id),
        unidad_medida_id: pieza.id,
        precio_venta: "12.50",
        descripcion: "",
        marca: "Bic",
        modelo: "",
        color: "",
    })
    .unwrap();
    PgProductos::new(pool.clone())
        .add(new, &Email::parse("ana@x.mx").unwrap())
        .await
        .unwrap()
}

/// El formulario como lo manda la pantalla: los datos de la pluma con otro nombre y precio.
fn form(pluma: &Producto, nombre: &str, precio: &str) -> String {
    format!(
        "nombre={nombre}&categoria={}&unidad={}&precio={precio}&marca=Bic",
        pluma.categoria_id, pluma.unidad_medida_id
    )
}

async fn stored(pool: &PgPool, pluma: &Producto) -> Producto {
    PgProductos::new(pool.clone())
        .get_by_nid(pluma.nid)
        .await
        .unwrap()
        .unwrap()
}

#[sqlx::test(migrator = "db::MIGRATOR")]
async fn sin_editar_catalogo_no_hay_enlace_ni_se_puede_editar(pool: PgPool) {
    cookie(&pool, "ana@x.mx", "Dueño").await;
    let cajero = cookie(&pool, "beto@x.mx", "Cajero").await;
    let pluma = pluma(&pool).await;
    let uri = format!("/productos/{}", pluma.nid);

    let lista = body_text(get_with_cookie(pool.clone(), "/productos", &cajero).await).await;
    let page = get_with_cookie(pool.clone(), &uri, &cajero).await;
    let saved = send(
        pool.clone(),
        post(&uri, &cajero, &form(&pluma, "PLUMA", "1"), true),
    )
    .await;

    assert!(lista.contains("PLUMA AZUL"));
    assert!(!lista.contains(&format!(r#"href="{uri}"#)));
    assert_eq!(page.status(), StatusCode::FORBIDDEN);
    assert_eq!(saved.status(), StatusCode::FORBIDDEN);
    assert_eq!(stored(&pool, &pluma).await, pluma);
}

#[sqlx::test(migrator = "db::MIGRATOR")]
async fn el_nombre_en_la_lista_lleva_a_su_edicion_con_la_busqueda(pool: PgPool) {
    let cookie = cookie(&pool, "ana@x.mx", "Dueño").await;
    let pluma = pluma(&pool).await;

    let todos = body_text(get_with_cookie(pool.clone(), "/productos", &cookie).await).await;
    let buscados = body_text(
        get_with_cookie(
            pool,
            &format!("/productos?q=pluma+az&categoria={}", pluma.categoria_id),
            &cookie,
        )
        .await,
    )
    .await;

    let nid = pluma.nid;
    assert!(todos.contains(&format!(r#"<a href="/productos/{nid}">PLUMA AZUL</a>"#)));
    assert!(buscados.contains(&format!(
        r#"<a href="/productos/{nid}?q=pluma%20az&#38;categoria={}">PLUMA AZUL</a>"#,
        pluma.categoria_id
    )));
}

#[sqlx::test(migrator = "db::MIGRATOR")]
async fn la_pagina_trae_sus_datos_actuales_sin_poner_el_cursor_en_el_nombre(pool: PgPool) {
    let cookie = cookie(&pool, "ana@x.mx", "Dueño").await;
    let pluma = pluma(&pool).await;

    let response = get_with_cookie(pool, &format!("/productos/{}", pluma.nid), &cookie).await;

    assert_eq!(response.status(), StatusCode::OK);
    let html = body_text(response).await;
    assert!(html.contains(&format!(
        "<title>Editar NID {} · Papelería de prueba</title>",
        pluma.nid
    )));
    assert!(html.contains(r#"name="nombre" value="PLUMA AZUL""#));
    assert!(html.contains(r#"name="precio" value="12.50""#));
    assert!(html.contains(r#"name="marca" value="Bic""#));
    assert!(html.contains(&format!(
        r#"<option value="{}" selected>Plumas</option>"#,
        pluma.categoria_id
    )));
    // Trae la marca: «Más datos» va abierto.
    assert!(html.contains(r#"<details class="col-12 mt-3" open>"#));
    assert!(!html.contains("autofocus"));
}

#[sqlx::test(migrator = "db::MIGRATOR")]
async fn la_flecha_regresa_a_la_lista_con_la_busqueda_de_donde_se_llego(pool: PgPool) {
    let cookie = cookie(&pool, "ana@x.mx", "Dueño").await;
    let pluma = pluma(&pool).await;

    let html = body_text(
        get_with_cookie(
            pool,
            &format!("/productos/{}?q=pluma%20az", pluma.nid),
            &cookie,
        )
        .await,
    )
    .await;

    assert!(html.contains(r#"<a href="/productos?q=pluma%20az""#));
}

#[sqlx::test(migrator = "db::MIGRATOR")]
async fn un_nid_que_no_existe_no_se_encuentra(pool: PgPool) {
    let cookie = cookie(&pool, "ana@x.mx", "Dueño").await;

    for uri in ["/productos/999", "/productos/abc", "/productos/0"] {
        let response = get_with_cookie(pool.clone(), uri, &cookie).await;
        assert_eq!(response.status(), StatusCode::NOT_FOUND, "{uri}");
        assert!(
            body_text(response)
                .await
                .contains("No hay ningún producto con ese NID.")
        );
    }
}

#[sqlx::test(migrator = "db::MIGRATOR")]
async fn guardar_deja_los_datos_nuevos_con_el_aviso(pool: PgPool) {
    let cookie = cookie(&pool, "ana@x.mx", "Dueño").await;
    let pluma = pluma(&pool).await;

    let response = send(
        pool.clone(),
        post(
            &format!("/productos/{}", pluma.nid),
            &cookie,
            &form(&pluma, "PLUMA+NEGRA", "13"),
            true,
        ),
    )
    .await;

    assert_eq!(response.status(), StatusCode::OK);
    let html = body_text(response).await;
    assert!(!html.contains("<html"), "debe ser solo el formulario");
    assert!(html.contains(&format!("Se guardaron los cambios del NID {}.", pluma.nid)));
    assert!(html.contains(r#"name="nombre" value="PLUMA NEGRA""#));
    assert!(html.contains(r#"name="precio" value="13.00""#));
    let saved = stored(&pool, &pluma).await;
    assert_eq!(saved.nombre, "PLUMA NEGRA");
    assert_eq!(
        saved.precio_venta.map(|p| p.value().to_string()),
        Some("13.00".into())
    );
}

#[sqlx::test(migrator = "db::MIGRATOR")]
async fn sin_javascript_guardar_regresa_a_la_misma_pagina_con_el_aviso(pool: PgPool) {
    let cookie = cookie(&pool, "ana@x.mx", "Dueño").await;
    let pluma = pluma(&pool).await;
    let uri = format!("/productos/{}", pluma.nid);

    let response = send(
        pool.clone(),
        post(&uri, &cookie, &form(&pluma, "PLUMA+NEGRA", "13"), false),
    )
    .await;

    assert_eq!(response.status(), StatusCode::SEE_OTHER);
    let location = response.headers()[header::LOCATION].to_str().unwrap();
    assert_eq!(location, format!("{uri}?saved=1"));
    let html = body_text(get_with_cookie(pool, location, &cookie).await).await;
    assert!(html.contains(&format!("Se guardaron los cambios del NID {}.", pluma.nid)));
}

#[sqlx::test(migrator = "db::MIGRATOR")]
async fn quitar_el_precio_no_se_guarda_y_lo_dice_junto_al_precio(pool: PgPool) {
    let cookie = cookie(&pool, "ana@x.mx", "Dueño").await;
    let pluma = pluma(&pool).await;

    let response = send(
        pool.clone(),
        post(
            &format!("/productos/{}", pluma.nid),
            &cookie,
            &form(&pluma, "PLUMA+NEGRA", ""),
            true,
        ),
    )
    .await;

    assert_eq!(response.status(), StatusCode::UNPROCESSABLE_ENTITY);
    let html = body_text(response).await;
    assert!(html.contains(
        r#"<div id="precio-error" class="invalid-feedback">Un producto con precio no puede quedar sin precio; si ya no se vende, descontinúalo.</div>"#
    ));
    // Lo escrito se queda para corregirlo.
    assert!(html.contains(r#"name="nombre" value="PLUMA NEGRA""#));
    assert_eq!(stored(&pool, &pluma).await, pluma);
}

#[sqlx::test(migrator = "db::MIGRATOR")]
async fn un_nombre_vacio_no_se_guarda_y_conserva_lo_demas(pool: PgPool) {
    let cookie = cookie(&pool, "ana@x.mx", "Dueño").await;
    let pluma = pluma(&pool).await;

    let response = send(
        pool.clone(),
        post(
            &format!("/productos/{}", pluma.nid),
            &cookie,
            &form(&pluma, "", "14"),
            true,
        ),
    )
    .await;

    assert_eq!(response.status(), StatusCode::UNPROCESSABLE_ENTITY);
    let html = body_text(response).await;
    assert!(html.contains("Escribe el nombre del producto."));
    assert!(html.contains(r#"name="precio" value="14""#));
    assert_eq!(stored(&pool, &pluma).await, pluma);
}

#[sqlx::test(migrator = "db::MIGRATOR")]
async fn el_formulario_se_manda_con_la_busqueda_de_donde_se_llego(pool: PgPool) {
    let cookie = cookie(&pool, "ana@x.mx", "Dueño").await;
    let pluma = pluma(&pool).await;

    let html = body_text(
        get_with_cookie(
            pool,
            &format!("/productos/{}?q=pluma%20az", pluma.nid),
            &cookie,
        )
        .await,
    )
    .await;

    assert!(html.contains(&format!(
        r#"action="/productos/{}?q=pluma%20az""#,
        pluma.nid
    )));
}

#[sqlx::test(migrator = "db::MIGRATOR")]
async fn sin_javascript_guardar_conserva_la_busqueda_para_volver(pool: PgPool) {
    let cookie = cookie(&pool, "ana@x.mx", "Dueño").await;
    let pluma = pluma(&pool).await;
    let uri = format!("/productos/{}", pluma.nid);

    let response = send(
        pool.clone(),
        post(
            &format!("{uri}?q=pluma%20az&categoria={}", pluma.categoria_id),
            &cookie,
            &form(&pluma, "PLUMA+NEGRA", "13"),
            false,
        ),
    )
    .await;

    let location = response.headers()[header::LOCATION].to_str().unwrap();
    let busqueda = format!("q=pluma%20az&categoria={}", pluma.categoria_id);
    assert_eq!(location, format!("{uri}?saved=1&{busqueda}"));
    let html = body_text(get_with_cookie(pool, location, &cookie).await).await;
    assert!(html.contains(&format!(
        r#"<a href="/productos?{}""#,
        busqueda.replace('&', "&#38;")
    )));
}

#[sqlx::test(migrator = "db::MIGRATOR")]
async fn sin_javascript_un_error_regresa_la_pagina_completa_con_la_busqueda(pool: PgPool) {
    let cookie = cookie(&pool, "ana@x.mx", "Dueño").await;
    let pluma = pluma(&pool).await;

    let response = send(
        pool.clone(),
        post(
            &format!("/productos/{}?q=pluma", pluma.nid),
            &cookie,
            &form(&pluma, "", "13"),
            false,
        ),
    )
    .await;

    assert_eq!(response.status(), StatusCode::UNPROCESSABLE_ENTITY);
    let html = body_text(response).await;
    assert!(html.contains("<html"));
    assert!(html.contains("Escribe el nombre del producto."));
    assert!(html.contains(r#"<a href="/productos?q=pluma""#));
}

#[sqlx::test(migrator = "db::MIGRATOR")]
async fn guardar_en_un_nid_que_no_existe_no_se_encuentra(pool: PgPool) {
    let cookie = cookie(&pool, "ana@x.mx", "Dueño").await;
    let pluma = pluma(&pool).await;

    for uri in ["/productos/999", "/productos/abc"] {
        let response = send(
            pool.clone(),
            post(uri, &cookie, &form(&pluma, "PLUMA", "1"), true),
        )
        .await;
        assert_eq!(response.status(), StatusCode::NOT_FOUND, "{uri}");
    }
    assert_eq!(stored(&pool, &pluma).await, pluma);
}

#[sqlx::test(migrator = "db::MIGRATOR")]
async fn filtrar_solo_por_categoria_la_lleva_sin_busqueda_vacia(pool: PgPool) {
    let cookie = cookie(&pool, "ana@x.mx", "Dueño").await;
    let pluma = pluma(&pool).await;

    let html = body_text(
        get_with_cookie(
            pool,
            &format!("/productos?categoria={}", pluma.categoria_id),
            &cookie,
        )
        .await,
    )
    .await;

    assert!(html.contains(&format!(
        r#"<a href="/productos/{}?categoria={}">PLUMA AZUL</a>"#,
        pluma.nid, pluma.categoria_id
    )));
}
