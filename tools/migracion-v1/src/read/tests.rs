use super::*;

/// Una v1 de juguete, con solo las columnas que se leen. CI no tiene la copia de la v1: así se
/// prueban los filtros de las consultas.
async fn create_toy_v1(pool: &PgPool) {
    for sql in [
        "CREATE TABLE categorias (id uuid PRIMARY KEY, nombre varchar)",
        "CREATE TABLE unidadesmedida (id uuid PRIMARY KEY, nombre varchar)",
        "CREATE TABLE productos (
            id uuid UNIQUE, nid serial PRIMARY KEY, nombre varchar NOT NULL DEFAULT 'LAPIZ',
            esservicio boolean, escompuesto boolean, unidadmedidaid uuid,
            marca varchar, modelo varchar, color varchar, descripcion text,
            datestamp timestamp DEFAULT '2026-06-20 10:00')",
        "CREATE TABLE categoriasproductos (id uuid, categoriaid uuid, productoid uuid)",
        "CREATE TABLE preciosproductos (
            id uuid, productoid uuid, precioventa numeric, fechacreado timestamp)",
        "INSERT INTO unidadesmedida VALUES ('00000000-0000-0000-0000-0000000000f1', 'PIEZA')",
        // 1 "CINTA" y 2 "CINTA " son de productos; 3 solo de un servicio; 4 de un kit; 5 vacía.
        "INSERT INTO categorias VALUES
            ('00000000-0000-0000-0000-000000000001', 'CINTA'),
            ('00000000-0000-0000-0000-000000000002', 'CINTA '),
            ('00000000-0000-0000-0000-000000000003', 'COPIAS'),
            ('00000000-0000-0000-0000-000000000004', 'KITS'),
            ('00000000-0000-0000-0000-000000000005', 'VACIA')",
        // a (NID 1) y b (NID 2) son productos; c es un servicio y d un kit.
        "INSERT INTO productos (id, esservicio, escompuesto) VALUES
            ('00000000-0000-0000-0000-00000000000a', false, false),
            ('00000000-0000-0000-0000-00000000000b', false, false),
            ('00000000-0000-0000-0000-00000000000c', true, false),
            ('00000000-0000-0000-0000-00000000000d', false, true)",
        // El producto a está dos veces en la misma categoría, como el NID 5 de la v1.
        "INSERT INTO categoriasproductos (categoriaid, productoid) VALUES
            ('00000000-0000-0000-0000-000000000001', '00000000-0000-0000-0000-00000000000a'),
            ('00000000-0000-0000-0000-000000000001', '00000000-0000-0000-0000-00000000000a'),
            ('00000000-0000-0000-0000-000000000002', '00000000-0000-0000-0000-00000000000b'),
            ('00000000-0000-0000-0000-000000000003', '00000000-0000-0000-0000-00000000000c'),
            ('00000000-0000-0000-0000-000000000004', '00000000-0000-0000-0000-00000000000d')",
    ] {
        sqlx::query(sql).execute(pool).await.unwrap();
    }
}

#[sqlx::test(migrations = false)]
async fn solo_lee_las_categorias_que_usa_algun_producto(pool: PgPool) {
    create_toy_v1(&pool).await;

    let v1 = read(&pool).await.unwrap();

    let nombres: Vec<&str> = v1.categorias.iter().map(|c| c.nombre.as_str()).collect();
    assert_eq!(nombres, ["CINTA", "CINTA "]);
}

#[sqlx::test(migrations = false)]
async fn espera_una_categoria_por_nombre_sin_contar_los_espacios_de_mas(pool: PgPool) {
    create_toy_v1(&pool).await;

    let v1 = read(&pool).await.unwrap();

    assert_eq!(v1.expected, RecordCounts { categorias: 1 });
}

#[sqlx::test(migrations = false)]
async fn espera_una_categoria_aunque_cambien_las_mayusculas_o_los_espacios_de_adentro(
    pool: PgPool,
) {
    create_toy_v1(&pool).await;
    for sql in [
        "INSERT INTO categorias VALUES
            ('00000000-0000-0000-0000-000000000006', 'Cinta  Roja'),
            ('00000000-0000-0000-0000-000000000007', 'CINTA ROJA')",
        "INSERT INTO productos (id, esservicio, escompuesto)
            VALUES ('00000000-0000-0000-0000-00000000000e', false, false)",
        "INSERT INTO categoriasproductos (categoriaid, productoid) VALUES
            ('00000000-0000-0000-0000-000000000006', '00000000-0000-0000-0000-00000000000a'),
            ('00000000-0000-0000-0000-000000000007', '00000000-0000-0000-0000-00000000000e')",
    ] {
        sqlx::query(sql).execute(&pool).await.unwrap();
    }

    let v1 = read(&pool).await.unwrap();

    assert_eq!(v1.expected, RecordCounts { categorias: 2 });
}

