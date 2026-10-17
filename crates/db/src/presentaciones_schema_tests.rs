//! Lo que el esquema de presentaciones y códigos de barras garantiza por sí solo (migración 0011),
//! aunque algo escriba sin pasar por el área.

use kernel::Uuid;
use sqlx::PgPool;

/// Un artículo nuevo del `kind` dado, en Plumas y por pieza, sin renglón en `productos`.
async fn insert_articulo(pool: &PgPool, kind: &str, nombre: &str) -> Uuid {
    sqlx::query("INSERT INTO categorias (nombre) VALUES ('Plumas') ON CONFLICT DO NOTHING")
        .execute(pool)
        .await
        .unwrap();
    sqlx::query_scalar(
        "INSERT INTO articulos (kind, nombre, categoria_id, unidad_medida_id)
         SELECT $1, $2, c.id, u.id
         FROM categorias c, unidades_medida u
         WHERE c.nombre = 'Plumas' AND u.nombre = 'Pieza'
         RETURNING id",
    )
    .bind(kind)
    .bind(nombre)
    .fetch_one(pool)
    .await
    .unwrap()
}

/// Un producto nuevo: el artículo y su renglón en `productos`.
async fn insert_producto(pool: &PgPool, nombre: &str) -> Uuid {
    let id = insert_articulo(pool, "producto", nombre).await;
    sqlx::query("INSERT INTO productos (articulo_id) VALUES ($1)")
        .bind(id)
        .execute(pool)
        .await
        .unwrap();
    id
}

async fn insert_presentacion(
    pool: &PgPool,
    producto: Uuid,
    nombre: &str,
    factor: &str,
) -> Result<Uuid, sqlx::Error> {
    sqlx::query_scalar(
        "INSERT INTO presentaciones (producto_id, nombre, factor, precio_venta)
         VALUES ($1, $2, $3::numeric, 10) RETURNING id",
    )
    .bind(producto)
    .bind(nombre)
    .bind(factor)
    .fetch_one(pool)
    .await
}

async fn insert_codigo(
    pool: &PgPool,
    codigo: &str,
    producto: Uuid,
    presentacion: Option<Uuid>,
) -> Result<(), sqlx::Error> {
    sqlx::query(
        "INSERT INTO codigos_barras (codigo, producto_id, presentacion_id) VALUES ($1, $2, $3)",
    )
    .bind(codigo)
    .bind(producto)
    .bind(presentacion)
    .execute(pool)
    .await
    .map(|_| ())
}

#[sqlx::test]
async fn una_presentacion_trae_mas_de_una_unidad_base(pool: PgPool) {
    let hoja = insert_producto(&pool, "Hoja carta").await;

    for invalido in ["1", "0.5", "0", "-20"] {
        let result = insert_presentacion(&pool, hoja, &format!("De {invalido}"), invalido).await;
        assert!(result.is_err(), "aceptó el factor {invalido}");
    }
    assert!(
        insert_presentacion(&pool, hoja, "Paquete de 20", "20")
            .await
            .is_ok()
    );
    assert!(
        insert_presentacion(&pool, hoja, "Rollo de 1.5", "1.5")
            .await
            .is_ok()
    );
}

#[sqlx::test]
async fn una_presentacion_no_repite_nombre_en_su_producto_sin_importar_mayusculas_pero_si_en_otro(
    pool: PgPool,
) {
    let hoja = insert_producto(&pool, "Hoja carta").await;
    let oficio = insert_producto(&pool, "Hoja oficio").await;
    insert_presentacion(&pool, hoja, "Paquete de 100", "100")
        .await
        .unwrap();

    for repetido in ["Paquete de 100", "PAQUETE DE 100"] {
        let result = insert_presentacion(&pool, hoja, repetido, "100").await;
        assert!(result.is_err(), "aceptó «{repetido}»");
    }
    assert!(
        insert_presentacion(&pool, oficio, "Paquete de 100", "100")
            .await
            .is_ok()
    );
}

#[sqlx::test]
async fn una_presentacion_solo_cuelga_de_un_producto(pool: PgPool) {
    let servicio = insert_articulo(&pool, "servicio", "Copia").await;

    let result = insert_presentacion(&pool, servicio, "Paquete", "10").await;
    assert!(result.is_err());
}

