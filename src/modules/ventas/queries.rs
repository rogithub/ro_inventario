use chrono::NaiveDate;
use rust_decimal::Decimal;
use sqlx::PgPool;
use uuid::Uuid;

use super::models::{NuevaVentaPayload, ResumenDia, Venta, VentaLinea};
use crate::{db::settings::Settings, error::AppError};

pub async fn balance_monedero(pool: &PgPool, cliente_id: Uuid) -> Result<Decimal, AppError> {
    // monederogenerados llega al cliente vía ajustesproductos → ajustes.clienteid
    let row = sqlx::query!(
        r#"SELECT
            (
                SELECT COALESCE(SUM(mg.dinerodigital), 0)
                FROM monederogenerados mg
                JOIN ajustesproductos ap ON ap.id = mg.ajusteproductoid
                JOIN ajustes a ON a.id = ap.ajusteid
                WHERE a.clienteid = $1
                  AND mg.devolucionid IS NULL
                  AND mg.fechaexpiracion > NOW()
            ) AS "generado!: Decimal",
            (
                SELECT COALESCE(SUM(mr.dinerodigital), 0)
                FROM monederoredimidos mr
                JOIN monederogenerados mg ON mg.id = mr.monederogeneradosid
                JOIN ajustesproductos ap ON ap.id = mg.ajusteproductoid
                JOIN ajustes a ON a.id = ap.ajusteid
                WHERE a.clienteid = $1
            ) AS "redimido!: Decimal""#,
        cliente_id
    )
    .fetch_one(pool)
    .await?;

    let balance = (row.generado - row.redimido).max(Decimal::ZERO);
    Ok(balance)
}

#[derive(sqlx::FromRow)]
struct VentaRow {
    venta_id: Uuid,
    fecha_ajuste: chrono::NaiveDateTime,
    pago: Decimal,
    pago_monedero: Decimal,
    pago_tarjeta: Decimal,
    pago_transferencia: Decimal,
    pago_dolares: Decimal,
    tipo_cambio_dolares: Decimal,
    cambio: Decimal,
    cliente_id: Option<Uuid>,
    linea_id: Uuid,
    producto_id: Uuid,
    nid: i32,
    nombre: String,
    cantidad: Decimal,
    precio_unitario: Decimal,
    monedero_linea: Decimal,
    es_ingreso_trasladado: bool,
}

const VENTAS_DIA_SQL: &str = r#"
    SELECT
        a.id                               AS venta_id,
        a.fechaajuste                      AS fecha_ajuste,
        COALESCE(a.pago,               0)  AS pago,
        COALESCE(a.pagomonedero,       0)  AS pago_monedero,
        COALESCE(a.pagotarjeta,        0)  AS pago_tarjeta,
        COALESCE(a.pagotransferencia,  0)  AS pago_transferencia,
        COALESCE(a.pagodolares,        0)  AS pago_dolares,
        COALESCE(a.tipocambiodolares,  0)  AS tipo_cambio_dolares,
        COALESCE(a.cambio,             0)  AS cambio,
        a.clienteid                        AS cliente_id,
        ap.id                              AS linea_id,
        ap.productoid                      AS producto_id,
        p.nid,
        p.nombre,
        COALESCE(ap.cantidad,              0) AS cantidad,
        COALESCE(ap.preciounitarioventa,   0) AS precio_unitario,
        COALESCE((
            SELECT SUM(mg.dinerodigital)
            FROM monederogenerados mg
            WHERE mg.ajusteproductoid = ap.id
              AND mg.devolucionid IS NULL
              AND mg.fechaexpiracion > NOW()
        ), 0) AS monedero_linea,
        (ap.productoid IN (SELECT id FROM v_ingresos_trasladados)) AS es_ingreso_trasladado
    FROM ajustes a
    JOIN ajustesproductos ap ON ap.ajusteid = a.id
    JOIN productos p ON p.id = ap.productoid
    WHERE a.tipoajuste = 0
      AND a.fechaajuste >= $1::date
      AND a.fechaajuste <  ($1::date + INTERVAL '1 day')
    ORDER BY a.fechaajuste, a.id, ap.datestamp NULLS LAST
