use std::collections::HashMap;
use std::str::FromStr;

use inventario::articulos::{ArticuloId, Nid, PrecioVenta};
use inventario::categorias::{CategoriaId, NewCategoria};
use inventario::productos::{NewProducto, ProductoFields};
use inventario::unidades_medida::UnidadMedidaId;
use kernel::{DateTime, Decimal, Utc, Uuid};

use super::*;
use crate::transform::productos::MigratedPrecio;

const CINTA: CategoriaId = CategoriaId(Uuid::from_u128(1));

fn two_categorias() -> Categorias {
    Categorias {
        to_insert: vec![
            (
                CategoriaId(Uuid::from_u128(1)),
                NewCategoria::new("CINTA").unwrap(),
            ),
            (
                CategoriaId(Uuid::from_u128(2)),
                NewCategoria::new("GOMA").unwrap(),
            ),
        ],
        by_v1_id: HashMap::new(),
    }
}

/// Lo que la v1 de las pruebas dice que debe quedar: las dos categorías y los productos dados.
fn expected(productos: &[MigratedProducto]) -> RecordCounts {
    let precios = productos.iter().flat_map(|p| &p.historial);
    RecordCounts {
        categorias: 2,
        productos: productos.len() as i64,
        precios: precios.clone().count() as i64,
        precios_sum: precios.map(|p| p.precio_venta.value()).sum(),
    }
}

fn utc(text: &str) -> DateTime<Utc> {
    DateTime::from_str(text).unwrap()
}

fn precio(precio_venta: &str, desde: &str) -> MigratedPrecio {
    MigratedPrecio {
        precio_venta: PrecioVenta::parse(precio_venta).unwrap(),
        desde: utc(desde),
    }
}

async fn pieza(pool: &PgPool) -> UnidadMedidaId {
    UnidadMedidaId(
        sqlx::query_scalar("SELECT id FROM unidades_medida WHERE nombre = 'Pieza'")
            .fetch_one(pool)
            .await
            .unwrap(),
    )
}

/// Un producto de la categoría CINTA que pasó las reglas, con su historial del más viejo al más
/// nuevo; el precio actual es el último.
fn migrated(
    nid: i32,
    nombre: &str,
    unidad: UnidadMedidaId,
    historial: Vec<MigratedPrecio>,
) -> MigratedProducto {
    let current = historial.last().unwrap().precio_venta.value().to_string();
    MigratedProducto {
        id: ArticuloId(Uuid::from_u128(1000 + nid as u128)),
        nid: Nid(nid),
        producto: NewProducto::new(ProductoFields {
            nombre,
            categoria_id: Some(CINTA),
            unidad_medida_id: unidad,
            precio_venta: &current,
            descripcion: "",
            marca: "",
            modelo: "",
            color: "",
        })
        .unwrap(),
        created_at: historial[0].desde,
        updated_at: utc("2026-06-20T15:00:00Z"),
        historial,
    }
}

async fn two_productos(pool: &PgPool) -> Vec<MigratedProducto> {
    let pieza = pieza(pool).await;
    vec![
        migrated(
            5,
            "CINTA CANELA",
            pieza,
            vec![
                precio("10", "2022-06-30T15:38:00Z"),
                precio("12.50", "2024-03-01T14:00:00Z"),
            ],
        ),
        migrated(
            7,
            "CINTA MAGICA",
            pieza,
            vec![precio("3.25", "2023-01-01T17:00:00Z")],
        ),
    ]
}

/// id, nid, nombre, categoria_id, precio_venta, created_at, updated_at y updated_by.
type ArticuloRow = (
    Uuid,
    i32,
    String,
    Uuid,
    Decimal,
    DateTime<Utc>,
    DateTime<Utc>,
    Option<Uuid>,
);

async fn count_articulos(pool: &PgPool) -> i64 {
    sqlx::query_scalar("SELECT count(*) FROM articulos")
        .fetch_one(pool)
        .await
        .unwrap()
}

async fn get_categorias(pool: &PgPool) -> Vec<(Uuid, String)> {
    sqlx::query_as("SELECT id, nombre FROM categorias ORDER BY nombre")
        .fetch_all(pool)
        .await
        .unwrap()
}

#[sqlx::test(migrator = "db::MIGRATOR")]
async fn escribe_las_categorias_con_el_uuid_de_la_v1(pool: PgPool) {
    let written = write(&pool, &two_categorias(), &[], expected(&[]), 1)
        .await
        .unwrap();

    assert_eq!(written, expected(&[]));
    assert_eq!(
        get_categorias(&pool).await,
        [
            (Uuid::from_u128(1), "CINTA".to_string()),
            (Uuid::from_u128(2), "GOMA".to_string()),
        ]
    );
}

