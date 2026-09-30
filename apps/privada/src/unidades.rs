//! Pantalla de unidades de medida: la lista y agregar una.

use askama::Template;
use axum::extract::State;
use axum::http::{HeaderMap, StatusCode};
use axum::response::{Html, IntoResponse, Redirect, Response};
use axum::{Extension, Form};
use inventario::unidades_medida::{
    NuevaUnidadMedida, UnidadMedida, UnidadMedidaError, UnidadesMedidaRepo,
};
use serde::Deserialize;
use usuarios::permisos::Permiso;
use usuarios::usuarios::Usuario;

use crate::{AppState, falla_interna};

#[derive(Template)]
#[template(path = "unidades.html")]
struct Pagina<'a> {
    negocio: &'a str,
    usuario_nombre: Option<&'a str>,
    /// Sin `editar_catalogo`, el formulario no aparece (y el servidor rechaza el POST igual).
    puede_editar: bool,
    unidades: Vec<UnidadMedida>,
    /// Lo que se escribió en el formulario, para no perderlo si hubo error.
    nombre: &'a str,
    allows_fraction: bool,
    error: Option<String>,
}

/// Solo la sección de la lista y el formulario: lo que htmx reemplaza.
#[derive(Template)]
#[template(path = "unidades.html", block = "unidades")]
struct Seccion<'a> {
    puede_editar: bool,
    unidades: Vec<UnidadMedida>,
    nombre: &'a str,
    allows_fraction: bool,
    error: Option<String>,
}

#[derive(Deserialize)]
pub struct FormUnidad {
    nombre: String,
    /// Una casilla marcada llega como "on"; sin marcar, no llega.
    allows_fraction: Option<String>,
}

pub async fn page(
    State(state): State<AppState>,
    Extension(usuario): Extension<Usuario>,
    headers: HeaderMap,
) -> Response {
    show(&state, &usuario, &headers, "", false, None).await
}

pub async fn add(
    State(state): State<AppState>,
    Extension(usuario): Extension<Usuario>,
    headers: HeaderMap,
    Form(form): Form<FormUnidad>,
) -> Response {
    if !usuario.can(Permiso::EditarCatalogo) {
        return (
            StatusCode::FORBIDDEN,
            Html(
                r#"<div class="alert alert-danger">No tienes permiso para agregar unidades.</div>"#,
            ),
        )
            .into_response();
    }
    let allows_fraction = form.allows_fraction.is_some();
    let resultado = match NuevaUnidadMedida::new(&form.nombre, allows_fraction) {
        Ok(nueva) => state.unidades.add(nueva).await.map(|_| ()),
        Err(error) => Err(error),
    };
    match resultado {
        // Sin JavaScript: de vuelta a la lista, para que recargar no reenvíe el formulario.
        Ok(()) if !is_htmx(&headers) => Redirect::to("/unidades").into_response(),
        Ok(()) => show(&state, &usuario, &headers, "", false, None).await,
        Err(UnidadMedidaError::Repo(error)) => falla_interna(&headers, &error),
        Err(error) => {
            show(
                &state,
                &usuario,
                &headers,
                &form.nombre,
                allows_fraction,
                Some(error.to_string()),
            )
            .await
        }
    }
}

fn is_htmx(headers: &HeaderMap) -> bool {
    headers.contains_key("hx-request")
}

/// La lista con el formulario: completa, o solo la sección si la pidió htmx. Con error, 422.
async fn show(
    state: &AppState,
    usuario: &Usuario,
    headers: &HeaderMap,
    nombre: &str,
    allows_fraction: bool,
    error: Option<String>,
) -> Response {
    let unidades = match state.unidades.list().await {
        Ok(unidades) => unidades,
        Err(error) => return falla_interna(headers, &error),
    };
    let status = if error.is_some() {
        StatusCode::UNPROCESSABLE_ENTITY
    } else {
        StatusCode::OK
    };
    let puede_editar = usuario.can(Permiso::EditarCatalogo);
    let html = if is_htmx(headers) {
        Seccion {
            puede_editar,
            unidades,
            nombre,
            allows_fraction,
            error,
        }
        .render()
    } else {
        Pagina {
            negocio: &state.negocio,
            usuario_nombre: Some(&usuario.nombre),
            puede_editar,
            unidades,
            nombre,
            allows_fraction,
            error,
        }
        .render()
    };
    match html {
        Ok(html) => (status, Html(html)).into_response(),
        Err(error) => falla_interna(headers, &error),
    }
}

#[cfg(test)]
mod tests {
    use crate::test_support::*;
    use axum::body::Body;
    use axum::http::{Request, StatusCode, header};
    use db::PgPool;

    fn post(cookie: &str, form: &str, con_htmx: bool) -> Request<Body> {
        let mut request = Request::post("/unidades")
            .header("cookie", cookie)
            .header(header::CONTENT_TYPE, "application/x-www-form-urlencoded");
        if con_htmx {
            request = request.header("hx-request", "true");
        }
        request.body(Body::from(form.to_string())).unwrap()
    }

