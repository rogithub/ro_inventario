use chrono::NaiveDateTime;
use rust_decimal::Decimal;
use uuid::Uuid;

#[derive(serde::Deserialize)]
pub struct LineaPayload {
    pub producto_id: Uuid,
    pub cantidad: Decimal,
    pub precio_unitario: Decimal,
    pub notas: Option<String>,
}

#[derive(serde::Deserialize)]
pub struct NuevaVentaPayload {
    pub cliente_id: Option<Uuid>,
    pub pago: Decimal,
    pub pago_monedero: Decimal,
    pub pago_tarjeta: Decimal,
    pub pago_transferencia: Decimal,
    pub pago_dolares: Decimal,
    pub tipo_cambio_dolares: Decimal,
    pub cambio: Decimal,
    pub lineas: Vec<LineaPayload>,
}

#[derive(Debug)]
pub struct VentaLinea {
    pub id: Uuid,
    pub producto_id: Uuid,
    pub nid: i32,
    pub nombre: String,
    pub cantidad: Decimal,
    pub precio_unitario: Decimal,
    pub es_ingreso_trasladado: bool,
}

impl VentaLinea {
    pub fn total(&self) -> Decimal {
        self.cantidad * self.precio_unitario
    }
}

#[derive(Debug)]
pub struct Venta {
    pub id: Uuid,
    pub fecha_ajuste: NaiveDateTime,
    pub pago: Decimal,
    pub pago_monedero: Decimal,
    pub pago_tarjeta: Decimal,
    pub pago_transferencia: Decimal,
    pub pago_dolares: Decimal,
    pub tipo_cambio_dolares: Decimal,
    pub cambio: Decimal,
    pub cliente_id: Option<Uuid>,
    pub monedero_generado: Decimal,
    pub lineas: Vec<VentaLinea>,
}

impl Venta {
    pub fn total(&self) -> Decimal {
        self.lineas.iter().map(|l| l.cantidad * l.precio_unitario).sum()
    }

    pub fn hora(&self) -> String {
        self.fecha_ajuste.format("%H:%M").to_string()
    }
}

pub struct ResumenDia {
    pub venta_productos: Decimal,
    pub ingresos_trasladados: Decimal,
    pub efectivo_en_caja: Decimal,
    pub total_monedero: Decimal,
    pub total_tarjeta: Decimal,
    pub total_transferencia: Decimal,
    pub total_dolares: Decimal,
    pub total_dolares_en_pesos: Decimal,
}

impl ResumenDia {
    pub fn from_ventas(ventas: &[Venta]) -> Self {
        let todas_las_lineas = ventas.iter().flat_map(|v| v.lineas.iter());
        let mut venta_productos = Decimal::ZERO;
        let mut ingresos_trasladados = Decimal::ZERO;
        for l in todas_las_lineas {
            let subtotal = l.cantidad * l.precio_unitario;
            if l.es_ingreso_trasladado {
                ingresos_trasladados += subtotal;
            } else {
                venta_productos += subtotal;
            }
        }
        let total_monedero: Decimal = ventas.iter().map(|v| v.pago_monedero).sum();
        let total_tarjeta: Decimal = ventas.iter().map(|v| v.pago_tarjeta).sum();
        let total_transferencia: Decimal = ventas.iter().map(|v| v.pago_transferencia).sum();
        let total_dolares: Decimal = ventas.iter().map(|v| v.pago_dolares).sum();
        let total_dolares_en_pesos: Decimal = ventas
            .iter()
            .map(|v| v.pago_dolares * v.tipo_cambio_dolares)
            .sum();
        let efectivo_en_caja = venta_productos + ingresos_trasladados
            - total_monedero
            - total_tarjeta
            - total_transferencia
            - total_dolares_en_pesos;
        Self {
            venta_productos,
            ingresos_trasladados,
            efectivo_en_caja,
            total_monedero,
            total_tarjeta,
            total_transferencia,
            total_dolares,
            total_dolares_en_pesos,
        }
    }
}

