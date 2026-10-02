use crate::test_support::*;
use axum::body::Body;
use axum::http::{Request, StatusCode, header};
use db::{PgCategorias, PgPool, PgProductos, PgUnidadesMedida};
use inventario::categorias::{CategoriasRepo, NewCategoria};
use inventario::productos::{Producto, ProductosRepo};
use inventario::unidades_medida::UnidadesMedidaRepo;

fn post(cookie: &str, form: &str, htmx: bool) -> Request<Body> {
    let mut request = Request::post("/productos")
        .header("cookie", cookie)
        .header(header::CONTENT_TYPE, "application/x-www-form-urlencoded");
    if htmx {
        request = request.header("hx-request", "true");
    }
    request.body(Body::from(form.to_string())).unwrap()
}

/// Los ids que manda el formulario: una categoría «Plumas» (nueva) y la unidad Pieza.
struct Ids {
    plumas: String,
    pieza: String,
}

async fn ids(pool: &PgPool) -> Ids {
    let plumas = PgCategorias::new(pool.clone())
        .add(NewCategoria::new("Plumas").unwrap())
        .await
        .unwrap()
        .id;
    let pieza = PgUnidadesMedida::new(pool.clone())
        .list()
        .await
        .unwrap()
        .into_iter()
        .find(|u| u.nombre == "Pieza")
        .unwrap()
        .id;
    Ids {
        plumas: plumas.to_string(),
        pieza: pieza.to_string(),
    }
}

/// El formulario con nombre, categoría Plumas, Pieza y lo que se agregue al final.
fn form(ids: &Ids, nombre: &str, extra: &str) -> String {
    format!(
        "nombre={nombre}&categoria={}&unidad={}{extra}",
        ids.plumas, ids.pieza
    )
}

async fn productos(pool: &PgPool) -> Vec<Producto> {
    PgProductos::new(pool.clone()).list().await.unwrap()
}

