use rust_decimal::Decimal;
use sqlx::PgPool;
use uuid::Uuid;

use crate::error::AppError;

/// Parámetros de negocio cargados desde la tabla `settings` al arrancar.
/// Todos los valores que varían entre negocios viven aquí — no en código Rust.
#[derive(Clone, Debug)]
pub struct Settings {
    /// IVA aplicado a la comisión por tarjeta (ej: 0.16)
    pub iva: Decimal,
    /// Tasa de comisión por tarjeta de crédito (ej: 0.036)
    pub tasa_comision_tc: Decimal,
    /// UUID del producto que representa la línea de comisión TC en el carrito
    pub comision_tc_servicio_id: Uuid,
    /// Tipo de cambio USD→MXN cacheado desde Banxico
    pub tipo_cambio_dolares: Decimal,
    /// UUID del cliente genérico para ventas sin cliente registrado
    pub id_cliente_generico: Uuid,
    /// Tasa de conversión para generar cashback (ej: 0.05 = 5%)
    pub tipo_cambio_monedero: Decimal,
    /// Días de vigencia del cashback generado
    pub dias_vigencia_monedero: i64,
    /// Token para la API de Banxico (tipo de cambio USD)
    pub banxico_api_token: String,
}

/// Lee todos los parámetros de negocio de la tabla `settings` de una vez.
/// Se llama al arrancar; el resultado se guarda en `AppState`.
pub async fn load(pool: &PgPool) -> Result<Settings, AppError> {
    // Una sola query — traemos todas las filas relevantes y las indexamos en memoria.
    let rows: Vec<(String, Option<String>)> =
        sqlx::query_as(
            "SELECT key, value FROM settings WHERE key = ANY($1)",
        )
        .bind(&[
            "IVA",
            "TASA_COMISION_TARJETRA_CREDITO",
            "SERVICIO_COMISION_TARJETRA_CREDITO_ID",
            "TIPO_CAMBIO_DOLARES",
            "ID_CLIENTE_GENERICO",
            "TIPO_CAMBIO_MONEDERO",
            "DIAS_VIGENCIA_MONEDERO",
            "BANXICO_API_TOKEN",
        ] as &[&str])
        .fetch_all(pool)
        .await?;

    let get = |key: &str| -> Result<String, AppError> {
        rows.iter()
            .find(|(k, _)| k == key)
            .and_then(|(_, v)| v.clone())
            .ok_or_else(|| {
                AppError::Internal(anyhow::anyhow!("Settings: falta la clave '{key}'"))
            })
    };

    let parse_decimal = |key: &str| -> Result<Decimal, AppError> {
        get(key)?
            .parse::<Decimal>()
            .map_err(|e| AppError::Internal(anyhow::anyhow!("Settings '{key}': {e}")))
    };

    let parse_uuid = |key: &str| -> Result<Uuid, AppError> {
        get(key)?
            .parse::<Uuid>()
            .map_err(|e| AppError::Internal(anyhow::anyhow!("Settings '{key}': {e}")))
    };

    let parse_i64 = |key: &str| -> Result<i64, AppError> {
        get(key)?
            .parse::<i64>()
            .map_err(|e| AppError::Internal(anyhow::anyhow!("Settings '{key}': {e}")))
    };

    Ok(Settings {
        iva: parse_decimal("IVA")?,
        tasa_comision_tc: parse_decimal("TASA_COMISION_TARJETRA_CREDITO")?,
        comision_tc_servicio_id: parse_uuid("SERVICIO_COMISION_TARJETRA_CREDITO_ID")?,
        tipo_cambio_dolares: parse_decimal("TIPO_CAMBIO_DOLARES")?,
        id_cliente_generico: parse_uuid("ID_CLIENTE_GENERICO")?,
        tipo_cambio_monedero: parse_decimal("TIPO_CAMBIO_MONEDERO")?,
        dias_vigencia_monedero: parse_i64("DIAS_VIGENCIA_MONEDERO")?,
        banxico_api_token: get("BANXICO_API_TOKEN")?,
    })
}