// ─── Tests ────────────────────────────────────────────────────────────────────
//
// Solo funciones puras — sin BD, sin IO. Tres niveles:
//   1. Unit tests: casos concretos y valores de frontera.
//   2. Property-based tests (proptest): propiedades matemáticas que deben
//      valer para cualquier entrada generada aleatoriamente.
//   3. Contracts (debug_assert!): en queries.rs, sobre los invariantes de entrada.

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::NaiveDateTime;
    use proptest::prelude::*;
    use rust_decimal::prelude::FromPrimitive;
    use uuid::Uuid;

    // ── Helpers de construcción ───────────────────────────────────────────────

    fn d(val: f64) -> Decimal {
        Decimal::from_f64(val).unwrap()
    }

    fn fecha() -> NaiveDateTime {
        NaiveDateTime::parse_from_str("2024-01-15 10:00:00", "%Y-%m-%d %H:%M:%S").unwrap()
    }

    fn linea(cantidad: f64, precio: f64, es_ingreso_trasladado: bool) -> VentaLinea {
        VentaLinea {
            id: Uuid::new_v4(),
            producto_id: Uuid::new_v4(),
            nid: 1,
            nombre: "Test".into(),
            cantidad: d(cantidad),
            precio_unitario: d(precio),
            es_ingreso_trasladado,
        }
    }

    fn venta_simple(lineas: Vec<VentaLinea>) -> Venta {
        Venta {
            id: Uuid::new_v4(),
            fecha_ajuste: fecha(),
            pago: Decimal::ZERO,
            pago_monedero: Decimal::ZERO,
            pago_tarjeta: Decimal::ZERO,
            pago_transferencia: Decimal::ZERO,
            pago_dolares: Decimal::ZERO,
            tipo_cambio_dolares: Decimal::ONE,
            cambio: Decimal::ZERO,
            cliente_id: None,
            monedero_generado: Decimal::ZERO,
            lineas,
        }
    }

    // ── Unit tests ────────────────────────────────────────────────────────────

    #[test]
    fn resumen_sin_ventas_es_todo_cero() {
        let r = ResumenDia::from_ventas(&[]);
        assert_eq!(r.venta_productos, Decimal::ZERO);
        assert_eq!(r.ingresos_trasladados, Decimal::ZERO);
        assert_eq!(r.efectivo_en_caja, Decimal::ZERO);
        assert_eq!(r.total_monedero, Decimal::ZERO);
        assert_eq!(r.total_tarjeta, Decimal::ZERO);
        assert_eq!(r.total_transferencia, Decimal::ZERO);
    }

    #[test]
    fn venta_total_es_suma_de_lineas() {
        let v = venta_simple(vec![
            linea(2.0, 10.0, false),
            linea(3.0, 5.0, false),
        ]);
        // 2×10 + 3×5 = 35
        assert_eq!(v.total(), d(35.0));
    }

    #[test]
    fn lineas_producto_y_trasladado_se_clasifican_por_separado() {
        let ventas = vec![venta_simple(vec![
            linea(1.0, 100.0, false),           // venta normal
            linea(1.0, 50.0, true),             // ingreso trasladado
        ])];
        let r = ResumenDia::from_ventas(&ventas);
        assert_eq!(r.venta_productos, d(100.0));
        assert_eq!(r.ingresos_trasladados, d(50.0));
    }

    #[test]
    fn efectivo_descuenta_todos_los_metodos_de_pago() {
        // Una venta de $200 pagada con $100 efectivo, $50 tarjeta, $50 transferencia
        let mut v = venta_simple(vec![linea(2.0, 100.0, false)]);
        v.pago = d(100.0);
        v.pago_tarjeta = d(50.0);
        v.pago_transferencia = d(50.0);

        let r = ResumenDia::from_ventas(&[v]);
        // efectivo_en_caja = 200 - 0 (monedero) - 50 (tarjeta) - 50 (transf) - 0 (dolares) = 100
        assert_eq!(r.efectivo_en_caja, d(100.0));
    }

    #[test]
    fn dolares_se_convierten_con_tipo_de_cambio() {
        let mut v = venta_simple(vec![linea(1.0, 200.0, false)]);
        v.pago_dolares = d(10.0);
        v.tipo_cambio_dolares = d(17.5);

        let r = ResumenDia::from_ventas(&[v]);
        assert_eq!(r.total_dolares, d(10.0));
        assert_eq!(r.total_dolares_en_pesos, d(175.0));
        // efectivo_en_caja = 200 - 175 = 25
        assert_eq!(r.efectivo_en_caja, d(25.0));
    }

    #[test]
    fn hora_formatea_correctamente() {
        let v = venta_simple(vec![]);
        assert_eq!(v.hora(), "10:00");
    }

    // ── Property-based tests ──────────────────────────────────────────────────
    //
    // proptest! genera cientos de combinaciones aleatorias e intenta violar
    // la propiedad. Si encuentra un caso que falla, lo encoge al mínimo
    // que sigue fallando (shrinking) y lo reporta.

    // Estrategia: genera un Decimal en un rango razonable para precios/pagos.
    // Se usan enteros para evitar imprecisiones de conversión f64→Decimal.
    fn monto() -> impl Strategy<Value = Decimal> {
        (0i64..=100_000i64).prop_map(|n| Decimal::new(n, 2)) // 0.00 a 1000.00
    }

    fn linea_aleatoria() -> impl Strategy<Value = VentaLinea> {
        (monto(), monto(), any::<bool>()).prop_map(|(cantidad, precio, es_ingreso)| {
            VentaLinea {
                id: Uuid::new_v4(),
                producto_id: Uuid::new_v4(),
                nid: 1,
                nombre: "Producto".into(),
                cantidad,
                precio_unitario: precio,
                es_ingreso_trasladado: es_ingreso,
            }
        })
    }

    fn venta_aleatoria() -> impl Strategy<Value = Venta> {
        (
            prop::collection::vec(linea_aleatoria(), 0..=5),
            monto(), // pago_monedero
            monto(), // pago_tarjeta
            monto(), // pago_transferencia
            monto(), // pago_dolares
            (1i64..=2500i64).prop_map(|n| Decimal::new(n, 2)), // tipo_cambio_dolares > 0
        )
            .prop_map(|(lineas, pm, pt, ptr, pd, tc)| Venta {
                id: Uuid::new_v4(),
                fecha_ajuste: fecha(),
                pago: Decimal::ZERO,
                pago_monedero: pm,
                pago_tarjeta: pt,
                pago_transferencia: ptr,
                pago_dolares: pd,
                tipo_cambio_dolares: tc,
                cambio: Decimal::ZERO,
                cliente_id: None,
                monedero_generado: Decimal::ZERO,
                lineas,
            })
    }

    proptest! {
        // Propiedad 1: la fórmula del efectivo siempre se mantiene.
        // efectivo = (venta_productos + ingresos_trasladados) - monedero - tarjeta - transf - dolares_en_pesos
        // Esta propiedad verifica que from_ventas() no tiene bugs aritméticos
        // independientemente del número de ventas o distribución de pagos.
        #[test]
        fn efectivo_es_ventas_menos_pagos_no_efectivo(
            ventas in prop::collection::vec(venta_aleatoria(), 0..=10)
        ) {
            let r = ResumenDia::from_ventas(&ventas);
            let esperado = r.venta_productos + r.ingresos_trasladados
                - r.total_monedero
                - r.total_tarjeta
                - r.total_transferencia
                - r.total_dolares_en_pesos;
            prop_assert_eq!(r.efectivo_en_caja, esperado);
        }

        // Propiedad 2: dolares_en_pesos es siempre la suma de (dolares × tipo_cambio) por venta.
        // Verifica que la conversión de moneda no acumula errores entre ventas.
        #[test]
        fn dolares_en_pesos_es_suma_de_conversiones_individuales(
            ventas in prop::collection::vec(venta_aleatoria(), 0..=10)
        ) {
            let r = ResumenDia::from_ventas(&ventas);
            let esperado: Decimal = ventas.iter()
                .map(|v| v.pago_dolares * v.tipo_cambio_dolares)
                .sum();
            prop_assert_eq!(r.total_dolares_en_pesos, esperado);
        }

        // Propiedad 3: venta_productos + ingresos_trasladados == suma de todos los subtotales de líneas.
        // Verifica que la clasificación no pierde ni duplica líneas.
        #[test]
        fn clasificacion_lineas_es_particion_completa(
            ventas in prop::collection::vec(venta_aleatoria(), 0..=10)
        ) {
            let r = ResumenDia::from_ventas(&ventas);
            let total_lineas: Decimal = ventas.iter()
                .flat_map(|v| v.lineas.iter())
                .map(|l| l.cantidad * l.precio_unitario)
                .sum();
            prop_assert_eq!(r.venta_productos + r.ingresos_trasladados, total_lineas);
        }
    }
}
