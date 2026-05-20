use askama::Template;
use axum::{
    Json,
    extract::{Path, Query, State},
    response::Response,
};
use chrono::NaiveDate;
use rust_decimal::Decimal;
use uuid::Uuid;

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

/// GET /api/tipo-cambio
/// Llama a Banxico; si falla usa el valor cacheado en Settings.
pub async fn tipo_cambio(
    State(state): State<AppState>,
) -> Result<Json<serde_json::Value>, AppError> {
    #[derive(serde::Deserialize)]
    struct BanxicoResp {
        bmx: Bmx,
    }
    #[derive(serde::Deserialize)]
    struct Bmx {
        series: Vec<Series>,
    }
    #[derive(serde::Deserialize)]
    struct Series {
        datos: Vec<Dato>,
    }
    #[derive(serde::Deserialize)]
    struct Dato {
        dato: String,
    }

    let url = "https://www.banxico.org.mx/SieAPIRest/service/v1/series/SF43718/datos/oportuno";
    let resultado = reqwest::Client::new()
        .get(url)
        .header("Bmx-Token", &state.settings.banxico_api_token)
        .send()
        .await
        .ok()
        .and_then(|r| r.error_for_status().ok());

    if let Some(resp) = resultado {
        if let Ok(body) = resp.json::<BanxicoResp>().await {
            if let Some(dato) = body.bmx.series.first()
                .and_then(|s| s.datos.first())
                .map(|d| d.dato.trim().to_string())
            {
                if let Ok(tc) = dato.parse::<Decimal>() {
                    return Ok(Json(serde_json::json!({
                        "tipo_cambio": tc,
                        "fuente": "banxico"
                    })));
                }
            }
        }
    }

    // Fallback al valor cacheado en Settings
    Ok(Json(serde_json::json!({
        "tipo_cambio": state.settings.tipo_cambio_dolares,
        "fuente": "cache"
    })))
}

/// GET /api/monedero/:cliente_id
pub async fn monedero(
    State(state): State<AppState>,
    Path(cliente_id): Path<Uuid>,
) -> Result<Json<serde_json::Value>, AppError> {
    let balance = queries::balance_monedero(&state.pool, cliente_id).await?;
    Ok(Json(serde_json::json!({ "balance": balance })))
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
