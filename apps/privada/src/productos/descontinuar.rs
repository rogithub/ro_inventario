//! Descontinuar un producto desde su edición (`POST /productos/{nid}/descontinuar`) y volver a
//! manejarlo (`/volver-a-manejar`). Sin htmx: al terminar regresa a la edición, con la búsqueda
//! de donde se llegó, y recargar no reenvía nada.

use axum::extract::{Path, Query, State};
use axum::http::HeaderMap;
use axum::response::{IntoResponse, Redirect, Response};
use axum::{Extension, Form};
use chrono_tz::Tz;
use inventario::articulos::Nid;
use inventario::productos::discontinue::MotivoDescontinuado;
use inventario::productos::{Producto, ProductoError, ProductosRepo};
use serde::Deserialize;
use usuarios::permisos::Permiso;
use usuarios::usuarios::Usuario;

use super::ListSearch;
use super::editar::{self, PageQuery};
use super::form::ProductoForm;
use crate::dates::short_date;
use crate::{AppState, internal_error};

/// Lo de descontinuar en la edición: si está descontinuado, la franja con «Volver a manejar»;
/// si no, el motivo y el botón.
pub struct DescontinuadoSection {
    /// A dónde va el botón, con la búsqueda.
    pub action: String,
    /// «Descontinuado el 3 oct 2026: el proveedor ya no lo vende».
    pub banner: Option<String>,
    /// Lo escrito en el motivo, para no perderlo si no se pudo.
    pub motivo: String,
    /// Junto al motivo.
    pub error: Option<String>,
    /// Lo que no es del motivo (por ejemplo, ya estaba descontinuado).
    pub notice: Option<String>,
}

impl DescontinuadoSection {
    pub fn new(producto: &Producto, time_zone: Tz, search: &ListSearch) -> Self {
        let (path, banner) = match &producto.descontinuado {
            Some(descontinuado) => (
                "volver-a-manejar",
                Some(format!(
                    "Descontinuado el {}: {}",
                    short_date(descontinuado.at, time_zone),
                    descontinuado.motivo
                )),
            ),
            None => ("descontinuar", None),
        };
        Self {
            action: search.url(format!("/productos/{}/{path}", producto.nid)),
            banner,
            motivo: String::new(),
            error: None,
            notice: None,
        }
    }

    pub fn has_error(&self) -> bool {
        self.error.is_some() || self.notice.is_some()
    }
}

#[derive(Deserialize)]
pub struct DiscontinueForm {
    #[serde(default)]
    motivo: String,
}

pub async fn discontinue(
    State(state): State<AppState>,
    Extension(usuario): Extension<Usuario>,
    headers: HeaderMap,
    Path(nid): Path<String>,
    Query(query): Query<PageQuery>,
    Form(values): Form<DiscontinueForm>,
) -> Response {
    if !usuario.can(Permiso::EditarCatalogo) {
        return editar::forbidden();
    }
    let producto = match editar::find(&state, &nid).await {
        Ok(Some(producto)) => producto,
        Ok(None) => return editar::not_found(),
        Err(error) => return internal_error(&headers, &error),
    };
    let result = match MotivoDescontinuado::parse(&values.motivo) {
        Ok(motivo) => {
            state
                .productos
                .discontinue(producto.id, motivo, &usuario.email)
                .await
        }
        Err(error) => Err(error),
    };
    let search = query.search();
    match result {
        Ok(_) => back(&search, producto.nid),
        // Ya descontinuado, no hay motivo junto al cual decirlo (una pestaña vieja): se avisa eso.
        Err(ProductoError::EmptyMotivo | ProductoError::LongMotivo)
            if producto.descontinuado.is_some() =>
        {
            let error = ProductoError::AlreadyDescontinuado;
            failed(&state, &usuario, &headers, search, producto.nid, error).await
        }
        Err(error @ (ProductoError::EmptyMotivo | ProductoError::LongMotivo)) => {
            let mut section = DescontinuadoSection::new(&producto, state.time_zone, &search);
            section.motivo = values.motivo;
            section.error = Some(error.to_string());
            show(&state, &usuario, &headers, search, &producto, section).await
        }
        Err(error) => failed(&state, &usuario, &headers, search, producto.nid, error).await,
    }
}

pub async fn resume(
    State(state): State<AppState>,
    Extension(usuario): Extension<Usuario>,
    headers: HeaderMap,
    Path(nid): Path<String>,
    Query(query): Query<PageQuery>,
) -> Response {
    if !usuario.can(Permiso::EditarCatalogo) {
        return editar::forbidden();
    }
    let producto = match editar::find(&state, &nid).await {
        Ok(Some(producto)) => producto,
        Ok(None) => return editar::not_found(),
        Err(error) => return internal_error(&headers, &error),
    };
    let search = query.search();
    match state.productos.resume(producto.id, &usuario.email).await {
        Ok(_) => back(&search, producto.nid),
        Err(error) => failed(&state, &usuario, &headers, search, producto.nid, error).await,
    }
}

/// A la edición, con la búsqueda.
fn back(search: &ListSearch, nid: Nid) -> Response {
    Redirect::to(&search.url(format!("/productos/{nid}"))).into_response()
}

/// La edición con el aviso, leída otra vez: si ya estaba descontinuado, con la fecha que tiene.
async fn failed(
    state: &AppState,
    usuario: &Usuario,
    headers: &HeaderMap,
    search: ListSearch,
    nid: Nid,
    error: ProductoError,
) -> Response {
    if let ProductoError::Repo(error) = error {
        return internal_error(headers, &error);
    }
    let producto = match state.productos.get_by_nid(nid).await {
        Ok(Some(producto)) => producto,
        Ok(None) => return editar::not_found(),
        Err(error) => return internal_error(headers, &error),
    };
    let mut section = DescontinuadoSection::new(&producto, state.time_zone, &search);
    section.notice = Some(error.to_string());
    show(state, usuario, headers, search, &producto, section).await
}

/// La página de edición completa, con el formulario del producto como está guardado.
async fn show(
    state: &AppState,
    usuario: &Usuario,
    headers: &HeaderMap,
    search: ListSearch,
    producto: &Producto,
    section: DescontinuadoSection,
) -> Response {
    let values = ProductoForm::from(producto);
    let form = editar::form_view(state, producto.nid, &search, values, None, None).await;
    editar::show(state, usuario, headers, search, producto.nid, form, section).await
}

#[cfg(test)]
mod tests;