#[sqlx::test(migrator = "db::MIGRATOR")]
async fn si_los_conteos_no_cuadran_no_deja_nada(pool: PgPool) {
    let mut counts = expected(&[]);
    counts.categorias = 3;
    let error = write(&pool, &two_categorias(), &[], counts, 1)
        .await
        .unwrap_err();

    assert!(
        matches!(error, WriteError::CountMismatch { .. }),
        "{error:?}"
    );
    assert_eq!(get_categorias(&pool).await, []);
}

#[sqlx::test(migrator = "db::MIGRATOR")]
async fn se_niega_si_la_base_ya_tiene_categorias_y_no_la_toca(pool: PgPool) {
    sqlx::query("INSERT INTO categorias (nombre) VALUES ('Plumas')")
        .execute(&pool)
        .await
        .unwrap();

    let error = write(&pool, &two_categorias(), &[], expected(&[]), 1)
        .await
        .unwrap_err();

    assert!(
        matches!(
            error,
            WriteError::NotEmpty {
                categorias: 1,
                articulos: 0
            }
        ),
        "{error:?}"
    );
    let nombres: Vec<String> = get_categorias(&pool)
        .await
        .into_iter()
        .map(|(_, nombre)| nombre)
        .collect();
    assert_eq!(nombres, ["Plumas"]);
}

#[sqlx::test(migrator = "db::MIGRATOR")]
async fn escribe_los_productos_con_su_uuid_nid_fechas_y_precio_actual(pool: PgPool) {
    let productos = two_productos(&pool).await;

    write(
        &pool,
        &two_categorias(),
        &productos,
        expected(&productos),
        7,
    )
    .await
    .unwrap();

    let rows: Vec<ArticuloRow> = sqlx::query_as(
        "SELECT a.id, a.nid, a.nombre, a.categoria_id, a.precio_venta, a.created_at,
                    a.updated_at, a.updated_by
             FROM articulos a JOIN productos p ON p.articulo_id = a.id
             WHERE a.kind = 'producto' ORDER BY a.nid",
    )
    .fetch_all(&pool)
    .await
    .unwrap();
    assert_eq!(
        rows,
        [
            (
                Uuid::from_u128(1005),
                5,
                "CINTA CANELA".to_string(),
                CINTA.0,
                Decimal::from_str("12.50").unwrap(),
                utc("2022-06-30T15:38:00Z"),
                utc("2026-06-20T15:00:00Z"),
                None,
            ),
            (
                Uuid::from_u128(1007),
                7,
                "CINTA MAGICA".to_string(),
                CINTA.0,
                Decimal::from_str("3.25").unwrap(),
                utc("2023-01-01T17:00:00Z"),
                utc("2026-06-20T15:00:00Z"),
                None,
            ),
        ]
    );
}

#[sqlx::test(migrator = "db::MIGRATOR")]
async fn escribe_el_historial_completo_con_sus_fechas(pool: PgPool) {
    let productos = two_productos(&pool).await;

    write(
        &pool,
        &two_categorias(),
        &productos,
        expected(&productos),
        7,
    )
    .await
    .unwrap();

    let historial: Vec<(i32, String, DateTime<Utc>, Option<Uuid>)> = sqlx::query_as(
        "SELECT a.nid, h.precio_venta::text, h.desde, h.updated_by
         FROM precios_historial h JOIN articulos a ON a.id = h.articulo_id
         ORDER BY a.nid, h.desde",
    )
    .fetch_all(&pool)
    .await
    .unwrap();
    assert_eq!(
        historial,
        [
            (5, "10.00".to_string(), utc("2022-06-30T15:38:00Z"), None),
            (5, "12.50".to_string(), utc("2024-03-01T14:00:00Z"), None),
            (7, "3.25".to_string(), utc("2023-01-01T17:00:00Z"), None),
        ]
    );
}

#[sqlx::test(migrator = "db::MIGRATOR")]
async fn la_primera_alta_despues_de_migrar_recibe_el_nid_siguiente_al_mas_alto_de_la_v1(
    pool: PgPool,
) {
    let productos = two_productos(&pool).await;
    // El más alto de la v1 es un servicio, el 9, que todavía no se migra.
    write(
        &pool,
        &two_categorias(),
        &productos,
        expected(&productos),
        9,
    )
    .await
    .unwrap();

    let nid: i32 = sqlx::query_scalar(
        "INSERT INTO articulos (kind, nombre, categoria_id, unidad_medida_id)
         VALUES ('producto', 'CINTA NUEVA', $1, $2) RETURNING nid",
    )
    .bind(CINTA.0)
    .bind(pieza(&pool).await.0)
    .fetch_one(&pool)
    .await
    .unwrap();

    assert_eq!(nid, 10);
}