"#;

pub async fn ventas_del_dia(
    pool: &PgPool,
    fecha: NaiveDate,
) -> Result<(Vec<Venta>, ResumenDia), AppError> {
    let rows = sqlx::query_as::<_, VentaRow>(VENTAS_DIA_SQL)
        .bind(fecha)
        .fetch_all(pool)
        .await?;

    // Las filas llegan ordenadas por venta — agrupamos por venta_id consecutivos.
    let mut ventas: Vec<Venta> = Vec::new();
    for row in rows {
        if ventas.last().map(|v| v.id) != Some(row.venta_id) {
            ventas.push(Venta {
                id: row.venta_id,
                fecha_ajuste: row.fecha_ajuste,
                pago: row.pago,
                pago_monedero: row.pago_monedero,
                pago_tarjeta: row.pago_tarjeta,
                pago_transferencia: row.pago_transferencia,
                pago_dolares: row.pago_dolares,
                tipo_cambio_dolares: row.tipo_cambio_dolares,
                cambio: row.cambio,
                cliente_id: row.cliente_id,
                monedero_generado: Decimal::ZERO,
                lineas: Vec::new(),
            });
        }
        let venta = ventas.last_mut().unwrap();
        venta.monedero_generado += row.monedero_linea;
        venta.lineas.push(VentaLinea {
            id: row.linea_id,
            producto_id: row.producto_id,
            nid: row.nid,
            nombre: row.nombre,
            cantidad: row.cantidad,
            precio_unitario: row.precio_unitario,
            es_ingreso_trasladado: row.es_ingreso_trasladado,
        });
    }

    let resumen = ResumenDia::from_ventas(&ventas);
    Ok((ventas, resumen))
}

// Fila de v_ajuste_producto_monedero usada al redimir monedero
#[derive(sqlx::FromRow)]
struct FilaMonedero {
    monederogeneradosid: i32,
    dinerodigitaldisponible: Decimal,
}

