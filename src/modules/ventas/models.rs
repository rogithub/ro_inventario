use chrono::NaiveDateTime;
use rust_decimal::Decimal;
use uuid::Uuid;

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
