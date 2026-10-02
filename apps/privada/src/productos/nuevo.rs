//! Alta de productos en el catálogo: `/productos/nuevo`. Al guardar, el formulario regresa vacío
//! con el aviso del NID, listo para el siguiente.

use askama::Template;
use axum::extract::{Query, State};
use axum::http::{HeaderMap, StatusCode};
use axum::response::{Html, IntoResponse, Redirect, Response};
use axum::{Extension, Form};
use inventario::articulos::Nid;
use inventario::categorias::CategoriasRepo;
use inventario::productos::search::SearchQuery;
use inventario::productos::{NewProducto, Producto, ProductoError, ProductosRepo};
use inventario::unidades_medida::UnidadesMedidaRepo;
use kernel::RepoError;
use serde::Deserialize;
use usuarios::permisos::Permiso;
use usuarios::usuarios::Usuario;

use super::form::{FieldErrors, ProductoForm, ProductoFormView};
use crate::{AppState, internal_error, is_htmx};

/// La unidad que aparece elegida: en la v1, 2,161 de 2,227 productos se venden por pieza.
const DEFAULT_UNIDAD: &str = "Pieza";

#[derive(Template)]
#[template(path = "productos_nuevo.html")]
struct AddProductoPage<'a> {
    negocio: &'a str,
    usuario_nombre: Option<&'a str>,
    form: ProductoFormView,
}

/// Solo el formulario: lo que htmx reemplaza.
#[derive(Template)]
#[template(path = "productos_nuevo.html", block = "form")]
struct AddProductoSection {
    form: ProductoFormView,
}

#[derive(Deserialize)]
pub struct PageQuery {
    /// Sin JavaScript, después de un alta: su NID, para el aviso.
    added: Option<String>,
    /// La categoría y la unidad con que empieza el formulario.
    #[serde(default)]
    categoria: String,
    #[serde(default)]
    unidad: String,
}

pub async fn page(
    State(state): State<AppState>,
    Extension(usuario): Extension<Usuario>,
    headers: HeaderMap,
    Query(query): Query<PageQuery>,
) -> Response {
    if !usuario.can(Permiso::EditarCatalogo) {
        return forbidden();
    }
    let added = match query.added.as_deref().and_then(Nid::parse) {
        Some(nid) => match added_notice(&state, nid).await {
            Ok(added) => added,
            Err(error) => return internal_error(&headers, &error),
        },
        None => None,
    };
    let values = ProductoForm {
        categoria: query.categoria,
        unidad: query.unidad,
        ..ProductoForm::default()
    };
    show(&state, &usuario, &headers, values, None, added).await
}

pub async fn add(
    State(state): State<AppState>,
    Extension(usuario): Extension<Usuario>,
    headers: HeaderMap,
    Form(values): Form<ProductoForm>,
) -> Response {
    if !usuario.can(Permiso::EditarCatalogo) {
        return forbidden();
    }
    let result = match values.fields().and_then(NewProducto::new) {
        Ok(new_producto) => state.productos.add(new_producto, &usuario.email).await,
        Err(error) => Err(error),
    };
    match result {
        // Sin JavaScript: de vuelta al formulario, para que recargar no lo reenvíe.
        Ok(producto) if !is_htmx(&headers) => Redirect::to(&format!(
            "/productos/nuevo?added={}&categoria={}&unidad={}",
            producto.nid, producto.categoria_id, producto.unidad_medida_id
        ))
        .into_response(),
        Ok(producto) => {
            let added = Some(notice(&producto));
            show(&state, &usuario, &headers, values.next(), None, added).await
        }
        Err(ProductoError::Repo(error)) => internal_error(&headers, &error),
        Err(error) => show(&state, &usuario, &headers, values, Some(&error), None).await,
    }
}

/// El formulario: la página completa, o solo él si lo pidió htmx. Con error, 422.
async fn show(
    state: &AppState,
    usuario: &Usuario,
    headers: &HeaderMap,
    mut values: ProductoForm,
    error: Option<&ProductoError>,
    added: Option<String>,
) -> Response {
    let categorias = match state.categorias.list().await {
        Ok(categorias) => categorias,
        Err(error) => return internal_error(headers, &error),
    };
    let unidades = match state.unidades_medida.list().await {
        Ok(unidades) => unidades,
        Err(error) => return internal_error(headers, &error),
    };
    if values.unidad.is_empty()
        && let Some(pieza) = unidades.iter().find(|u| u.nombre == DEFAULT_UNIDAD)
    {
        values.unidad = pieza.id.to_string();
    }
    let status = if error.is_some() {
        StatusCode::UNPROCESSABLE_ENTITY
    } else {
        StatusCode::OK
    };
    let form = ProductoFormView {
        action: "/productos",
        categorias,
        unidades,
        values,
        errors: error.map(FieldErrors::from_error).unwrap_or_default(),
        added,
    };
    let html = if is_htmx(headers) {
        AddProductoSection { form }.render()
    } else {
        AddProductoPage {
            negocio: &state.negocio,
            usuario_nombre: Some(&usuario.nombre),
            form,
        }
        .render()
    };
    match html {
        Ok(html) => (status, Html(html)).into_response(),
        Err(error) => internal_error(headers, &error),
    }
}

/// El aviso del alta con ese NID (sin JavaScript, el formulario lo recibe en la dirección).
async fn added_notice(state: &AppState, nid: Nid) -> Result<Option<String>, RepoError> {
    let found = state
        .productos
        .search(&SearchQuery::new(&nid.to_string()), None, 1)
        .await?;
    Ok(found.productos.iter().find(|p| p.nid == nid).map(notice))
}

fn notice(producto: &Producto) -> String {
    format!("Se agregó el NID {} «{}».", producto.nid, producto.nombre)
}

fn forbidden() -> Response {
    (
        StatusCode::FORBIDDEN,
        Html(r#"<div class="alert alert-danger">No tienes permiso para dar de alta productos.</div>"#),
    )
        .into_response()
}

#[cfg(test)]
mod tests;
