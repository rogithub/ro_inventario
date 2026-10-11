//! Pantalla de productos: la lista, con búsqueda por nombre o NID, filtro por categoría y la
//! casilla «Ver descontinuados». El alta está en `nuevo`; la edición, en `editar`, y descontinuar
//! desde ella, en `descontinuar`.

pub mod descontinuar;
pub mod editar;
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
    with_descontinuados: bool,
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
    /// Con `editar_catalogo`, cada nombre lleva a su edición.
    can_edit: bool,
    /// La búsqueda, para que la edición regrese a ella.
    search: ListSearch,
}

/// La búsqueda de la lista, para llevarla a la edición de un producto y regresar con ella.
#[derive(Default)]
pub struct ListSearch {
    pub q: String,
    pub categoria: Option<CategoriaId>,
    /// La casilla «Ver descontinuados».
    pub with_descontinuados: bool,
}

impl ListSearch {
    /// La dirección con la búsqueda agregada (`/productos?q=pluma%20az&categoria=…&descontinuados=1`);
    /// sin búsqueda, tal cual. `path` puede traer ya su `?`.
    pub fn url(&self, path: impl AsRef<str>) -> String {
        let path = path.as_ref();
        let mut params = Vec::new();
        if !self.q.is_empty() {
            params.push(format!("q={}", encode(&self.q)));
        }
        if let Some(categoria) = self.categoria {
            params.push(format!("categoria={categoria}"));
        }
        if self.with_descontinuados {
            params.push("descontinuados=1".to_string());
        }
        if params.is_empty() {
            return path.to_string();
        }
        let separator = if path.contains('?') { '&' } else { '?' };
        format!("{path}{separator}{}", params.join("&"))
    }
}

/// Para un valor dentro de la dirección: todo lo que no es letra, número o `-_.~` va como `%XX`.
fn encode(text: &str) -> String {
    match askama::filters::urlencode_strict(text) {
        Ok(encoded) => encoded.to_string(),
        Err(never) => match never {},
    }
}

struct Row {
    producto: Producto,
    categoria: String,
}

impl Row {
    /// Un descontinuado se ve en gris.
    fn text_class(&self) -> &'static str {
        if self.producto.descontinuado.is_some() {
            " text-body-secondary"
        } else {
            ""
        }
    }
}

#[derive(Deserialize)]
pub struct PageQuery {
    #[serde(default)]
    q: String,
    /// El id de la categoría; vacío o algo que no es un id = todas.
    categoria: Option<String>,
    /// La casilla «Ver descontinuados»: sin marcar, no llega.
    descontinuados: Option<String>,
}

pub async fn page(
    State(state): State<AppState>,
    Extension(usuario): Extension<Usuario>,
    headers: HeaderMap,
    Query(query): Query<PageQuery>,
) -> Response {
    let with_descontinuados = query.descontinuados.is_some();
    let search_query = SearchQuery::new(&query.q).with_descontinuados(with_descontinuados);
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
        is_filtered: !search_query.is_empty() || categoria.is_some() || with_descontinuados,
        can_edit: usuario.can(Permiso::EditarCatalogo),
        search: ListSearch {
            q: query.q.clone(),
            categoria,
            with_descontinuados,
        },
    };
    let html = if is_htmx(&headers) {
        ResultsSection { results }.render()
    } else {
        ProductosPage {
            negocio: &state.negocio,
            usuario_nombre: Some(&usuario.nombre),
            can_edit: results.can_edit,
            q: &query.q,
            categoria,
            categorias,
            with_descontinuados,
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
