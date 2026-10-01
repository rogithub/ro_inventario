use crate::test_support::*;
use axum::body::Body;
use axum::http::{Request, StatusCode, header};
use db::{PgCategorias, PgPool, PgProductos, PgUnidadesMedida};
use inventario::categorias::{CategoriaId, CategoriasRepo, NewCategoria};
use inventario::productos::{NewProducto, ProductoFields, ProductosRepo};
use inventario::unidades_medida::UnidadesMedidaRepo;
use usuarios::usuarios::Email;

/// La categoría con ese nombre (la crea si no existe).
async fn categoria(pool: &PgPool, nombre: &str) -> CategoriaId {
    let categorias = PgCategorias::new(pool.clone());
    if let Some(c) = categorias
        .list()
        .await
        .unwrap()
        .into_iter()
        .find(|c| c.nombre == nombre)
    {
        return c.id;
    }
    categorias
        .add(NewCategoria::new(nombre).unwrap())
        .await
        .unwrap()
        .id
}

/// Da de alta un producto por pieza, a nombre de ana@x.mx (la de `cookie`).
async fn producto(pool: &PgPool, nombre: &str, categoria_nombre: &str, precio: &str) -> i32 {
    let categoria_id = categoria(pool, categoria_nombre).await;
    let pieza = PgUnidadesMedida::new(pool.clone())
        .list()
        .await
        .unwrap()
        .into_iter()
        .find(|u| u.nombre == "Pieza")
        .unwrap();
    let new = NewProducto::new(ProductoFields {
        nombre,
        categoria_id,
        unidad_medida_id: pieza.id,
        precio_venta: precio,
        descripcion: "",
        marca: "",
        modelo: "",
        color: "",
    })
    .unwrap();
    PgProductos::new(pool.clone())
        .add(new, &Email::parse("ana@x.mx").unwrap())
        .await
        .unwrap()
        .nid
        .0
}

fn get_htmx(uri: &str, cookie: &str) -> Request<Body> {
    Request::get(uri)
        .header("cookie", cookie)
        .header("hx-request", "true")
        .body(Body::empty())
        .unwrap()
}

