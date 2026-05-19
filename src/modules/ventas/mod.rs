use askama::Template;
use axum::response::Response;

use crate::templates;

#[derive(Template)]
#[template(path = "ventas/index.html")]
struct VentasTemplate;

pub async fn index() -> Response {
    templates::render(VentasTemplate)
}
