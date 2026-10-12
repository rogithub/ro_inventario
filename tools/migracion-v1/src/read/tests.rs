use super::*;

/// Una v1 de juguete, con solo las columnas que se leen. CI no tiene la copia de la v1: así se
/// prueban los filtros de las consultas.
async fn create_toy_v1(pool: &PgPool) {
    for sql in [
        "CREATE TABLE categorias (id uuid PRIMARY KEY, nombre varchar)",
        "CREATE TABLE productos (id uuid PRIMARY KEY, esservicio boolean, escompuesto boolean)",
        "CREATE TABLE categoriasproductos (id uuid, categoriaid uuid, productoid uuid)",
        // 1 "CINTA" y 2 "CINTA " son de productos; 3 solo de un servicio; 4 de un kit; 5 vacía.
        "INSERT INTO categorias VALUES
            ('00000000-0000-0000-0000-000000000001', 'CINTA'),
            ('00000000-0000-0000-0000-000000000002', 'CINTA '),
            ('00000000-0000-0000-0000-000000000003', 'COPIAS'),
            ('00000000-0000-0000-0000-000000000004', 'KITS'),
            ('00000000-0000-0000-0000-000000000005', 'VACIA')",
        "INSERT INTO productos VALUES
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
        "INSERT INTO productos VALUES ('00000000-0000-0000-0000-00000000000e', false, false)",
        "INSERT INTO categoriasproductos (categoriaid, productoid) VALUES
            ('00000000-0000-0000-0000-000000000006', '00000000-0000-0000-0000-00000000000a'),
            ('00000000-0000-0000-0000-000000000007', '00000000-0000-0000-0000-00000000000e')",
    ] {
        sqlx::query(sql).execute(&pool).await.unwrap();
    }

    let v1 = read(&pool).await.unwrap();

    assert_eq!(v1.expected, RecordCounts { categorias: 2 });
}