#[sqlx::test(migrator = "db::MIGRATOR")]
async fn la_pagina_lista_los_productos_con_nid_nombre_categoria_y_precio(pool: PgPool) {
    let cookie = cookie(&pool, "ana@x.mx", "Dueño").await;
    let nid = producto(&pool, "PLUMA AZUL", "Plumas", "12.5").await;

    let response = get_with_cookie(pool, "/productos", &cookie).await;

    assert_eq!(response.status(), StatusCode::OK);
    let html = body_text(response).await;
    assert!(html.contains("<title>Productos · Papelería de prueba</title>"));
    assert!(html.contains(&format!(">{nid}</td>")));
    assert!(html.contains("PLUMA AZUL"));
    assert!(html.contains(r#"<div class="small text-body-secondary">Plumas</div>"#));
    assert!(html.contains("$12.50"));
}

#[sqlx::test(migrator = "db::MIGRATOR")]
async fn un_producto_sin_precio_lo_dice(pool: PgPool) {
    let cookie = cookie(&pool, "ana@x.mx", "Dueño").await;
    producto(&pool, "PLUMA AZUL", "Plumas", "").await;

    let html = body_text(get_with_cookie(pool, "/productos", &cookie).await).await;

    assert!(html.contains("Sin precio"));
}

#[sqlx::test(migrator = "db::MIGRATOR")]
async fn sin_productos_la_pagina_lo_dice(pool: PgPool) {
    let cookie = cookie(&pool, "ana@x.mx", "Dueño").await;
    let html = body_text(get_with_cookie(pool, "/productos", &cookie).await).await;
    assert!(html.contains("Todavía no hay productos."));
}

#[sqlx::test(migrator = "db::MIGRATOR")]
async fn buscar_muestra_solo_los_que_coinciden_y_conserva_lo_escrito(pool: PgPool) {
    let cookie = cookie(&pool, "ana@x.mx", "Dueño").await;
    producto(&pool, "LÁPIZ AMARILLO", "Lápices", "5").await;
    producto(&pool, "PLUMA AZUL", "Plumas", "12").await;

    let html = body_text(get_with_cookie(pool, "/productos?q=lapiz", &cookie).await).await;

    assert!(html.contains("LÁPIZ AMARILLO"));
    assert!(!html.contains("PLUMA AZUL"));
    assert!(html.contains(r#"value="lapiz""#));
}

#[sqlx::test(migrator = "db::MIGRATOR")]
async fn sin_coincidencias_lo_dice(pool: PgPool) {
    let cookie = cookie(&pool, "ana@x.mx", "Dueño").await;
    producto(&pool, "PLUMA AZUL", "Plumas", "12").await;

    let html = body_text(get_with_cookie(pool, "/productos?q=tijeras", &cookie).await).await;

    assert!(html.contains("Ningún producto coincide con la búsqueda."));
}

#[sqlx::test(migrator = "db::MIGRATOR")]
async fn buscar_con_htmx_regresa_solo_los_resultados(pool: PgPool) {
    let cookie = cookie(&pool, "ana@x.mx", "Dueño").await;
    producto(&pool, "PLUMA AZUL", "Plumas", "12").await;

    let response = send(pool, get_htmx("/productos?q=pluma", &cookie)).await;

    assert_eq!(response.status(), StatusCode::OK);
    // La misma URL responde la página o la sección: el navegador no debe confundirlas.
    assert_eq!(
        response.headers()[header::VARY],
        "HX-Request, HX-History-Restore-Request"
    );
    let html = body_text(response).await;
    assert!(html.contains("PLUMA AZUL"));
    assert!(!html.contains("<html"), "debe ser solo la sección");
    assert!(!html.contains("<form"), "el buscador no se reemplaza");
}

#[sqlx::test(migrator = "db::MIGRATOR")]
async fn al_volver_atras_sin_cache_htmx_recibe_la_pagina_completa(pool: PgPool) {
    let cookie = cookie(&pool, "ana@x.mx", "Dueño").await;
    let request = Request::get("/productos?q=pluma")
        .header("cookie", &cookie)
        .header("hx-request", "true")
        .header("hx-history-restore-request", "true")
        .body(Body::empty())
        .unwrap();

    let html = body_text(send(pool, request).await).await;

    assert!(html.contains("<html"));
}

#[sqlx::test(migrator = "db::MIGRATOR")]
async fn filtrar_por_categoria_deja_esa_categoria_elegida(pool: PgPool) {
    let cookie = cookie(&pool, "ana@x.mx", "Dueño").await;
    producto(&pool, "PLUMA AZUL", "Plumas", "12").await;
    producto(&pool, "CUADERNO", "Cuadernos", "30").await;
    let cuadernos = categoria(&pool, "Cuadernos").await;

    let uri = format!("/productos?q=&categoria={cuadernos}");
    let html = body_text(get_with_cookie(pool, &uri, &cookie).await).await;

    assert!(html.contains("CUADERNO"));
    assert!(!html.contains("PLUMA AZUL"));
    assert!(html.contains(&format!(
        r#"<option value="{cuadernos}" selected>Cuadernos</option>"#
    )));
}

#[sqlx::test(migrator = "db::MIGRATOR")]
async fn una_categoria_que_no_es_un_id_se_ignora(pool: PgPool) {
    let cookie = cookie(&pool, "ana@x.mx", "Dueño").await;
    producto(&pool, "PLUMA AZUL", "Plumas", "12").await;

    let response = get_with_cookie(pool, "/productos?categoria=plumas", &cookie).await;

    assert_eq!(response.status(), StatusCode::OK);
    assert!(body_text(response).await.contains("PLUMA AZUL"));
}

#[sqlx::test(migrator = "db::MIGRATOR")]
async fn con_mas_de_100_muestra_100_y_dice_cuantos_hay(pool: PgPool) {
    let cookie = cookie(&pool, "ana@x.mx", "Dueño").await;
    for i in 0..101 {
        producto(&pool, &format!("PLUMA {i:03}"), "Plumas", "12").await;
    }

    let html = body_text(get_with_cookie(pool, "/productos", &cookie).await).await;

    assert!(html.contains("Se muestran 100 de 101; escribe para encontrar los demás."));
    assert!(html.contains("PLUMA 099"));
    assert!(!html.contains("PLUMA 100"));
}

#[test]
fn los_totales_se_escriben_con_separador_de_miles() {
    assert_eq!(super::thousands(7), "7");
    assert_eq!(super::thousands(2336), "2,336");
    assert_eq!(super::thousands(1_234_567), "1,234,567");
}

#[sqlx::test(migrator = "db::MIGRATOR")]
async fn un_cajero_tambien_puede_buscar_productos(pool: PgPool) {
    cookie(&pool, "ana@x.mx", "Dueño").await;
    producto(&pool, "PLUMA AZUL", "Plumas", "12").await;
    let cajero = cookie(&pool, "beto@x.mx", "Cajero").await;

    let response = get_with_cookie(pool, "/productos?q=pluma", &cajero).await;

    assert_eq!(response.status(), StatusCode::OK);
    assert!(body_text(response).await.contains("PLUMA AZUL"));
}

#[sqlx::test(migrator = "db::MIGRATOR")]
async fn si_falla_la_busqueda_se_ve_el_id_para_reportarlo(pool: PgPool) {
    let cookie = cookie(&pool, "ana@x.mx", "Dueño").await;
    sqlx::query("DROP TABLE productos CASCADE")
        .execute(&pool)
        .await
        .unwrap();

    let response = get_with_cookie(pool, "/productos", &cookie).await;

    assert_eq!(response.status(), StatusCode::INTERNAL_SERVER_ERROR);
    let id = response.headers()["x-request-id"]
        .to_str()
        .unwrap()
        .to_string();
    assert!(body_text(response).await.contains(&id));
}

/// Si una búsqueda falla, el error se pinta dentro del contenedor y no en su lugar: la siguiente
/// tecla sigue encontrando dónde poner los resultados (revisor de 7b1).
#[sqlx::test(migrator = "db::MIGRATOR")]
async fn si_falla_una_busqueda_el_buscador_sigue_teniendo_donde_pintar(pool: PgPool) {
    let cookie = cookie(&pool, "ana@x.mx", "Dueño").await;
    let page = body_text(get_with_cookie(pool.clone(), "/productos", &cookie).await).await;
    assert!(page.contains(r##"hx-target="#results" hx-swap="innerHTML""##));
    assert!(page.contains(r#"<div id="results">"#));
    sqlx::query("DROP TABLE productos CASCADE")
        .execute(&pool)
        .await
        .unwrap();

    let response = send(pool, get_htmx("/productos?q=pluma", &cookie)).await;

    assert_eq!(response.status(), StatusCode::INTERNAL_SERVER_ERROR);
    let html = body_text(response).await;
    assert!(html.contains("Algo falló"));
    assert!(
        !html.contains(r#"id="results""#),
        "el contenedor se queda en la página"
    );
}
