use std::str::FromStr;

use kernel::Decimal;

use super::*;

const CATEGORIA_V1: Uuid = Uuid::from_u128(1);
const CATEGORIA_FUSIONADA_V1: Uuid = Uuid::from_u128(2);
const CATEGORIA_V2: CategoriaId = CategoriaId(Uuid::from_u128(1));
const PIEZA: UnidadMedidaId = UnidadMedidaId(Uuid::from_u128(101));
const METRO: UnidadMedidaId = UnidadMedidaId(Uuid::from_u128(102));
const GRAMO: UnidadMedidaId = UnidadMedidaId(Uuid::from_u128(103));

fn fecha(text: &str) -> NaiveDateTime {
    NaiveDateTime::parse_from_str(text, "%Y-%m-%d %H:%M").unwrap()
}

fn utc(text: &str) -> DateTime<Utc> {
    fecha(text).and_utc()
}

/// Un lápiz por pieza, en una categoría, editado por última vez el 20 de junio de 2026.
fn producto_v1(nid: i32) -> ProductoV1 {
    ProductoV1 {
        id: Uuid::from_u128(1000 + nid as u128),
        nid,
        nombre: "LAPIZ".to_string(),
        unidad: Some("PIEZA".to_string()),
        categoria_ids: vec![CATEGORIA_V1],
        marca: None,
        modelo: None,
        color: None,
        descripcion: None,
        datestamp: fecha("2026-06-20 10:00"),
    }
}

fn precio(producto: &ProductoV1, precio_venta: &str, fecha_v1: &str) -> PrecioV1 {
    PrecioV1 {
        producto_id: producto.id,
        precio_venta: Decimal::from_str(precio_venta).unwrap(),
        fecha: fecha(fecha_v1),
    }
}

fn migrate(
    productos_v1: &[ProductoV1],
    precios: &[PrecioV1],
) -> Result<Vec<ProductoMigrado>, Vec<String>> {
    let categorias = HashMap::from([
        (CATEGORIA_V1, CATEGORIA_V2),
        (CATEGORIA_FUSIONADA_V1, CATEGORIA_V2),
    ]);
    let unidades =
        [("Pieza", PIEZA), ("Metro", METRO), ("Gramo", GRAMO)].map(|(nombre, id)| UnidadMedida {
            id,
            nombre: nombre.to_string(),
            allows_fraction: nombre != "Pieza",
        });
    productos(productos_v1, precios, &categorias, &unidades)
}

/// Migra un solo producto y regresa el resultado o su único error.
fn migrate_one(producto: ProductoV1, precios: &[PrecioV1]) -> Result<ProductoMigrado, String> {
    migrate(&[producto], precios)
        .map(|mut migrados| migrados.remove(0))
        .map_err(|mut errors| errors.remove(0))
}

fn precio_actual(migrado: &ProductoMigrado) -> String {
    migrado
        .producto
        .precio_venta()
        .map(|p| p.value().to_string())
        .unwrap_or_default()
}

#[test]
fn conserva_el_uuid_y_el_nid_de_la_v1() {
    let lapiz = producto_v1(5);
    let precios = [precio(&lapiz, "8", "2024-03-01 09:00")];

    let migrado = migrate_one(lapiz.clone(), &precios).unwrap();

    assert_eq!(migrado.id, ArticuloId(lapiz.id));
    assert_eq!(migrado.nid, Nid(5));
}

#[test]
fn el_precio_actual_es_el_ultimo_del_historial() {
    let lapiz = producto_v1(1);
    let precios = [
        precio(&lapiz, "12.50", "2025-01-10 09:00"),
        precio(&lapiz, "8", "2022-06-30 10:38"),
        precio(&lapiz, "10", "2024-03-01 09:00"),
    ];

    let migrado = migrate_one(lapiz, &precios).unwrap();

    assert_eq!(precio_actual(&migrado), "12.50");
}

#[test]
fn el_historial_pasa_completo_y_en_orden_aun_si_repite_el_precio() {
    let lapiz = producto_v1(1);
    let precios = [
        precio(&lapiz, "10", "2024-03-01 09:00"),
        precio(&lapiz, "8", "2022-06-30 10:38"),
        precio(&lapiz, "10", "2025-01-10 09:00"),
    ];

    let migrado = migrate_one(lapiz, &precios).unwrap();

    let historial: Vec<String> = migrado
        .historial
        .iter()
        .map(|p| p.precio_venta.value().to_string())
        .collect();
    assert_eq!(historial, ["8.00", "10.00", "10.00"]);
}

#[test]
fn la_hora_de_la_v1_es_de_cancun_y_se_guarda_en_utc() {
    let lapiz = producto_v1(1);
    let precios = [precio(&lapiz, "8", "2024-03-01 09:00")];

    let migrado = migrate_one(lapiz, &precios).unwrap();

    assert_eq!(migrado.historial[0].desde, utc("2024-03-01 14:00"));
}

#[test]
fn se_dio_de_alta_cuando_tuvo_su_primer_precio() {
    let lapiz = producto_v1(1);
    let precios = [
        precio(&lapiz, "10", "2024-03-01 09:00"),
        precio(&lapiz, "8", "2022-06-30 10:38"),
    ];

    let migrado = migrate_one(lapiz, &precios).unwrap();

    assert_eq!(migrado.created_at, utc("2022-06-30 15:38"));
}

#[test]
fn la_ultima_edicion_es_el_datestamp_si_es_posterior_al_ultimo_precio() {
    let lapiz = producto_v1(1);
    let precios = [precio(&lapiz, "8", "2024-03-01 09:00")];

    let migrado = migrate_one(lapiz, &precios).unwrap();

    assert_eq!(migrado.updated_at, utc("2026-06-20 15:00"));
}

