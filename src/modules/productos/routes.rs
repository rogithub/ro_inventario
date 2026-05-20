use axum::{Json, extract::{Query, State}};

use super::queries;
use super::models::ProductoBuscado;
use crate::{AppState, error::AppError};

#[derive(serde::Deserialize)]
pub struct BuscarParams {
    q: Option<String>,
    stock: Option<bool>,
}

pub async fn buscar(
    State(state): State<AppState>,
    Query(params): Query<BuscarParams>,
) -> Result<Json<Vec<ProductoBuscado>>, AppError> {
    let q = params.q.unwrap_or_default();
    if q.trim().is_empty() {
        return Ok(Json(vec![]));
    }
    let solo_con_stock = params.stock.unwrap_or(false);
    let resultados = queries::buscar(&state.pool, q.trim(), solo_con_stock).await?;
    Ok(Json(resultados))
}
