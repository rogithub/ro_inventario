//! Pantalla de unidades de medida: la lista y agregar una.

use askama::Template;
use axum::Form;
use axum::extract::State;
use axum::http::{HeaderMap, StatusCode};
use axum::response::{Html, IntoResponse, Redirect, Response};
use inventario::unidades_medida::{
    NuevaUnidadMedida, UnidadMedida, UnidadMedidaError, UnidadesMedidaRepo,
};
use serde::Deserialize;

use crate::{AppState, falla_interna};

#[derive(Template)]
#[template(path = "unidades.html")]
struct Pagina<'a> {
    negocio: &'a str,
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

pub async fn page(State(state): State<AppState>, headers: HeaderMap) -> Response {
    show(&state, &headers, "", false, None).await
}

pub async fn add(
    State(state): State<AppState>,
    headers: HeaderMap,
    Form(form): Form<FormUnidad>,
) -> Response {
    let allows_fraction = form.allows_fraction.is_some();
    let resultado = match NuevaUnidadMedida::new(&form.nombre, allows_fraction) {
        Ok(nueva) => state.unidades.add(nueva).await.map(|_| ()),
        Err(error) => Err(error),
    };
    match resultado {
        // Sin JavaScript: de vuelta a la lista, para que recargar no reenvíe el formulario.
        Ok(()) if !is_htmx(&headers) => Redirect::to("/unidades").into_response(),
        Ok(()) => show(&state, &headers, "", false, None).await,
        Err(UnidadMedidaError::Repo(error)) => falla_interna(&headers, &error),
        Err(error) => {
            show(
                &state,
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
    let html = if is_htmx(headers) {
        Seccion {
            unidades,
            nombre,
            allows_fraction,
            error,
        }
        .render()
    } else {
        Pagina {
            negocio: &state.negocio,
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

    fn post(form: &str, con_htmx: bool) -> Request<Body> {
        let mut request = Request::post("/unidades")
            .header(header::CONTENT_TYPE, "application/x-www-form-urlencoded");
        if con_htmx {
            request = request.header("hx-request", "true");
        }
        request.body(Body::from(form.to_string())).unwrap()
    }

    #[sqlx::test(migrator = "db::MIGRATOR")]
    async fn la_pagina_muestra_las_unidades_base(pool: PgPool) {
        let respuesta = get(pool, "/unidades").await;
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
        let respuesta = send(pool, post("nombre=Hoja&allows_fraction=on", true)).await;
        assert_eq!(respuesta.status(), StatusCode::OK);

        let html = body_text(respuesta).await;
        assert!(html.contains("<td>Hoja</td>"));
        assert!(!html.contains("<html"), "debe ser solo la sección");
        assert!(html.contains(r#"value="""#), "el formulario queda vacío");
    }

    #[sqlx::test(migrator = "db::MIGRATOR")]
    async fn agregar_sin_htmx_regresa_a_la_lista(pool: PgPool) {
        let respuesta = send(pool.clone(), post("nombre=Hoja", false)).await;
        assert_eq!(respuesta.status(), StatusCode::SEE_OTHER);
        assert_eq!(respuesta.headers()[header::LOCATION], "/unidades");

        let html = body_text(get(pool, "/unidades").await).await;
        assert!(html.contains("<td>Hoja</td>"));
    }

    #[sqlx::test(migrator = "db::MIGRATOR")]
    async fn un_nombre_vacio_regresa_422_con_el_mensaje(pool: PgPool) {
        let respuesta = send(pool, post("nombre=+++", true)).await;
        assert_eq!(respuesta.status(), StatusCode::UNPROCESSABLE_ENTITY);
        assert!(
            body_text(respuesta)
                .await
                .contains("Escribe el nombre de la unidad.")
        );
    }

    #[sqlx::test(migrator = "db::MIGRATOR")]
    async fn un_nombre_repetido_regresa_422_y_conserva_lo_escrito(pool: PgPool) {
        let respuesta = send(pool, post("nombre=pieza&allows_fraction=on", true)).await;
        assert_eq!(respuesta.status(), StatusCode::UNPROCESSABLE_ENTITY);

        let html = body_text(respuesta).await;
        assert!(html.contains("Ya existe la unidad «pieza»."));
        assert!(html.contains(r#"value="pieza""#));
        assert!(html.contains("checked"));
    }

    #[tokio::test]
    async fn si_la_base_falla_se_ve_el_id_para_reportarlo() {
        let respuesta = get(pool_sin_base(), "/unidades").await;
        assert_eq!(respuesta.status(), StatusCode::INTERNAL_SERVER_ERROR);

        let id = respuesta.headers()["x-request-id"]
            .to_str()
            .unwrap()
            .to_string();
        assert!(body_text(respuesta).await.contains(&id));
    }
}
