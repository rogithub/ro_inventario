//! Pantalla de unidades de medida: la lista y agregar una.

use askama::Template;
use axum::extract::State;
use axum::http::{HeaderMap, StatusCode};
use axum::response::{Html, IntoResponse, Redirect, Response};
use axum::{Extension, Form};
use inventario::unidades_medida::{
    NewUnidadMedida, UnidadMedida, UnidadMedidaError, UnidadesMedidaRepo,
};
use serde::Deserialize;
use usuarios::permisos::Permiso;
use usuarios::usuarios::Usuario;

use crate::{AppState, internal_error, is_htmx};

#[derive(Template)]
#[template(path = "unidades.html")]
struct UnidadesPage<'a> {
    negocio: &'a str,
    usuario_nombre: Option<&'a str>,
    /// Sin `editar_catalogo`, el formulario no aparece (y el servidor rechaza el POST igual).
    can_edit: bool,
    unidades: Vec<UnidadMedida>,
    /// Lo que se escribió en el formulario, para no perderlo si hubo error.
    nombre: &'a str,
    allows_fraction: bool,
    error: Option<String>,
}

/// Solo la sección de la lista y el formulario: lo que htmx reemplaza.
#[derive(Template)]
#[template(path = "unidades.html", block = "unidades")]
struct UnidadesSection<'a> {
    can_edit: bool,
    unidades: Vec<UnidadMedida>,
    nombre: &'a str,
    allows_fraction: bool,
    error: Option<String>,
}