#[sqlx::test(migrations = false)]
async fn solo_lee_los_productos_ni_servicios_ni_kits(pool: PgPool) {
    create_toy_v1(&pool).await;

    let v1 = read(&pool).await.unwrap();

    let nids: Vec<i32> = v1.productos.iter().map(|p| p.nid).collect();
    assert_eq!(nids, [1, 2]);
}

#[sqlx::test(migrations = false)]
async fn un_producto_que_repite_su_categoria_la_trae_una_sola_vez(pool: PgPool) {
    create_toy_v1(&pool).await;

    let v1 = read(&pool).await.unwrap();

    assert_eq!(v1.productos[0].categoria_ids, [Uuid::from_u128(1)]);
}

#[sqlx::test(migrations = false)]
async fn un_producto_sin_categoria_se_lee_con_la_lista_vacia(pool: PgPool) {
    create_toy_v1(&pool).await;
    sqlx::query("DELETE FROM categoriasproductos WHERE categoriaid = '00000000-0000-0000-0000-000000000002'")
        .execute(&pool)
        .await
        .unwrap();

    let v1 = read(&pool).await.unwrap();

    assert!(v1.productos[1].categoria_ids.is_empty());
}

#[sqlx::test(migrations = false)]
async fn lee_la_unidad_por_su_nombre_en_la_v1(pool: PgPool) {
    create_toy_v1(&pool).await;
    sqlx::query(
        "UPDATE productos SET unidadmedidaid = '00000000-0000-0000-0000-0000000000f1' WHERE nid = 1",
    )
    .execute(&pool)
    .await
    .unwrap();

    let v1 = read(&pool).await.unwrap();

    assert_eq!(v1.productos[0].unidad.as_deref(), Some("PIEZA"));
    assert_eq!(v1.productos[1].unidad, None);
}

#[sqlx::test(migrations = false)]
async fn lee_el_historial_solo_de_los_productos_y_en_orden_por_fecha(pool: PgPool) {
    create_toy_v1(&pool).await;
    sqlx::query(
        "INSERT INTO preciosproductos (productoid, precioventa, fechacreado) VALUES
            ('00000000-0000-0000-0000-00000000000a', 12.50, '2024-03-01 09:00'),
            ('00000000-0000-0000-0000-00000000000a', 10, '2022-06-30 10:38'),
            ('00000000-0000-0000-0000-00000000000c', 5, '2023-01-01 12:00')",
    )
    .execute(&pool)
    .await
    .unwrap();

    let v1 = read(&pool).await.unwrap();

    let precios: Vec<String> = v1
        .precios
        .iter()
        .map(|p| format!("{} {}", p.precio_venta.unwrap(), p.created_at))
        .collect();
    assert_eq!(
        precios,
        ["10 2022-06-30 10:38:00", "12.50 2024-03-01 09:00:00"]
    );
}

#[sqlx::test(migrations = false)]
async fn un_producto_sin_datestamp_se_lee_para_que_las_reglas_digan_cual_es(pool: PgPool) {
    create_toy_v1(&pool).await;
    sqlx::query("UPDATE productos SET datestamp = NULL WHERE nid = 1")
        .execute(&pool)
        .await
        .unwrap();

    let v1 = read(&pool).await.unwrap();

    assert_eq!(v1.productos[0].datestamp, None);
}

#[sqlx::test(migrations = false)]
async fn un_precio_vacio_se_lee_para_que_las_reglas_digan_de_cual_producto_es(pool: PgPool) {
    create_toy_v1(&pool).await;
    sqlx::query(
        "INSERT INTO preciosproductos (productoid, precioventa, fechacreado)
         VALUES ('00000000-0000-0000-0000-00000000000a', NULL, '2024-03-01 09:00')",
    )
    .execute(&pool)
    .await
    .unwrap();

    let v1 = read(&pool).await.unwrap();

    assert_eq!(v1.precios[0].precio_venta, None);
}

#[sqlx::test(migrations = false)]
async fn si_no_se_sabe_si_un_articulo_es_servicio_o_kit_no_lo_deja_fuera_sin_avisar(pool: PgPool) {
    create_toy_v1(&pool).await;
    sqlx::query("UPDATE productos SET esservicio = NULL WHERE nid = 1")
        .execute(&pool)
        .await
        .unwrap();
    sqlx::query("UPDATE productos SET escompuesto = NULL WHERE nid = 2")
        .execute(&pool)
        .await
        .unwrap();

    let error = read(&pool).await.unwrap_err();

    assert_eq!(
        error.to_string(),
        "2 artículos de la v1 no dicen si son servicio o kit (esservicio o escompuesto vacío): \
         no se sabe si migrarlos como productos."
    );
}