/// Registra una venta completa en una sola transacción:
/// 1. INSERT ajustes  2. INSERT ajustesproductos  3. INSERT monederogenerados (si aplica)
/// 4. INSERT monederoredimidos (si pago_monedero > 0)
pub async fn crear_venta(
    pool: &PgPool,
    payload: &NuevaVentaPayload,
    user_id: Uuid,
    settings: &Settings,
) -> Result<Uuid, AppError> {
    if payload.lineas.is_empty() {
        return Err(AppError::Internal(anyhow::anyhow!("La venta debe tener al menos una línea")));
    }

    let generar_monedero = payload.cliente_id.is_some();

    // Cargar IDs de ingresos trasladados para excluirlos del monedero.
    // Se usa query_scalar (sin !) para no requerir entrada en el caché .sqlx/.
    let ingresos_trasladados: std::collections::HashSet<Uuid> = if generar_monedero {
        sqlx::query_scalar::<_, Uuid>("SELECT id FROM v_ingresos_trasladados")
            .fetch_all(pool)
            .await?
            .into_iter()
            .collect()
    } else {
        std::collections::HashSet::new()
    };

    let venta_id = Uuid::new_v4();
    let ahora = chrono::Local::now().naive_local();
    let fecha_expiracion = ahora + chrono::Duration::days(settings.dias_vigencia_monedero);

    let mut tx = pool.begin().await?;

    // 1. INSERT ajustes
    sqlx::query(
        "INSERT INTO ajustes \
         (id, pago, pagomonedero, pagotransferencia, pagotarjeta, pagodolares, \
          tipocambiodolares, cambio, fechaajuste, tipoajuste, ivaventa, userupdatedid, clienteid) \
         VALUES ($1,$2,$3,$4,$5,$6,$7,$8,$9,0,$10,$11,$12)",
    )
    .bind(venta_id)
    .bind(payload.pago)
    .bind(payload.pago_monedero)
    .bind(payload.pago_transferencia)
    .bind(payload.pago_tarjeta)
    .bind(payload.pago_dolares)
    .bind(payload.tipo_cambio_dolares)
    .bind(payload.cambio)
    .bind(ahora)
    .bind(settings.iva)
    .bind(user_id)
    .bind(payload.cliente_id)
    .execute(&mut *tx)
    .await?;

    // 2 + 3. INSERT ajustesproductos y monederogenerados por línea
    for linea in &payload.lineas {
        let linea_id = Uuid::new_v4();
        sqlx::query(
            "INSERT INTO ajustesproductos \
             (id, productoid, ajusteid, cantidad, notas, preciounitarioventa, datestamp, userupdatedid) \
             VALUES ($1,$2,$3,$4,$5,$6,$7,$8)",
        )
        .bind(linea_id)
        .bind(linea.producto_id)
        .bind(venta_id)
        .bind(linea.cantidad)
        .bind(linea.notas.as_deref().unwrap_or(""))
        .bind(linea.precio_unitario)
        .bind(ahora)
        .bind(user_id)
        .execute(&mut *tx)
        .await?;

        if generar_monedero && !ingresos_trasladados.contains(&linea.producto_id) {
            let dinero_digital = linea.cantidad * linea.precio_unitario * settings.tipo_cambio_monedero;
            sqlx::query(
                "INSERT INTO monederogenerados \
                 (ajusteproductoid, dinerodigital, tipocambiomonedero, userupdatedid, fechacreado, fechaexpiracion) \
                 VALUES ($1,$2,$3,$4,$5,$6)",
            )
            .bind(linea_id)
            .bind(dinero_digital)
            .bind(settings.tipo_cambio_monedero)
            .bind(user_id)
            .bind(ahora)
            .bind(fecha_expiracion)
            .execute(&mut *tx)
            .await?;
        }
    }

    // 4. INSERT monederoredimidos — consume balance del cliente, más antiguo primero
    if payload.pago_monedero > Decimal::ZERO {
        if let Some(cliente_id) = payload.cliente_id {
            // La subconsulta encuentra el primer MonederoGenerados cuyo BalanceDinero acumulado
            // cubre el pago; luego seleccionamos todas las filas hasta ese punto.
            let filas = sqlx::query_as::<_, FilaMonedero>(
                "SELECT monederogeneradosid, dinerodigitaldisponible \
                 FROM v_ajuste_producto_monedero \
                 WHERE clienteid = $1 \
                   AND monederogeneradosid <= ( \
                       SELECT monederogeneradosid \
                       FROM v_ajuste_producto_monedero \
                       WHERE clienteid = $1 \
                         AND balancedinero >= $2 \
                       ORDER BY monederogeneradosid ASC \
                       LIMIT 1 \
                   ) \
                 ORDER BY monederogeneradosid ASC",
            )
            .bind(cliente_id)
            .bind(payload.pago_monedero)
            .fetch_all(&mut *tx)
            .await?;

            let mut restante = payload.pago_monedero;
            for fila in filas {
                if restante <= Decimal::ZERO {
                    break;
                }
                let consumir = restante.min(fila.dinerodigitaldisponible);
                restante -= consumir;
                sqlx::query(
                    "INSERT INTO monederoredimidos \
                     (monederogeneradosid, dinerodigital, userupdatedid, fecharedimido, ajusteid) \
                     VALUES ($1,$2,$3,$4,$5)",
                )
                .bind(fila.monederogeneradosid)
                .bind(consumir)
                .bind(user_id)
                .bind(ahora)
                .bind(venta_id)
                .execute(&mut *tx)
                .await?;
            }
        }
    }

    tx.commit().await?;
    Ok(venta_id)
}
