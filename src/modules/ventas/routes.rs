use askama::Template;
use axum::{
    extract::{Query, State},
    response::Response,
};
use chrono::NaiveDate;

use super::models::{ResumenDia, Venta};
use super::queries;
use crate::{error::AppError, filters, templates, AppState};

#[derive(serde::Deserialize)]
pub struct VentasQuery {
    fecha: Option<NaiveDate>,
}

#[derive(Template)]
#[template(path = "ventas/index.html")]
struct VentasIndexTemplate {
    ventas: Vec<Venta>,
    fecha_input: String, // valor para <input type="date"> — formato YYYY-MM-DD
    resumen: ResumenDia,
}

pub async fn index(
    State(state): State<AppState>,
    Query(params): Query<VentasQuery>,
) -> Result<Response, AppError> {
    let fecha = params.fecha.unwrap_or_else(|| chrono::Local::now().date_naive());
    let (ventas, resumen) = queries::ventas_del_dia(&state.pool, fecha).await?;
    Ok(templates::render(VentasIndexTemplate {
        ventas,
        fecha_input: fecha.format("%Y-%m-%d").to_string(),
        resumen,
    }))
}