#[sqlx::test]
async fn una_presentacion_puede_ir_sin_precio_pero_no_con_cero_ni_negativo(pool: PgPool) {
    let hoja = insert_producto(&pool, "Hoja carta").await;
    let insert = "INSERT INTO presentaciones (producto_id, nombre, factor, precio_venta)
                  VALUES ($1, $2, 100, $3::numeric)";

    for (nombre, precio) in [("Con cero", Some("0")), ("Negativo", Some("-10"))] {
        let result = sqlx::query(insert)
            .bind(hoja)
            .bind(nombre)
            .bind(precio)
            .execute(&pool)
            .await;
        assert!(result.is_err(), "aceptó el precio {precio:?}");
    }
    let sin_precio = sqlx::query(insert)
        .bind(hoja)
        .bind("Caja para comprar")
        .bind(None::<&str>)
        .execute(&pool)
        .await;
    assert!(sin_precio.is_ok());
}

#[sqlx::test]
async fn el_nombre_de_una_presentacion_no_pasa_de_150_ni_queda_en_blanco(pool: PgPool) {
    let hoja = insert_producto(&pool, "Hoja carta").await;
    let de_151 = "ñ".repeat(151);

    for invalido in ["", "   ", de_151.as_str()] {
        let result = insert_presentacion(&pool, hoja, invalido, "100").await;
        assert!(result.is_err(), "aceptó «{invalido}»");
    }
    let de_150 = "ñ".repeat(150);
    assert!(
        insert_presentacion(&pool, hoja, &de_150, "100")
            .await
            .is_ok()
    );
}

#[sqlx::test]
async fn un_codigo_solo_es_de_un_producto(pool: PgPool) {
    let servicio = insert_articulo(&pool, "servicio", "Copia").await;

    assert!(
        insert_codigo(&pool, "7501015200964", servicio, None)
            .await
            .is_err()
    );
}

#[sqlx::test]
async fn un_codigo_no_se_repite_pero_distingue_mayusculas(pool: PgPool) {
    let pluma = insert_producto(&pool, "Pluma azul").await;
    let otra = insert_producto(&pool, "Pluma azul").await;
    insert_codigo(&pool, "EDaq89393", pluma, None)
        .await
        .unwrap();

    assert!(insert_codigo(&pool, "EDaq89393", otra, None).await.is_err());
    assert!(insert_codigo(&pool, "EDAQ89393", otra, None).await.is_ok());
}

#[sqlx::test]
async fn un_producto_puede_tener_varios_codigos(pool: PgPool) {
    let pluma = insert_producto(&pool, "Pluma azul").await;

    for codigo in ["7501015200964", "7501015200971"] {
        insert_codigo(&pool, codigo, pluma, None).await.unwrap();
    }
}

#[sqlx::test]
async fn un_codigo_no_lleva_espacios_ni_queda_vacio_ni_pasa_de_50(pool: PgPool) {
    let pluma = insert_producto(&pool, "Pluma azul").await;
    let de_51 = "7".repeat(51);

    for invalido in ["", " ", "oaNi38386 ", "750 1015", de_51.as_str()] {
        let result = insert_codigo(&pool, invalido, pluma, None).await;
        assert!(result.is_err(), "aceptó «{invalido}»");
    }
    for valido in ["perm0#?rojo", "GF2411L'78", "7".repeat(50).as_str()] {
        insert_codigo(&pool, valido, pluma, None).await.unwrap();
    }
}

#[sqlx::test]
async fn un_codigo_solo_apunta_a_una_presentacion_de_su_producto(pool: PgPool) {
    let hoja = insert_producto(&pool, "Hoja carta").await;
    let oficio = insert_producto(&pool, "Hoja oficio").await;
    let paquete = insert_presentacion(&pool, hoja, "Paquete de 100", "100")
        .await
        .unwrap();

    assert!(
        insert_codigo(&pool, "111", oficio, Some(paquete))
            .await
            .is_err()
    );
    assert!(
        insert_codigo(&pool, "222", hoja, Some(paquete))
            .await
            .is_ok()
    );
}

#[sqlx::test]
async fn un_precio_del_historial_solo_apunta_a_una_presentacion_de_su_articulo(pool: PgPool) {
    let hoja = insert_producto(&pool, "Hoja carta").await;
    let oficio = insert_producto(&pool, "Hoja oficio").await;
    let paquete = insert_presentacion(&pool, hoja, "Paquete de 100", "100")
        .await
        .unwrap();
    let insert = "INSERT INTO precios_historial (articulo_id, presentacion_id, precio_venta)
                  VALUES ($1, $2, 50)";

    let ajeno = sqlx::query(insert)
        .bind(oficio)
        .bind(paquete)
        .execute(&pool)
        .await;
    let propio = sqlx::query(insert)
        .bind(hoja)
        .bind(paquete)
        .execute(&pool)
        .await;

    assert!(ajeno.is_err());
    assert!(propio.is_ok());
}
