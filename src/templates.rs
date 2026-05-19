use askama::Template;
use axum::{
    http::StatusCode,
    response::{Html, IntoResponse, Response},
};

// Convierte cualquier template Askama en un Response de Axum.
// Al separarlo del derive, no dependemos de askama_axum ni de versiones específicas
// de la integración — basta con que el template implemente `Template`.
pub fn render<T: Template>(t: T) -> Response {
    match t.render() {
        Ok(html) => Html(html).into_response(),
        Err(e) => {
            tracing::error!("Error al renderizar template: {e}");
            StatusCode::INTERNAL_SERVER_ERROR.into_response()
        }
    }
}
