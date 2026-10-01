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
mod tests;
