//! Editar un producto del catálogo: `/productos/{nid}`. Al guardar, se queda en la página con el
//! aviso; la flecha regresa a la lista con la búsqueda de donde se llegó.

use askama::Template;
use axum::extract::{Path, Query, State};
use axum::http::{HeaderMap, StatusCode};
use axum::response::{Html, IntoResponse, Redirect, Response};
use axum::{Extension, Form};
use inventario::articulos::Nid;
use inventario::categorias::{CategoriaId, CategoriasRepo};
use inventario::productos::{NewProducto, Producto, ProductoError, ProductosRepo};
use inventario::unidades_medida::UnidadesMedidaRepo;
use kernel::RepoError;
use serde::Deserialize;
use usuarios::permisos::Permiso;
use usuarios::usuarios::Usuario;

use super::ListSearch;
use super::form::{FieldErrors, ProductoForm, ProductoFormView};
use crate::{AppState, internal_error, is_htmx};

#[derive(Template)]
#[template(path = "productos_editar.html")]
struct EditProductoPage<'a> {
    negocio: &'a str,
    usuario_nombre: Option<&'a str>,
    nid: Nid,
    /// La búsqueda de la lista, para regresar a ella.
    search: ListSearch,
    form: ProductoFormView,
}

/// Solo el formulario: lo que htmx reemplaza.
#[derive(Template)]
#[template(path = "productos_editar.html", block = "form")]
struct EditProductoSection {
    form: ProductoFormView,
}

/// La búsqueda de la lista de donde se llegó: viaja en la dirección de la página, del formulario y
/// de la redirección, para que la flecha regrese a ella también sin JavaScript.
#[derive(Deserialize)]
pub struct PageQuery {
    /// Sin JavaScript, después de guardar: para el aviso.
    saved: Option<String>,
    #[serde(default)]
    q: String,
    /// El id de la categoría; algo que no es un id = sin filtro.
    categoria: Option<String>,
}

impl PageQuery {
    fn search(&self) -> ListSearch {
        ListSearch {
            q: self.q.clone(),
            categoria: self.categoria.as_deref().and_then(CategoriaId::parse),
        }
    }
}

pub async fn page(
    State(state): State<AppState>,
    Extension(usuario): Extension<Usuario>,
    headers: HeaderMap,
    Path(nid): Path<String>,
    Query(query): Query<PageQuery>,
) -> Response {
    if !usuario.can(Permiso::EditarCatalogo) {
        return forbidden();
    }
    let producto = match find(&state, &nid).await {
        Ok(Some(producto)) => producto,
        Ok(None) => return not_found(),
        Err(error) => return internal_error(&headers, &error),
    };
    let notice = query.saved.is_some().then(|| notice(&producto));
    let search = query.search();
    let form = form_view(
        &state,
        producto.nid,
        &search,
        ProductoForm::from(&producto),
        None,
        notice,
    )
    .await;
    show(&state, &usuario, &headers, search, producto.nid, form).await
}

pub async fn update(
    State(state): State<AppState>,
    Extension(usuario): Extension<Usuario>,
    headers: HeaderMap,
    Path(nid): Path<String>,
    Query(query): Query<PageQuery>,
    Form(values): Form<ProductoForm>,
) -> Response {
    if !usuario.can(Permiso::EditarCatalogo) {
        return forbidden();
    }
    let producto = match find(&state, &nid).await {
        Ok(Some(producto)) => producto,
        Ok(None) => return not_found(),
        Err(error) => return internal_error(&headers, &error),
    };
    let result = match values.fields().and_then(NewProducto::new) {
        Ok(new) => {
            state
                .productos
                .update(producto.id, new, &usuario.email)
                .await
        }
        Err(error) => Err(error),
    };
    let search = query.search();
    match result {
        // Sin JavaScript: de vuelta a la página, para que recargar no reenvíe el formulario.
        Ok(updated) if !is_htmx(&headers) => {
            Redirect::to(&search.url(format!("/productos/{}?saved=1", updated.nid))).into_response()
        }
        Ok(updated) => {
            let values = ProductoForm::from(&updated);
            let notice = Some(notice(&updated));
            let form = form_view(&state, updated.nid, &search, values, None, notice).await;
            show(&state, &usuario, &headers, search, updated.nid, form).await
        }
        Err(ProductoError::Repo(error)) => internal_error(&headers, &error),
        Err(error) => {
            let form = form_view(&state, producto.nid, &search, values, Some(&error), None).await;
            show(&state, &usuario, &headers, search, producto.nid, form).await
        }
    }
}

/// Lo que no es un NID no es de ningún producto.
async fn find(state: &AppState, nid: &str) -> Result<Option<Producto>, RepoError> {
    match Nid::parse(nid) {
        Some(nid) => state.productos.get_by_nid(nid).await,
        None => Ok(None),
    }
}

/// El formulario de ese producto, con sus listas para elegir.
async fn form_view(
    state: &AppState,
    nid: Nid,
    search: &ListSearch,
    values: ProductoForm,
    error: Option<&ProductoError>,
    notice: Option<String>,
) -> Result<ProductoFormView, RepoError> {
    Ok(ProductoFormView {
        action: search.url(format!("/productos/{nid}")),
        categorias: state.categorias.list().await?,
        unidades: state.unidades_medida.list().await?,
        values,
        errors: error.map(FieldErrors::from_error).unwrap_or_default(),
        notice,
        autofocus: false,
    })
}

/// La página completa, o solo el formulario si lo pidió htmx. Con error, 422.
async fn show(
    state: &AppState,
    usuario: &Usuario,
    headers: &HeaderMap,
    search: ListSearch,
    nid: Nid,
    form: Result<ProductoFormView, RepoError>,
) -> Response {
    let form = match form {
        Ok(form) => form,
        Err(error) => return internal_error(headers, &error),
    };
    let status = if form.errors.is_empty() {
        StatusCode::OK
    } else {
        StatusCode::UNPROCESSABLE_ENTITY
    };
    let html = if is_htmx(headers) {
        EditProductoSection { form }.render()
    } else {
        EditProductoPage {
            negocio: &state.negocio,
            usuario_nombre: Some(&usuario.nombre),
            nid,
            search,
            form,
        }
        .render()
    };
    match html {
        Ok(html) => (status, Html(html)).into_response(),
        Err(error) => internal_error(headers, &error),
    }
}

fn notice(producto: &Producto) -> String {
    format!("Se guardaron los cambios del NID {}.", producto.nid)
}

fn forbidden() -> Response {
    (
        StatusCode::FORBIDDEN,
        Html(r#"<div class="alert alert-danger">No tienes permiso para editar productos.</div>"#),
    )
        .into_response()
}

fn not_found() -> Response {
    (
        StatusCode::NOT_FOUND,
        Html(r#"<div class="alert alert-warning">No hay ningún producto con ese NID.</div>"#),
    )
        .into_response()
}

#[cfg(test)]
mod tests;
