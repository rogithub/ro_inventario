//! Lo que el esquema de artículos garantiza por sí solo (migración 0007), aunque algo escriba
//! sin pasar por el área: la red de seguridad debajo de las reglas de `inventario`.

use kernel::{Decimal, Uuid};
use sqlx::PgPool;

/// La categoría Plumas (se crea la primera vez) y la unidad Pieza, para colgarles artículos.
async fn categoria_y_pieza(pool: &PgPool) -> (Uuid, Uuid) {
    sqlx::query("INSERT INTO categorias (nombre) VALUES ('Plumas') ON CONFLICT DO NOTHING")
        .execute(pool)
        .await
        .unwrap();
    let categoria: Uuid = sqlx::query_scalar("SELECT id FROM categorias WHERE nombre = 'Plumas'")
        .fetch_one(pool)
        .await
        .unwrap();
    let pieza: Uuid = sqlx::query_scalar("SELECT id FROM unidades_medida WHERE nombre = 'Pieza'")
        .fetch_one(pool)
        .await
        .unwrap();
    (categoria, pieza)
}

const INSERT_ARTICULO: &str =
    "INSERT INTO articulos (kind, nombre, categoria_id, unidad_medida_id, precio_venta)
     VALUES ($1, $2, $3, $4, $5) RETURNING id";

async fn insert_articulo(
    pool: &PgPool,
    kind: &str,
    nombre: &str,
    precio: Option<Decimal>,
) -> Result<Uuid, sqlx::Error> {
    let (categoria, pieza) = categoria_y_pieza(pool).await;
    sqlx::query_scalar(INSERT_ARTICULO)
        .bind(kind)
        .bind(nombre)
        .bind(categoria)
        .bind(pieza)
        .bind(precio)
        .fetch_one(pool)
        .await
}

#[sqlx::test]
async fn los_ids_nuevos_son_uuid_v7_en_las_tablas_nuevas_y_en_las_de_antes(pool: PgPool) {
    let articulo = insert_articulo(&pool, "producto", "Pluma azul", None)
        .await
        .unwrap();
    let unidad: Uuid = sqlx::query_scalar(
        "INSERT INTO unidades_medida (nombre, allows_fraction) VALUES ('Hoja', false) RETURNING id",
    )
    .fetch_one(&pool)
    .await
    .unwrap();
    let rol: Uuid = sqlx::query_scalar("INSERT INTO roles (nombre) VALUES ('Socia') RETURNING id")
        .fetch_one(&pool)
        .await
        .unwrap();
    let categoria: Uuid = sqlx::query_scalar("SELECT categoria_id FROM articulos")
        .fetch_one(&pool)
        .await
        .unwrap();
    let usuario: Uuid = sqlx::query_scalar(
        "INSERT INTO usuarios (email, nombre, password_hash, rol_id)
         SELECT 'ana@x.mx', 'Ana', 'x', id FROM roles WHERE nombre = 'Dueño' RETURNING id",
    )
    .fetch_one(&pool)
    .await
    .unwrap();
    let precio: Uuid = sqlx::query_scalar(
        "INSERT INTO precios_historial (articulo_id, precio_venta) VALUES ($1, 12.50) RETURNING id",
    )
    .bind(articulo)
    .fetch_one(&pool)
    .await
    .unwrap();

    for id in [articulo, unidad, rol, categoria, usuario, precio] {
        assert_eq!(id.get_version_num(), 7, "{id}");
    }
}

#[sqlx::test]
async fn el_nid_se_asigna_solo_y_es_consecutivo(pool: PgPool) {
    let (categoria, pieza) = categoria_y_pieza(&pool).await;
    let mut nids = Vec::new();
    for nombre in ["Pluma azul", "Pluma roja"] {
        let nid: i32 = sqlx::query_scalar(
            "INSERT INTO articulos (kind, nombre, categoria_id, unidad_medida_id)
             VALUES ('producto', $1, $2, $3) RETURNING nid",
        )
        .bind(nombre)
        .bind(categoria)
        .bind(pieza)
        .fetch_one(&pool)
        .await
        .unwrap();
        nids.push(nid);
    }
    assert_eq!(nids, [1, 2]);
}

#[sqlx::test]
async fn un_nid_de_la_v1_se_conserva_y_no_se_puede_repetir(pool: PgPool) {
    let (categoria, pieza) = categoria_y_pieza(&pool).await;
    let insert = "INSERT INTO articulos (nid, kind, nombre, categoria_id, unidad_medida_id)
                  VALUES (40, 'producto', $1, $2, $3) RETURNING nid";

    let hoja: i32 = sqlx::query_scalar(insert)
        .bind("HOJA BLANCA")
        .bind(categoria)
        .bind(pieza)
        .fetch_one(&pool)
        .await
        .unwrap();
    let repetido = sqlx::query_scalar::<_, i32>(insert)
        .bind("OTRA")
        .bind(categoria)
        .bind(pieza)
        .fetch_one(&pool)
        .await;

    assert_eq!(hoja, 40);
    assert!(repetido.is_err());
}