#[sqlx::test(migrator = "db::MIGRATOR")]
async fn si_los_productos_no_cuadran_no_deja_nada(pool: PgPool) {
    let productos = two_productos(&pool).await;
    let mut counts = expected(&productos);
    counts.productos = 3;

    let error = write(&pool, &two_categorias(), &productos, counts, 7)
        .await
        .unwrap_err();

    assert!(
        matches!(error, WriteError::CountMismatch { .. }),
        "{error:?}"
    );
    assert_eq!(count_articulos(&pool).await, 0);
    assert_eq!(get_categorias(&pool).await, []);
}

#[sqlx::test(migrator = "db::MIGRATOR")]
async fn si_la_suma_del_historial_no_cuadra_no_deja_nada(pool: PgPool) {
    let productos = two_productos(&pool).await;
    let mut counts = expected(&productos);
    counts.precios_sum += Decimal::new(1, 2);

    let error = write(&pool, &two_categorias(), &productos, counts, 7)
        .await
        .unwrap_err();

    assert!(
        matches!(error, WriteError::CountMismatch { .. }),
        "{error:?}"
    );
    assert_eq!(count_articulos(&pool).await, 0);
}

#[sqlx::test(migrator = "db::MIGRATOR")]
async fn si_un_nid_migrado_pasa_del_mas_alto_de_la_v1_no_deja_nada(pool: PgPool) {
    let productos = two_productos(&pool).await;

    let error = write(
        &pool,
        &two_categorias(),
        &productos,
        expected(&productos),
        6,
    )
    .await
    .unwrap_err();

    assert!(
        matches!(
            error,
            WriteError::NidAboveV1 {
                max_nid_v1: 6,
                max_nid_written: 7
            }
        ),
        "{error:?}"
    );
    assert_eq!(count_articulos(&pool).await, 0);
}

#[sqlx::test(migrator = "db::MIGRATOR")]
async fn si_la_v1_no_tiene_articulos_la_primera_alta_recibe_el_nid_1(pool: PgPool) {
    write(&pool, &two_categorias(), &[], expected(&[]), 0)
        .await
        .unwrap();

    let nid: i32 = sqlx::query_scalar(
        "INSERT INTO articulos (kind, nombre, categoria_id, unidad_medida_id)
         VALUES ('producto', 'CINTA NUEVA', $1, $2) RETURNING nid",
    )
    .bind(CINTA.0)
    .bind(pieza(&pool).await.0)
    .fetch_one(&pool)
    .await
    .unwrap();

    assert_eq!(nid, 1);
}

#[sqlx::test(migrator = "db::MIGRATOR")]
async fn escribe_la_unidad_la_descripcion_la_marca_el_modelo_y_el_color(pool: PgPool) {
    let metro: Uuid = sqlx::query_scalar("SELECT id FROM unidades_medida WHERE nombre = 'Metro'")
        .fetch_one(&pool)
        .await
        .unwrap();
    let mut liston = migrated(
        3,
        "LISTON",
        UnidadMedidaId(metro),
        vec![precio("4", "2024-03-01T14:00:00Z")],
    );
    liston.producto = NewProducto::new(ProductoFields {
        nombre: "LISTON",
        categoria_id: Some(CINTA),
        unidad_medida_id: UnidadMedidaId(metro),
        precio_venta: "4",
        descripcion: "DE SATIN",
        marca: "OFFIX",
        modelo: "N3",
        color: "ROJO",
    })
    .unwrap();
    let productos = [liston];

    write(
        &pool,
        &two_categorias(),
        &productos,
        expected(&productos),
        3,
    )
    .await
    .unwrap();

    let row: (Uuid, String, String, String, String) = sqlx::query_as(
        "SELECT a.unidad_medida_id, a.descripcion, p.marca, p.modelo, p.color
         FROM articulos a JOIN productos p ON p.articulo_id = a.id",
    )
    .fetch_one(&pool)
    .await
    .unwrap();
    assert_eq!(
        row,
        (
            metro,
            "DE SATIN".to_string(),
            "OFFIX".to_string(),
            "N3".to_string(),
            "ROJO".to_string(),
        )
    );
}