#[test]
fn la_ultima_edicion_es_el_ultimo_precio_si_cambio_despues_del_datestamp() {
    let lapiz = producto_v1(1649);
    let precios = [precio(&lapiz, "8", "2026-09-18 17:57")];

    let migrado = migrate_one(lapiz, &precios).unwrap();

    assert_eq!(migrado.updated_at, utc("2026-09-18 22:57"));
}

#[test]
fn el_nombre_junta_los_espacios_de_mas_y_conserva_las_mayusculas() {
    let mut cinta = producto_v1(1);
    cinta.nombre = " FLORATAPE  VERDE LIMON ".to_string();
    let precios = [precio(&cinta, "8", "2024-03-01 09:00")];

    let migrado = migrate_one(cinta, &precios).unwrap();

    assert_eq!(migrado.producto.nombre(), "FLORATAPE VERDE LIMON");
}

#[test]
fn pasa_marca_modelo_color_y_descripcion() {
    let mut lapiz = producto_v1(1);
    lapiz.marca = Some("MIRADO".to_string());
    lapiz.modelo = Some("2HB".to_string());
    lapiz.color = Some("AMARILLO".to_string());
    lapiz.descripcion = Some("Lápiz de grafito".to_string());
    let precios = [precio(&lapiz, "8", "2024-03-01 09:00")];

    let migrado = migrate_one(lapiz, &precios).unwrap();

    let p = &migrado.producto;
    assert_eq!(
        (p.marca(), p.modelo(), p.color(), p.descripcion()),
        (
            Some("MIRADO"),
            Some("2HB"),
            Some("AMARILLO"),
            Some("Lápiz de grafito")
        )
    );
}

#[test]
fn las_unidades_de_la_v1_pasan_a_pieza_metro_y_gramo() {
    let v1: Vec<ProductoV1> = ["PIEZA", "METRO", "GRAMOS"]
        .iter()
        .zip(1..)
        .map(|(unidad, nid)| ProductoV1 {
            unidad: Some(unidad.to_string()),
            ..producto_v1(nid)
        })
        .collect();
    let precios: Vec<PrecioV1> = v1
        .iter()
        .map(|p| precio(p, "8", "2024-03-01 09:00"))
        .collect();

    let migrados = migrate(&v1, &precios).unwrap();

    let unidades: Vec<UnidadMedidaId> = migrados
        .iter()
        .map(|m| m.producto.unidad_medida_id())
        .collect();
    assert_eq!(unidades, [PIEZA, METRO, GRAMO]);
}

#[test]
fn la_categoria_fusionada_lleva_a_la_que_se_conservo() {
    let mut cinta = producto_v1(1);
    cinta.categoria_ids = vec![CATEGORIA_FUSIONADA_V1];
    let precios = [precio(&cinta, "8", "2024-03-01 09:00")];

    let migrado = migrate_one(cinta, &precios).unwrap();

    assert_eq!(migrado.producto.categoria_id(), CATEGORIA_V2);
}

#[test]
fn una_unidad_que_no_es_pieza_metro_ni_gramos_detiene_la_migracion() {
    let mut hoja = producto_v1(40);
    hoja.unidad = Some("HOJA".to_string());
    let precios = [precio(&hoja, "1", "2024-03-01 09:00")];

    let error = migrate_one(hoja, &precios).unwrap_err();

    assert_eq!(
        error,
        "Producto NID 40 «LAPIZ» de la v1: la unidad «HOJA» no es PIEZA, METRO ni GRAMOS."
    );
}

#[test]
fn un_producto_con_dos_categorias_distintas_detiene_la_migracion() {
    let mut lapiz = producto_v1(1);
    lapiz.categoria_ids = vec![CATEGORIA_V1, Uuid::from_u128(3)];
    let precios = [precio(&lapiz, "8", "2024-03-01 09:00")];

    let error = migrate_one(lapiz, &precios).unwrap_err();

    assert!(error.ends_with("tiene dos categorías distintas; en la v2 solo puede tener una."));
}

#[test]
fn un_producto_sin_categoria_detiene_la_migracion() {
    let mut lapiz = producto_v1(1);
    lapiz.categoria_ids = vec![];
    let precios = [precio(&lapiz, "8", "2024-03-01 09:00")];

    let error = migrate_one(lapiz, &precios).unwrap_err();

    assert!(error.ends_with("no tiene categoría."));
}

#[test]
fn un_producto_sin_historial_detiene_la_migracion() {
    let error = migrate_one(producto_v1(1), &[]).unwrap_err();

    assert!(error.ends_with("no tiene precio en su historial."));
}

#[test]
fn dos_precios_a_la_misma_hora_detienen_la_migracion() {
    let lapiz = producto_v1(1);
    let precios = [
        precio(&lapiz, "8", "2024-03-01 09:00"),
        precio(&lapiz, "10", "2024-03-01 09:00"),
    ];

    let error = migrate_one(lapiz, &precios).unwrap_err();

    assert!(error.ends_with("tiene dos precios a la misma hora y no se sabe cuál es el actual."));
}

#[test]
fn junta_los_errores_de_todos_los_productos_antes_de_detenerse() {
    let sin_categoria = ProductoV1 {
        categoria_ids: vec![],
        ..producto_v1(1)
    };
    let bien = producto_v1(2);
    let precio_con_milesimas = producto_v1(3);
    let precios = [
        precio(&sin_categoria, "8", "2024-03-01 09:00"),
        precio(&bien, "8", "2024-03-01 09:00"),
        precio(&precio_con_milesimas, "8.125", "2024-03-01 09:00"),
    ];

    let errors = migrate(&[sin_categoria, bien, precio_con_milesimas], &precios).unwrap_err();

    assert_eq!(errors.len(), 2, "{errors:?}");
}
