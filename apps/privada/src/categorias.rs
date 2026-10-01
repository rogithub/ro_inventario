//! Pantalla de categorías: la lista, agregar una y renombrarla (en `rename`).

mod rename;

pub use rename::rename;

use askama::Template;
use axum::extract::{Query, State};
use axum::http::{HeaderMap, StatusCode};
use axum::response::{Html, IntoResponse, Redirect, Response};
use axum::{Extension, Form};
use inventario::categorias::{
    Categoria, CategoriaError, CategoriaId, CategoriasRepo, NewCategoria,
};
use serde::Deserialize;
use usuarios::permisos::Permiso;
use usuarios::usuarios::Usuario;

use crate::{AppState, internal_error, is_htmx};

#[derive(Template)]
#[template(path = "categorias.html")]
struct CategoriasPage<'a> {
    negocio: &'a str,
    usuario_nombre: Option<&'a str>,
    /// Sin `editar_catalogo`, el formulario no aparece (y el servidor rechaza el POST igual).
    can_edit: bool,
    categorias: Vec<Categoria>,
    /// Lo que se escribió en el formulario, para no perderlo si hubo error.
    nombre: &'a str,
    error: Option<String>,
    /// El renglón que se está renombrando, con lo escrito y su error.
    editing: Option<Editing>,
    /// Un aviso sobre la lista (la categoría que se quería renombrar ya no existe).
    alert: Option<String>,
}

/// Solo la sección de la lista y el formulario: lo que htmx reemplaza.
#[derive(Template)]
#[template(path = "categorias.html", block = "categorias")]
struct CategoriasSection<'a> {
    can_edit: bool,
    categorias: Vec<Categoria>,
    nombre: &'a str,
    error: Option<String>,
    editing: Option<Editing>,
    alert: Option<String>,
}

/// Un renglón en modo "renombrar".
struct Editing {
    id: CategoriaId,
    nombre: String,
    error: Option<String>,
}

impl Editing {
    fn is(&self, id: &CategoriaId) -> bool {
        self.id == *id
    }
}

#[derive(Deserialize)]
pub struct CategoriaForm {
    nombre: String,
}

#[derive(Deserialize)]
pub struct PageQuery {
    /// `?edit={id}`: ese renglón aparece listo para renombrar.
    edit: Option<String>,
}

pub async fn page(
    State(state): State<AppState>,
    Extension(usuario): Extension<Usuario>,
    headers: HeaderMap,
    Query(query): Query<PageQuery>,
) -> Response {
    let editing = query
        .edit
        .as_deref()
        .and_then(CategoriaId::parse)
        .map(|id| Editing {
            id,
            nombre: String::new(),
            error: None,
        });
    show(&state, &usuario, &headers, "", None, editing).await
}

pub async fn add(
    State(state): State<AppState>,
    Extension(usuario): Extension<Usuario>,
    headers: HeaderMap,
    Form(form): Form<CategoriaForm>,
) -> Response {
    if !usuario.can(Permiso::EditarCatalogo) {
        return (
            StatusCode::FORBIDDEN,
            Html(
                r#"<div class="alert alert-danger">No tienes permiso para agregar categorías.</div>"#,
            ),
        )
            .into_response();
    }
    let result = match NewCategoria::new(&form.nombre) {
        Ok(new_categoria) => state.categorias.add(new_categoria).await.map(|_| ()),
        Err(error) => Err(error),
    };
    match result {
        // Sin JavaScript: de vuelta a la lista, para que recargar no reenvíe el formulario.
        Ok(()) if !is_htmx(&headers) => Redirect::to("/categorias").into_response(),
        Ok(()) => show(&state, &usuario, &headers, "", None, None).await,
        Err(CategoriaError::Repo(error)) => internal_error(&headers, &error),
        Err(error) => {
            show(
                &state,
                &usuario,
                &headers,
                &form.nombre,
                Some(error.to_string()),
                None,
            )
            .await
        }
    }
}

/// La lista con el formulario: completa, o solo la sección si la pidió htmx. Con error, 422.
/// `editing` sin nombre escrito toma el actual; si no es de ninguna categoría, o el usuario no
/// puede editar, no se muestra.
async fn show(
    state: &AppState,
    usuario: &Usuario,
    headers: &HeaderMap,
    nombre: &str,
    error: Option<String>,
    editing: Option<Editing>,
) -> Response {
    let categorias = match state.categorias.list().await {
        Ok(categorias) => categorias,
        Err(error) => return internal_error(headers, &error),
    };
    let can_edit = usuario.can(Permiso::EditarCatalogo);
    let mut alert = None;
    let editing = editing.filter(|_| can_edit).and_then(|mut editing| {
        let Some(actual) = categorias.iter().find(|c| c.id == editing.id) else {
            // Sin renglón donde mostrar el error, va como aviso.
            alert = editing.error;
            return None;
        };
        if editing.nombre.is_empty() && editing.error.is_none() {
            editing.nombre.clone_from(&actual.nombre);
        }
        Some(editing)
    });
    let failed =
        error.is_some() || alert.is_some() || editing.as_ref().is_some_and(|e| e.error.is_some());
    let status = if failed {
        StatusCode::UNPROCESSABLE_ENTITY
    } else {
        StatusCode::OK
    };
    let html = if is_htmx(headers) {
        CategoriasSection {
            can_edit,
            categorias,
            nombre,
            error,
            editing,
            alert,
        }
        .render()
    } else {
        CategoriasPage {
            negocio: &state.negocio,
            usuario_nombre: Some(&usuario.nombre),
            can_edit,
            categorias,
            nombre,
            error,
            editing,
            alert,
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
        sqlx::query("DROP TABLE categorias")
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
}