#[sqlx::test]
async fn el_kind_solo_acepta_producto_servicio_o_kit(pool: PgPool) {
    for kind in ["producto", "servicio", "kit"] {
        insert_articulo(&pool, kind, kind, None).await.unwrap();
    }
    assert!(
        insert_articulo(&pool, "insumo", "Mica", None)
            .await
            .is_err()
    );
}

#[sqlx::test]
async fn solo_un_articulo_de_tipo_producto_tiene_renglon_en_productos(pool: PgPool) {
    let insert = "INSERT INTO productos (articulo_id, marca) VALUES ($1, 'Bic')";
    let pluma = insert_articulo(&pool, "producto", "Pluma azul", None)
        .await
        .unwrap();
    let copia = insert_articulo(&pool, "servicio", "Copia BN", None)
        .await
        .unwrap();

    sqlx::query(insert)
        .bind(pluma)
        .execute(&pool)
        .await
        .unwrap();
    assert!(
        sqlx::query(insert)
            .bind(copia)
            .execute(&pool)
            .await
            .is_err()
    );
}

#[sqlx::test]
async fn descontinuar_lleva_motivo_y_el_motivo_solo_va_si_se_descontinuo(pool: PgPool) {
    let pluma = insert_articulo(&pool, "producto", "Pluma azul", None)
        .await
        .unwrap();

    let sin_motivo = sqlx::query("UPDATE articulos SET descontinuado_at = now() WHERE id = $1")
        .bind(pluma)
        .execute(&pool)
        .await;
    let motivo_sin_fecha =
        sqlx::query("UPDATE articulos SET motivo_descontinuado = 'ya no lo venden' WHERE id = $1")
            .bind(pluma)
            .execute(&pool)
            .await;
    let con_motivo = sqlx::query(
        "UPDATE articulos SET descontinuado_at = now(), motivo_descontinuado = 'ya no lo venden'
         WHERE id = $1",
    )
    .bind(pluma)
    .execute(&pool)
    .await;

    assert!(sin_motivo.is_err());
    assert!(motivo_sin_fecha.is_err());
    con_motivo.unwrap();
}

#[sqlx::test]
async fn el_precio_se_lee_como_decimal_y_no_acepta_cero_ni_negativos(pool: PgPool) {
    let pluma = insert_articulo(&pool, "producto", "Pluma azul", Some(Decimal::new(1250, 2)))
        .await
        .unwrap();
    let precio: Decimal = sqlx::query_scalar("SELECT precio_venta FROM articulos WHERE id = $1")
        .bind(pluma)
        .fetch_one(&pool)
        .await
        .unwrap();
    assert_eq!(precio, Decimal::new(1250, 2));

    for invalido in [Decimal::ZERO, Decimal::new(-1, 0)] {
        let result = insert_articulo(&pool, "producto", "Gratis", Some(invalido)).await;
        assert!(result.is_err(), "aceptó {invalido}");
    }
}

#[sqlx::test]
async fn el_nombre_de_un_articulo_no_pasa_de_150_ni_queda_en_blanco(pool: PgPool) {
    assert!(
        insert_articulo(&pool, "producto", &"ñ".repeat(150), None)
            .await
            .is_ok()
    );
    for invalido in ["ñ".repeat(151), "   ".to_string()] {
        let result = insert_articulo(&pool, "producto", &invalido, None).await;
        assert!(result.is_err(), "aceptó «{invalido}»");
    }
}

#[sqlx::test]
async fn dos_articulos_pueden_llamarse_igual(pool: PgPool) {
    insert_articulo(&pool, "producto", "FOLDER", None)
        .await
        .unwrap();
    insert_articulo(&pool, "producto", "FOLDER", None)
        .await
        .unwrap();
}

#[sqlx::test]
async fn la_base_rechaza_descripcion_marca_modelo_y_color_de_mas(pool: PgPool) {
    let pluma = insert_articulo(&pool, "producto", "Pluma azul", None)
        .await
        .unwrap();
    sqlx::query("INSERT INTO productos (articulo_id) VALUES ($1)")
        .bind(pluma)
        .execute(&pool)
        .await
        .unwrap();
    let casos = [
        ("UPDATE articulos SET descripcion = $2 WHERE id = $1", 1000),
        (
            "UPDATE productos SET marca = $2 WHERE articulo_id = $1",
            150,
        ),
        (
            "UPDATE productos SET modelo = $2 WHERE articulo_id = $1",
            200,
        ),
        (
            "UPDATE productos SET color = $2 WHERE articulo_id = $1",
            150,
        ),
    ];
    for (update, max) in casos {
        let de_mas = sqlx::query(update)
            .bind(pluma)
            .bind("ñ".repeat(max + 1))
            .execute(&pool)
            .await;
        assert!(de_mas.is_err(), "aceptó {} en: {update}", max + 1);
        sqlx::query(update)
            .bind(pluma)
            .bind("ñ".repeat(max))
            .execute(&pool)
            .await
            .unwrap();
    }
}
