use std::str::FromStr;

use kernel::Decimal;

use super::*;

const CATEGORIA_V1: Uuid = Uuid::from_u128(1);
const MERGED_CATEGORIA_V1: Uuid = Uuid::from_u128(2);
const CATEGORIA_V2: CategoriaId = CategoriaId(Uuid::from_u128(1));
const PIEZA: UnidadMedidaId = UnidadMedidaId(Uuid::from_u128(101));
const METRO: UnidadMedidaId = UnidadMedidaId(Uuid::from_u128(102));
const GRAMO: UnidadMedidaId = UnidadMedidaId(Uuid::from_u128(103));

fn datetime(text: &str) -> NaiveDateTime {
    NaiveDateTime::parse_from_str(text, "%Y-%m-%d %H:%M").unwrap()
}

fn utc(text: &str) -> DateTime<Utc> {
    datetime(text).and_utc()
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
        datestamp: Some(datetime("2026-06-20 10:00")),
    }
}

fn precio(producto: &ProductoV1, precio_venta: &str, datetime_v1: &str) -> PrecioV1 {
    PrecioV1 {
        producto_id: producto.id,
        precio_venta: Some(Decimal::from_str(precio_venta).unwrap()),
        created_at: datetime(datetime_v1),
    }
}

fn migrate(
    productos_v1: &[ProductoV1],
    precios: &[PrecioV1],
) -> Result<Vec<MigratedProducto>, Vec<String>> {
    let categorias = HashMap::from([
        (CATEGORIA_V1, CATEGORIA_V2),
        (MERGED_CATEGORIA_V1, CATEGORIA_V2),
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
fn migrate_one(producto: ProductoV1, precios: &[PrecioV1]) -> Result<MigratedProducto, String> {
    migrate(&[producto], precios)
        .map(|mut all_migrated| all_migrated.remove(0))
        .map_err(|mut errors| errors.remove(0))
}

fn current_precio(migrated: &MigratedProducto) -> String {
    migrated
        .producto
        .precio_venta()
        .map(|p| p.value().to_string())
        .unwrap_or_default()
}

#[test]
fn conserva_el_uuid_y_el_nid_de_la_v1() {
    let lapiz = producto_v1(5);
    let precios = [precio(&lapiz, "8", "2024-03-01 09:00")];

    let migrated = migrate_one(lapiz.clone(), &precios).unwrap();

    assert_eq!(migrated.id, ArticuloId(lapiz.id));
    assert_eq!(migrated.nid, Nid(5));
}

#[test]
fn el_current_precio_es_el_ultimo_del_historial() {
    let lapiz = producto_v1(1);
    let precios = [
        precio(&lapiz, "12.50", "2025-01-10 09:00"),
        precio(&lapiz, "8", "2022-06-30 10:38"),
        precio(&lapiz, "10", "2024-03-01 09:00"),
    ];

    let migrated = migrate_one(lapiz, &precios).unwrap();

    assert_eq!(current_precio(&migrated), "12.50");
}

#[test]
fn el_historial_pasa_completo_y_en_orden_aun_si_repite_el_precio() {
    let lapiz = producto_v1(1);
    let precios = [
        precio(&lapiz, "10", "2024-03-01 09:00"),
        precio(&lapiz, "8", "2022-06-30 10:38"),
        precio(&lapiz, "10", "2025-01-10 09:00"),
    ];

    let migrated = migrate_one(lapiz, &precios).unwrap();

    let historial: Vec<String> = migrated
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

    let migrated = migrate_one(lapiz, &precios).unwrap();

    assert_eq!(migrated.historial[0].desde, utc("2024-03-01 14:00"));
}

#[test]
fn se_dio_de_alta_cuando_tuvo_su_primer_precio() {
    let lapiz = producto_v1(1);
    let precios = [
        precio(&lapiz, "10", "2024-03-01 09:00"),
        precio(&lapiz, "8", "2022-06-30 10:38"),
    ];

    let migrated = migrate_one(lapiz, &precios).unwrap();

    assert_eq!(migrated.created_at, utc("2022-06-30 15:38"));
}

#[test]
fn la_ultima_edicion_es_el_datestamp_si_es_posterior_al_ultimo_precio() {
    let lapiz = producto_v1(1);
    let precios = [precio(&lapiz, "8", "2024-03-01 09:00")];

    let migrated = migrate_one(lapiz, &precios).unwrap();

    assert_eq!(migrated.updated_at, utc("2026-06-20 15:00"));
}

#[test]
fn la_ultima_edicion_es_el_ultimo_precio_si_cambio_despues_del_datestamp() {
    let lapiz = producto_v1(1649);
    let precios = [precio(&lapiz, "8", "2026-09-18 17:57")];

    let migrated = migrate_one(lapiz, &precios).unwrap();

    assert_eq!(migrated.updated_at, utc("2026-09-18 22:57"));
}

#[test]
fn el_nombre_junta_los_espacios_de_mas_y_conserva_las_mayusculas() {
    let mut cinta = producto_v1(1);
    cinta.nombre = " FLORATAPE  VERDE LIMON ".to_string();
    let precios = [precio(&cinta, "8", "2024-03-01 09:00")];

    let migrated = migrate_one(cinta, &precios).unwrap();

    assert_eq!(migrated.producto.nombre(), "FLORATAPE VERDE LIMON");
}

#[test]
fn pasa_marca_modelo_color_y_descripcion() {
    let mut lapiz = producto_v1(1);
    lapiz.marca = Some("MIRADO".to_string());
    lapiz.modelo = Some("2HB".to_string());
    lapiz.color = Some("AMARILLO".to_string());
    lapiz.descripcion = Some("Lápiz de grafito".to_string());
    let precios = [precio(&lapiz, "8", "2024-03-01 09:00")];

    let migrated = migrate_one(lapiz, &precios).unwrap();

    let p = &migrated.producto;
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

    let all_migrated = migrate(&v1, &precios).unwrap();

    let unidades: Vec<UnidadMedidaId> = all_migrated
        .iter()
        .map(|m| m.producto.unidad_medida_id())
        .collect();
    assert_eq!(unidades, [PIEZA, METRO, GRAMO]);
}

#[test]
fn la_categoria_fusionada_lleva_a_la_que_se_conservo() {
    let mut cinta = producto_v1(1);
    cinta.categoria_ids = vec![MERGED_CATEGORIA_V1];
    let precios = [precio(&cinta, "8", "2024-03-01 09:00")];

    let migrated = migrate_one(cinta, &precios).unwrap();

    assert_eq!(migrated.producto.categoria_id(), CATEGORIA_V2);
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
fn un_precio_vacio_en_el_historial_detiene_la_migracion() {
    let lapiz = producto_v1(7);
    let mut vacio = precio(&lapiz, "8", "2025-01-10 09:00");
    vacio.precio_venta = None;
    let precios = [precio(&lapiz, "8", "2024-03-01 09:00"), vacio];

    let error = migrate_one(lapiz, &precios).unwrap_err();

    assert_eq!(
        error,
        "Producto NID 7 «LAPIZ» de la v1: un precio de su historial está vacío."
    );
}

#[test]
fn un_producto_sin_datestamp_detiene_la_migracion() {
    let mut lapiz = producto_v1(7);
    lapiz.datestamp = None;
    let precios = [precio(&lapiz, "8", "2024-03-01 09:00")];

    let error = migrate_one(lapiz, &precios).unwrap_err();

    assert_eq!(
        error,
        "Producto NID 7 «LAPIZ» de la v1: no tiene fecha de última edición (datestamp)."
    );
}

#[test]
fn una_hora_que_no_existio_en_cancun_detiene_la_migracion() {
    // Hasta 2015 Cancún cambiaba de horario: el 6 de abril de 2014 se saltó de las 2:00 a las 3:00.
    let lapiz = producto_v1(1);
    let precios = [precio(&lapiz, "8", "2014-04-06 02:30")];

    let error = migrate_one(lapiz, &precios).unwrap_err();

    assert!(error.ends_with("la hora 2014-04-06 02:30:00 no existe o es ambigua en Cancún."));
}

#[test]
fn una_categoria_que_no_se_migro_detiene_la_migracion() {
    let mut lapiz = producto_v1(1);
    lapiz.categoria_ids = vec![Uuid::from_u128(3)];
    let precios = [precio(&lapiz, "8", "2024-03-01 09:00")];

    let error = migrate_one(lapiz, &precios).unwrap_err();

    assert!(error.ends_with("su categoría no está entre las migradas."));
}

#[test]
fn un_producto_sin_unidad_detiene_la_migracion() {
    let mut lapiz = producto_v1(1);
    lapiz.unidad = None;
    let precios = [precio(&lapiz, "8", "2024-03-01 09:00")];

    let error = migrate_one(lapiz, &precios).unwrap_err();

    assert!(error.ends_with("no tiene unidad."));
}

#[test]
fn un_nid_de_cero_o_negativo_detiene_la_migracion() {
    for nid in [0, -3] {
        let mut lapiz = producto_v1(1);
        lapiz.nid = nid;
        let precios = [precio(&lapiz, "8", "2024-03-01 09:00")];

        let error = migrate_one(lapiz, &precios).unwrap_err();

        assert_eq!(
            error,
            format!("Producto NID {nid} «LAPIZ» de la v1: el NID debe ser mayor que cero.")
        );
    }
}

#[test]
fn una_unidad_que_no_existe_en_la_v2_detiene_la_migracion() {
    let mut cinta = producto_v1(1);
    cinta.unidad = Some("GRAMOS".to_string());
    let precios = [precio(&cinta, "8", "2024-03-01 09:00")];
    let categorias = HashMap::from([(CATEGORIA_V1, CATEGORIA_V2)]);
    let solo_pieza = [UnidadMedida {
        id: PIEZA,
        nombre: "Pieza".to_string(),
        allows_fraction: false,
    }];

    let errors = productos(&[cinta], &precios, &categorias, &solo_pieza).unwrap_err();

    assert_eq!(
        errors,
        ["Producto NID 1 «LAPIZ» de la v1: la unidad «Gramo» no existe en la v2."]
    );
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

    let nids: Vec<&str> = errors
        .iter()
        .map(|e| e.split(" «").next().unwrap_or_default())
        .collect();
    assert_eq!(nids, ["Producto NID 1", "Producto NID 3"], "{errors:?}");
}