#[sqlx::test(migrator = "db::MIGRATOR")]
async fn sin_editar_catalogo_no_se_ve_el_boton_ni_se_puede_dar_de_alta(pool: PgPool) {
    let ids = ids(&pool).await;
    let cajero = cookie(&pool, "beto@x.mx", "Cajero").await;

    let lista = body_text(get_with_cookie(pool.clone(), "/productos", &cajero).await).await;
    let page = get_with_cookie(pool.clone(), "/productos/nuevo", &cajero).await;
    let added = send(pool.clone(), post(&cajero, &form(&ids, "PLUMA", ""), true)).await;

    assert!(!lista.contains(r#"href="/productos/nuevo""#));
    assert_eq!(page.status(), StatusCode::FORBIDDEN);
    assert_eq!(added.status(), StatusCode::FORBIDDEN);
    assert!(productos(&pool).await.is_empty());
}

#[sqlx::test(migrator = "db::MIGRATOR")]
async fn con_editar_catalogo_la_lista_lleva_al_alta(pool: PgPool) {
    let cookie = cookie(&pool, "ana@x.mx", "Dueño").await;

    let html = body_text(get_with_cookie(pool, "/productos", &cookie).await).await;

    assert!(html.contains(r#"href="/productos/nuevo""#));
}

#[sqlx::test(migrator = "db::MIGRATOR")]
async fn el_formulario_trae_las_categorias_y_pieza_como_unidad(pool: PgPool) {
    let ids = ids(&pool).await;
    let cookie = cookie(&pool, "ana@x.mx", "Dueño").await;

    let response = get_with_cookie(pool, "/productos/nuevo", &cookie).await;

    assert_eq!(response.status(), StatusCode::OK);
    let html = body_text(response).await;
    assert!(html.contains("<title>Nuevo producto · Papelería de prueba</title>"));
    assert!(html.contains(&format!(
        r#"<option value="{}">Plumas</option>"#,
        ids.plumas
    )));
    assert!(html.contains(&format!(
        r#"<option value="{}" selected>Pieza</option>"#,
        ids.pieza
    )));
    // Sin nada escrito, «Más datos» va plegado.
    assert!(html.contains(r#"<details class="col-12 mt-3">"#));
}

#[sqlx::test(migrator = "db::MIGRATOR")]
async fn guardar_regresa_el_formulario_vacio_con_el_aviso_del_nid_y_la_misma_categoria(
    pool: PgPool,
) {
    let ids = ids(&pool).await;
    let cookie = cookie(&pool, "ana@x.mx", "Dueño").await;

    let response = send(
        pool.clone(),
        post(&cookie, &form(&ids, "PLUMA+AZUL", "&precio=12.5"), true),
    )
    .await;

    assert_eq!(response.status(), StatusCode::OK);
    let html = body_text(response).await;
    let nid = productos(&pool).await[0].nid;
    assert!(html.contains(&format!("Se agregó el NID {nid} «PLUMA AZUL».")));
    assert!(html.contains(r#"name="nombre" value="""#));
    assert!(html.contains(r#"name="precio" value="""#));
    assert!(html.contains(&format!(
        r#"<option value="{}" selected>Plumas</option>"#,
        ids.plumas
    )));
    let lista = body_text(get_with_cookie(pool, "/productos", &cookie).await).await;
    assert!(lista.contains("PLUMA AZUL"));
    assert!(lista.contains("$12.50"));
}

#[sqlx::test(migrator = "db::MIGRATOR")]
async fn sin_javascript_guardar_redirige_al_formulario_con_el_aviso(pool: PgPool) {
    let ids = ids(&pool).await;
    let cookie = cookie(&pool, "ana@x.mx", "Dueño").await;

    let response = send(pool.clone(), post(&cookie, &form(&ids, "PLUMA", ""), false)).await;

    assert_eq!(response.status(), StatusCode::SEE_OTHER);
    let location = response.headers()[header::LOCATION].to_str().unwrap();
    let nid = productos(&pool).await[0].nid;
    assert!(location.starts_with(&format!("/productos/nuevo?added={nid}&")));
    let html = body_text(get_with_cookie(pool, location, &cookie).await).await;
    assert!(html.contains(&format!("Se agregó el NID {nid} «PLUMA».")));
    assert!(html.contains(&format!(
        r#"<option value="{}" selected>Plumas</option>"#,
        ids.plumas
    )));
}

#[sqlx::test(migrator = "db::MIGRATOR")]
async fn guarda_marca_modelo_color_y_descripcion(pool: PgPool) {
    let ids = ids(&pool).await;
    let cookie = cookie(&pool, "ana@x.mx", "Dueño").await;
    let extra = "&marca=Bic&modelo=Cristal&color=Azul&descripcion=Punto+mediano";

    send(
        pool.clone(),
        post(&cookie, &form(&ids, "PLUMA", extra), true),
    )
    .await;

    let producto = &productos(&pool).await[0];
    assert_eq!(producto.marca.as_deref(), Some("Bic"));
    assert_eq!(producto.modelo.as_deref(), Some("Cristal"));
    assert_eq!(producto.color.as_deref(), Some("Azul"));
    assert_eq!(producto.descripcion.as_deref(), Some("Punto mediano"));
}

#[sqlx::test(migrator = "db::MIGRATOR")]
async fn un_producto_sin_precio_dice_sin_precio_en_la_lista(pool: PgPool) {
    let ids = ids(&pool).await;
    let cookie = cookie(&pool, "ana@x.mx", "Dueño").await;

    send(
        pool.clone(),
        post(&cookie, &form(&ids, "PLUMA", "&precio="), true),
    )
    .await;

    assert_eq!(productos(&pool).await[0].precio_venta, None);
    let lista = body_text(get_with_cookie(pool, "/productos", &cookie).await).await;
    assert!(lista.contains("Sin precio"));
}

#[sqlx::test(migrator = "db::MIGRATOR")]
async fn un_nombre_vacio_no_se_guarda_y_conserva_lo_escrito(pool: PgPool) {
    let ids = ids(&pool).await;
    let cookie = cookie(&pool, "ana@x.mx", "Dueño").await;

    let response = send(
        pool.clone(),
        post(&cookie, &form(&ids, "+++", "&precio=12.50"), true),
    )
    .await;

    assert_eq!(response.status(), StatusCode::UNPROCESSABLE_ENTITY);
    let html = body_text(response).await;
    assert!(html.contains("Escribe el nombre del producto."));
    assert!(html.contains(r#"value="12.50""#));
    assert!(productos(&pool).await.is_empty());
}

#[sqlx::test(migrator = "db::MIGRATOR")]
async fn un_precio_con_tres_decimales_dice_por_que(pool: PgPool) {
    let ids = ids(&pool).await;
    let cookie = cookie(&pool, "ana@x.mx", "Dueño").await;

    let response = send(
        pool.clone(),
        post(&cookie, &form(&ids, "PLUMA", "&precio=12.345"), true),
    )
    .await;

    assert_eq!(response.status(), StatusCode::UNPROCESSABLE_ENTITY);
    let html = body_text(response).await;
    assert!(html.contains(
        r#"<div id="precio-error" class="invalid-feedback">El precio lleva como máximo dos decimales.</div>"#
    ));
    assert!(html.contains(r#"name="nombre" value="PLUMA""#));
}

#[sqlx::test(migrator = "db::MIGRATOR")]
async fn sin_categoria_pide_elegirla(pool: PgPool) {
    let ids = ids(&pool).await;
    let cookie = cookie(&pool, "ana@x.mx", "Dueño").await;
    let body = format!("nombre=PLUMA&categoria=&unidad={}", ids.pieza);

    let response = send(pool.clone(), post(&cookie, &body, true)).await;

    assert_eq!(response.status(), StatusCode::UNPROCESSABLE_ENTITY);
    let html = body_text(response).await;
    assert!(html.contains(
        r#"<div id="categoria-error" class="invalid-feedback">Elige la categoría.</div>"#
    ));
}

#[sqlx::test(migrator = "db::MIGRATOR")]
async fn un_error_en_mas_datos_lo_deja_abierto(pool: PgPool) {
    let ids = ids(&pool).await;
    let cookie = cookie(&pool, "ana@x.mx", "Dueño").await;
    let marca = format!("&marca={}", "a".repeat(151));

    let response = send(
        pool.clone(),
        post(&cookie, &form(&ids, "PLUMA", &marca), true),
    )
    .await;

    assert_eq!(response.status(), StatusCode::UNPROCESSABLE_ENTITY);
    let html = body_text(response).await;
    assert!(html.contains(r#"<details class="col-12 mt-3" open>"#));
    assert!(html.contains("La marca no puede pasar de 150 caracteres."));
}

/// El formulario con categoría Plumas y la unidad Metro, que no es la de omisión.
async fn form_en_metros(pool: &PgPool, ids: &Ids) -> (String, String) {
    let metro = PgUnidadesMedida::new(pool.clone())
        .list()
        .await
        .unwrap()
        .into_iter()
        .find(|u| u.nombre == "Metro")
        .unwrap()
        .id
        .to_string();
    let body = format!("nombre=LISTON&categoria={}&unidad={metro}", ids.plumas);
    (body, metro)
}

#[sqlx::test(migrator = "db::MIGRATOR")]
async fn guardar_conserva_la_unidad_aunque_no_sea_pieza(pool: PgPool) {
    let ids = ids(&pool).await;
    let cookie = cookie(&pool, "ana@x.mx", "Dueño").await;
    let (body, metro) = form_en_metros(&pool, &ids).await;

    let html = body_text(send(pool, post(&cookie, &body, true)).await).await;

    assert!(html.contains(&format!(
        r#"<option value="{metro}" selected>Metro</option>"#
    )));
    assert!(html.contains(&format!(r#"<option value="{}">Pieza</option>"#, ids.pieza)));
}

#[sqlx::test(migrator = "db::MIGRATOR")]
async fn sin_javascript_guardar_conserva_la_unidad_aunque_no_sea_pieza(pool: PgPool) {
    let ids = ids(&pool).await;
    let cookie = cookie(&pool, "ana@x.mx", "Dueño").await;
    let (body, metro) = form_en_metros(&pool, &ids).await;

    let response = send(pool.clone(), post(&cookie, &body, false)).await;
    let location = response.headers()[header::LOCATION].to_str().unwrap();
    let html = body_text(get_with_cookie(pool, location, &cookie).await).await;

    assert!(html.contains(&format!(
        r#"<option value="{metro}" selected>Metro</option>"#
    )));
}
