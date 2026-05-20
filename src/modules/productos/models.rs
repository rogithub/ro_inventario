use rust_decimal::Decimal;
use uuid::Uuid;

#[derive(serde::Serialize, sqlx::FromRow)]
pub struct ProductoBuscado {
    pub id: Uuid,
    pub nid: i32,
    pub nombre: String,
    pub esservicio: bool,
    pub unidadmedida: String,
    #[sqlx(rename = "ultimoprecioventa")]
    pub precio_venta: Decimal,
    #[sqlx(rename = "preciocomprapromedio")]
    pub precio_compra_promedio: Decimal,
    pub stock: Decimal,
    pub codigobarrasitem: String,
    pub codigobarrascaja: String,
}