    #[sqlx::test(migrator = "db::MIGRATOR")]
    async fn la_pagina_muestra_las_unidades_base(pool: PgPool) {
        let cookie = cookie(&pool, "ana@x.mx", "Dueño").await;
        let respuesta = get_con(pool, "/unidades", &cookie).await;
        assert_eq!(respuesta.status(), StatusCode::OK);

        let html = body_text(respuesta).await;
        assert!(html.contains("<title>Unidades de medida · Papelería de prueba</title>"));
        for unidad in ["Gramo", "Hora", "Metro", "Pieza"] {
            assert!(
                html.contains(&format!("<td>{unidad}</td>")),
                "falta {unidad}"
            );
        }
    }

    #[sqlx::test(migrator = "db::MIGRATOR")]
    async fn agregar_con_htmx_regresa_solo_la_seccion_con_la_nueva(pool: PgPool) {
        let cookie = cookie(&pool, "ana@x.mx", "Dueño").await;
        let respuesta = send(pool, post(&cookie, "nombre=Hoja&allows_fraction=on", true)).await;
        assert_eq!(respuesta.status(), StatusCode::OK);

        let html = body_text(respuesta).await;
        assert!(html.contains("<td>Hoja</td>"));
        assert!(!html.contains("<html"), "debe ser solo la sección");
        assert!(html.contains(r#"value="""#), "el formulario queda vacío");
    }

    #[sqlx::test(migrator = "db::MIGRATOR")]
    async fn agregar_sin_htmx_regresa_a_la_lista(pool: PgPool) {
        let cookie = cookie(&pool, "ana@x.mx", "Dueño").await;
        let respuesta = send(pool.clone(), post(&cookie, "nombre=Hoja", false)).await;
        assert_eq!(respuesta.status(), StatusCode::SEE_OTHER);
        assert_eq!(respuesta.headers()[header::LOCATION], "/unidades");

        let html = body_text(get_con(pool, "/unidades", &cookie).await).await;
        assert!(html.contains("<td>Hoja</td>"));
    }

    #[sqlx::test(migrator = "db::MIGRATOR")]
    async fn un_nombre_vacio_regresa_422_con_el_mensaje(pool: PgPool) {
        let cookie = cookie(&pool, "ana@x.mx", "Dueño").await;
        let respuesta = send(pool, post(&cookie, "nombre=+++", true)).await;
        assert_eq!(respuesta.status(), StatusCode::UNPROCESSABLE_ENTITY);
        assert!(
            body_text(respuesta)
                .await
                .contains("Escribe el nombre de la unidad.")
        );
    }

    #[sqlx::test(migrator = "db::MIGRATOR")]
    async fn un_nombre_repetido_regresa_422_y_conserva_lo_escrito(pool: PgPool) {
        let cookie = cookie(&pool, "ana@x.mx", "Dueño").await;
        let respuesta = send(pool, post(&cookie, "nombre=pieza&allows_fraction=on", true)).await;
        assert_eq!(respuesta.status(), StatusCode::UNPROCESSABLE_ENTITY);

        let html = body_text(respuesta).await;
        assert!(html.contains("Ya existe la unidad «pieza»."));
        assert!(html.contains(r#"value="pieza""#));
        assert!(html.contains("checked"));
    }

    #[sqlx::test(migrator = "db::MIGRATOR")]
    async fn un_cajero_ve_la_lista_pero_no_el_formulario(pool: PgPool) {
        let cookie = cookie(&pool, "ana@x.mx", "Cajero").await;
        let html = body_text(get_con(pool, "/unidades", &cookie).await).await;
        assert!(html.contains("<td>Pieza</td>"));
        assert!(!html.contains(r#"action="/unidades""#));
    }

    #[sqlx::test(migrator = "db::MIGRATOR")]
    async fn un_cajero_no_puede_agregar_aunque_mande_el_formulario(pool: PgPool) {
        let cookie = cookie(&pool, "ana@x.mx", "Cajero").await;

        let respuesta = send(pool.clone(), post(&cookie, "nombre=Hoja", true)).await;

        assert_eq!(respuesta.status(), StatusCode::FORBIDDEN);
        let html = body_text(get_con(pool, "/unidades", &cookie).await).await;
        assert!(!html.contains("<td>Hoja</td>"));
    }

    #[sqlx::test(migrator = "db::MIGRATOR")]
    async fn si_falla_la_lista_se_ve_el_id_para_reportarlo(pool: PgPool) {
        let cookie = cookie(&pool, "ana@x.mx", "Dueño").await;
        sqlx::query("DROP TABLE unidades_medida")
            .execute(&pool)
            .await
            .unwrap();

        let respuesta = get_con(pool, "/unidades", &cookie).await;

        assert_eq!(respuesta.status(), StatusCode::INTERNAL_SERVER_ERROR);
        let id = respuesta.headers()["x-request-id"]
            .to_str()
            .unwrap()
            .to_string();
        assert!(body_text(respuesta).await.contains(&id));
    }
}
