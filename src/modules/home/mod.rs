use askama::Template;
use axum::response::Response;

use crate::templates;

#[derive(Template)]
#[template(path = "index.html")]
struct HomeTemplate;

pub async fn index() -> Response {
    templates::render(HomeTemplate)
}