#[derive(Deserialize)]
pub struct UnidadForm {
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
    Form(form): Form<UnidadForm>,
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
    let result = match NewUnidadMedida::new(&form.nombre, allows_fraction) {
        Ok(new_unidad) => state.unidades_medida.add(new_unidad).await.map(|_| ()),
        Err(error) => Err(error),
    };
    match result {
        // Sin JavaScript: de vuelta a la lista, para que recargar no reenvíe el formulario.
        Ok(()) if !is_htmx(&headers) => Redirect::to("/unidades").into_response(),
        Ok(()) => show(&state, &usuario, &headers, "", false, None).await,
        Err(UnidadMedidaError::Repo(error)) => internal_error(&headers, &error),
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

/// La lista con el formulario: completa, o solo la sección si la pidió htmx. Con error, 422.
async fn show(
    state: &AppState,
    usuario: &Usuario,
    headers: &HeaderMap,
    nombre: &str,
    allows_fraction: bool,
    error: Option<String>,
) -> Response {
    let unidades = match state.unidades_medida.list().await {
        Ok(unidades) => unidades,
        Err(error) => return internal_error(headers, &error),
    };
    let status = if error.is_some() {
        StatusCode::UNPROCESSABLE_ENTITY
    } else {
        StatusCode::OK
    };
    let can_edit = usuario.can(Permiso::EditarCatalogo);
    let html = if is_htmx(headers) {
        UnidadesSection {
            can_edit,
            unidades,
            nombre,
            allows_fraction,
            error,
        }
        .render()
    } else {
        UnidadesPage {
            negocio: &state.negocio,
            usuario_nombre: Some(&usuario.nombre),
            can_edit,
            unidades,
            nombre,
            allows_fraction,
            error,
        }
        .render()
    };
    match html {
        Ok(html) => (status, Html(html)).into_response(),
        Err(error) => internal_error(headers, &error),
    }
}

#[cfg(test)]
mod tests {
    use crate::test_support::*;
    use axum::body::Body;
    use axum::http::{Request, StatusCode, header};
    use db::PgPool;

    fn post(cookie: &str, form: &str, htmx: bool) -> Request<Body> {
        let mut request = Request::post("/unidades")
            .header("cookie", cookie)
            .header(header::CONTENT_TYPE, "application/x-www-form-urlencoded");
        if htmx {
            request = request.header("hx-request", "true");
        }
        request.body(Body::from(form.to_string())).unwrap()
    }

    #[sqlx::test(migrator = "db::MIGRATOR")]
    async fn la_pagina_muestra_las_unidades_base(pool: PgPool) {
        let cookie = cookie(&pool, "ana@x.mx", "Dueño").await;
        let response = get_with_cookie(pool, "/unidades", &cookie).await;
        assert_eq!(response.status(), StatusCode::OK);

        let html = body_text(response).await;
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
        let response = send(pool, post(&cookie, "nombre=Hoja&allows_fraction=on", true)).await;
        assert_eq!(response.status(), StatusCode::OK);

        let html = body_text(response).await;
        assert!(html.contains("<td>Hoja</td>"));
        assert!(!html.contains("<html"), "debe ser solo la sección");
        assert!(html.contains(r#"value="""#), "el formulario queda vacío");
    }

    #[sqlx::test(migrator = "db::MIGRATOR")]
    async fn agregar_sin_htmx_regresa_a_la_lista(pool: PgPool) {
        let cookie = cookie(&pool, "ana@x.mx", "Dueño").await;
        let response = send(pool.clone(), post(&cookie, "nombre=Hoja", false)).await;
        assert_eq!(response.status(), StatusCode::SEE_OTHER);
        assert_eq!(response.headers()[header::LOCATION], "/unidades");

        let html = body_text(get_with_cookie(pool, "/unidades", &cookie).await).await;
        assert!(html.contains("<td>Hoja</td>"));
    }

    #[sqlx::test(migrator = "db::MIGRATOR")]
    async fn un_nombre_vacio_regresa_422_con_el_mensaje(pool: PgPool) {
        let cookie = cookie(&pool, "ana@x.mx", "Dueño").await;
        let response = send(pool, post(&cookie, "nombre=+++", true)).await;
        assert_eq!(response.status(), StatusCode::UNPROCESSABLE_ENTITY);
        assert!(
            body_text(response)
                .await
                .contains("Escribe el nombre de la unidad.")
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
        let response = send(pool, post(&cookie, "nombre=pieza&allows_fraction=on", true)).await;
        assert_eq!(response.status(), StatusCode::UNPROCESSABLE_ENTITY);

        let html = body_text(response).await;
        assert!(html.contains("Ya existe la unidad «pieza»."));
        assert!(html.contains(r#"value="pieza""#));
        assert!(html.contains("checked"));
    }

    #[sqlx::test(migrator = "db::MIGRATOR")]
    async fn un_cajero_ve_la_lista_pero_no_el_formulario(pool: PgPool) {
        let cookie = cookie(&pool, "ana@x.mx", "Cajero").await;
        let html = body_text(get_with_cookie(pool, "/unidades", &cookie).await).await;
        assert!(html.contains("<td>Pieza</td>"));
        assert!(!html.contains(r#"action="/unidades""#));
    }

    #[sqlx::test(migrator = "db::MIGRATOR")]
    async fn un_cajero_no_puede_agregar_aunque_mande_el_formulario(pool: PgPool) {
        let cookie = cookie(&pool, "ana@x.mx", "Cajero").await;

        let response = send(pool.clone(), post(&cookie, "nombre=Hoja", true)).await;

        assert_eq!(response.status(), StatusCode::FORBIDDEN);
        let html = body_text(get_with_cookie(pool, "/unidades", &cookie).await).await;
        assert!(!html.contains("<td>Hoja</td>"));
    }

    #[sqlx::test(migrator = "db::MIGRATOR")]
    async fn si_falla_la_lista_se_ve_el_id_para_reportarlo(pool: PgPool) {
        let cookie = cookie(&pool, "ana@x.mx", "Dueño").await;
        sqlx::query("DROP TABLE unidades_medida CASCADE")
            .execute(&pool)
            .await
            .unwrap();

        let response = get_with_cookie(pool, "/unidades", &cookie).await;

        assert_eq!(response.status(), StatusCode::INTERNAL_SERVER_ERROR);
        let id = response.headers()["x-request-id"]
            .to_str()
            .unwrap()
            .to_string();
        assert!(body_text(response).await.contains(&id));
    }
}
