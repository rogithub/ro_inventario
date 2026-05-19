use chrono::NaiveDate;
use rust_decimal::Decimal;
use sqlx::PgPool;
use uuid::Uuid;

use super::models::{ResumenDia, Venta, VentaLinea};
use crate::error::AppError;

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
        ), 0) AS monedero_linea
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
        });
    }

    let resumen = ResumenDia::from_ventas(&ventas);
    Ok((ventas, resumen))
}
