//! Pantalla de productos: la lista, con búsqueda por nombre o NID y filtro por categoría. El alta
//! está en `nuevo`.

mod form;
pub mod nuevo;

use askama::Template;
use axum::Extension;
use axum::extract::{Query, State};
use axum::http::{HeaderMap, header};
use axum::response::{Html, IntoResponse, Response};
use inventario::categorias::{Categoria, CategoriaId, CategoriasRepo};
use inventario::productos::search::{MAX_RESULTS, SearchQuery};
use inventario::productos::{Producto, ProductosRepo};
use serde::Deserialize;
use usuarios::permisos::Permiso;
use usuarios::usuarios::Usuario;

use crate::{AppState, internal_error, is_htmx};

#[derive(Template)]
#[template(path = "productos.html")]
struct ProductosPage<'a> {
    negocio: &'a str,
    usuario_nombre: Option<&'a str>,
    /// Sin `editar_catalogo`, el botón de alta no aparece (y el servidor la rechaza igual).
    can_edit: bool,
    /// Lo escrito en el buscador, para dejarlo ahí.
    q: &'a str,
    categoria: Option<CategoriaId>,
    categorias: Vec<Categoria>,
    results: SearchResults,
}

impl ProductosPage<'_> {
    fn is_selected(&self, id: &CategoriaId) -> bool {
        self.categoria == Some(*id)
    }
}

/// Solo los resultados: lo que htmx reemplaza mientras se escribe (el buscador se queda).
#[derive(Template)]
#[template(path = "productos.html", block = "results")]
struct ResultsSection {
    results: SearchResults,
}

/// Lo que se pinta de una búsqueda.
struct SearchResults {
    rows: Vec<Row>,
    /// Si no caben todos: cuántos se muestran de cuántos.
    notice: Option<String>,
    /// Hay búsqueda o filtro: una lista vacía es "no coincide", no "no hay".
    is_filtered: bool,
}

struct Row {
    producto: Producto,
    categoria: String,
}

#[derive(Deserialize)]
pub struct PageQuery {
    #[serde(default)]
    q: String,
    /// El id de la categoría; vacío o algo que no es un id = todas.
    categoria: Option<String>,
}

pub async fn page(
    State(state): State<AppState>,
    Extension(usuario): Extension<Usuario>,
    headers: HeaderMap,
    Query(query): Query<PageQuery>,
) -> Response {
    let search_query = SearchQuery::new(&query.q);
    let categoria = query.categoria.as_deref().and_then(CategoriaId::parse);
    let categorias = match state.categorias.list().await {
        Ok(categorias) => categorias,
        Err(error) => return internal_error(&headers, &error),
    };
    let found = match state
        .productos
        .search(&search_query, categoria, MAX_RESULTS)
        .await
    {
        Ok(found) => found,
        Err(error) => return internal_error(&headers, &error),
    };
    let notice = (found.total > found.productos.len() as u64).then(|| {
        format!(
            "Se muestran {} de {}; escribe para encontrar los demás.",
            found.productos.len(),
            thousands(found.total)
        )
    });
    let rows = found
        .productos
        .into_iter()
        .map(|producto| Row {
            categoria: categorias
                .iter()
                .find(|c| c.id == producto.categoria_id)
                .map(|c| c.nombre.clone())
                .unwrap_or_default(),
            producto,
        })
        .collect();
    let results = SearchResults {
        rows,
        notice,
        is_filtered: !search_query.is_empty() || categoria.is_some(),
    };
    let html = if is_htmx(&headers) {
        ResultsSection { results }.render()
    } else {
        ProductosPage {
            negocio: &state.negocio,
            usuario_nombre: Some(&usuario.nombre),
            can_edit: usuario.can(Permiso::EditarCatalogo),
            q: &query.q,
            categoria,
            categorias,
            results,
        }
        .render()
    };
    match html {
        // La misma URL da la página o solo los resultados (ver `is_htmx`): que el navegador no
        // guarde una por otra.
        Ok(html) => (
            [(header::VARY, "HX-Request, HX-History-Restore-Request")],
            Html(html),
        )
            .into_response(),
        Err(error) => internal_error(&headers, &error),
    }
}

/// 2336 → "2,336".
fn thousands(n: u64) -> String {
    let digits = n.to_string();
    let mut text = String::with_capacity(digits.len() + digits.len() / 3);
    for (i, c) in digits.chars().enumerate() {
        if i > 0 && (digits.len() - i).is_multiple_of(3) {
            text.push(',');
        }
        text.push(c);
    }
    text
}

#[cfg(test)]
mod tests;
