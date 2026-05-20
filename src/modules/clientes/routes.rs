use axum::{Json, extract::{Query, State}};

use super::models::ClienteBuscado;
use super::queries;
use crate::{AppState, error::AppError};

#[derive(serde::Deserialize)]
pub struct BuscarParams {
    q: Option<String>,
}

pub async fn buscar(
    State(state): State<AppState>,
    Query(params): Query<BuscarParams>,
) -> Result<Json<Vec<ClienteBuscado>>, AppError> {
    let q = params.q.unwrap_or_default();
    if q.trim().is_empty() {
        return Ok(Json(vec![]));
    }
    let resultados = queries::buscar(&state.pool, q.trim()).await?;
    Ok(Json(resultados))
}
