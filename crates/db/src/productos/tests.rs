use inventario::categorias::{CategoriasRepo, NewCategoria};
use inventario::productos::contract::{self, Setup};
use inventario::productos::{NewProducto, ProductoFields};
use inventario::unidades_medida::UnidadesMedidaRepo;
use usuarios::usuarios::{NewUsuario, UsuariosRepo};

use super::*;
use crate::{PgCategorias, PgUnidadesMedida, PgUsuarios};

/// La categoría Plumas, la unidad Pieza de la migración y la usuaria Ana.
async fn setup(pool: &PgPool) -> Setup {
    let categoria = PgCategorias::new(pool.clone())
        .add(NewCategoria::new("Plumas").unwrap())
        .await
        .unwrap();
    let pieza = PgUnidadesMedida::new(pool.clone())
        .list()
        .await
        .unwrap()
        .into_iter()
        .find(|u| u.nombre == "Pieza")
        .unwrap();
    let ana = PgUsuarios::new(pool.clone())
        .add(NewUsuario::new("ana@x.mx", "Ana", "Dueño", "caja-de-lapices").unwrap())
        .await
        .unwrap();
    Setup {
        categoria: categoria.id,
        unidad: pieza.id,
        by: ana.email,
    }
}

fn pluma(setup: &Setup, precio: &str) -> NewProducto {
    NewProducto::new(ProductoFields {
        nombre: "PLUMA AZUL",
        categoria_id: setup.categoria,
        unidad_medida_id: setup.unidad,
        precio_venta: precio,
        descripcion: "",
        marca: "",
        modelo: "",
        color: "",
    })
    .unwrap()
}

#[sqlx::test]
async fn cumple_el_contrato_agregado_aparece_con_todos_sus_datos(pool: PgPool) {
    let setup = setup(&pool).await;
    contract::agregado_aparece_en_la_lista_con_todos_sus_datos(&PgProductos::new(pool), &setup)
        .await;
}

#[sqlx::test]
async fn cumple_el_contrato_sin_precio_queda_por_llegar(pool: PgPool) {
    let setup = setup(&pool).await;
    contract::sin_precio_queda_por_llegar(&PgProductos::new(pool), &setup).await;
}

#[sqlx::test]
async fn cumple_el_contrato_nid_mayor(pool: PgPool) {
    let setup = setup(&pool).await;
    contract::cada_alta_recibe_un_nid_mayor_que_el_anterior(&PgProductos::new(pool), &setup).await;
}

#[sqlx::test]
async fn cumple_el_contrato_nombres_repetidos(pool: PgPool) {
    let setup = setup(&pool).await;
    contract::dos_productos_pueden_llamarse_igual_y_se_ordenan_por_nid(
        &PgProductos::new(pool),
        &setup,
    )
    .await;
}

#[sqlx::test]
async fn cumple_el_contrato_orden_como_postgres(pool: PgPool) {
    let setup = setup(&pool).await;
    contract::la_lista_ordena_como_postgres(&PgProductos::new(pool), &setup).await;
}

#[sqlx::test]
async fn cumple_el_contrato_categoria_que_no_existe(pool: PgPool) {
    let setup = setup(&pool).await;
    contract::con_una_categoria_que_no_existe_se_rechaza_sin_agregar_nada(
        &PgProductos::new(pool),
        &setup,
    )
    .await;
}

#[sqlx::test]
async fn cumple_el_contrato_unidad_que_no_existe(pool: PgPool) {
    let setup = setup(&pool).await;
    contract::con_una_unidad_que_no_existe_se_rechaza_sin_agregar_nada(
        &PgProductos::new(pool),
        &setup,
    )
    .await;
}

#[sqlx::test]
async fn cumple_el_contrato_usuario_que_no_existe(pool: PgPool) {
    let setup = setup(&pool).await;
    contract::con_un_usuario_que_no_existe_se_rechaza_sin_agregar_nada(
        &PgProductos::new(pool),
        &setup,
    )
    .await;
}

#[sqlx::test]
async fn el_alta_con_precio_deja_el_precio_en_su_historial_con_quien_lo_puso(pool: PgPool) {
    let setup = setup(&pool).await;
    let added = PgProductos::new(pool.clone())
        .add(pluma(&setup, "12.50"), &setup.by)
        .await
        .unwrap();

    let historial: Vec<(Decimal, String)> = sqlx::query_as(
        "SELECT h.precio_venta, u.email FROM precios_historial h
         JOIN usuarios u ON u.id = h.updated_by WHERE h.articulo_id = $1",
    )
    .bind(added.id.0)
    .fetch_all(&pool)
    .await
    .unwrap();
    assert_eq!(historial, [(Decimal::new(1250, 2), "ana@x.mx".to_string())]);
}

#[sqlx::test]
async fn el_alta_sin_precio_no_escribe_historial(pool: PgPool) {
    let setup = setup(&pool).await;
    PgProductos::new(pool.clone())
        .add(pluma(&setup, ""), &setup.by)
        .await
        .unwrap();

    let total: i64 = sqlx::query_scalar("SELECT count(*) FROM precios_historial")
        .fetch_one(&pool)
        .await
        .unwrap();
    assert_eq!(total, 0);
}

#[sqlx::test]
async fn el_alta_anota_quien_lo_dio_de_alta_y_su_id_es_v7(pool: PgPool) {
    let setup = setup(&pool).await;
    let added = PgProductos::new(pool.clone())
        .add(pluma(&setup, "12"), &setup.by)
        .await
        .unwrap();

    let (kind, email): (String, String) = sqlx::query_as(
        "SELECT a.kind, u.email FROM articulos a JOIN usuarios u ON u.id = a.updated_by
         WHERE a.id = $1",
    )
    .bind(added.id.0)
    .fetch_one(&pool)
    .await
    .unwrap();
    assert_eq!((kind.as_str(), email.as_str()), ("producto", "ana@x.mx"));
    assert_eq!(added.id.0.get_version_num(), 7);
}

#[sqlx::test]
async fn la_lista_no_trae_servicios_ni_kits(pool: PgPool) {
    let setup = setup(&pool).await;
    sqlx::query(
        "INSERT INTO articulos (kind, nombre, categoria_id, unidad_medida_id)
         VALUES ('servicio', 'COPIA BN', $1, $2), ('kit', 'PAGO DE SERVICIO', $1, $2)",
    )
    .bind(setup.categoria.0)
    .bind(setup.unidad.0)
    .execute(&pool)
    .await
    .unwrap();

    assert!(PgProductos::new(pool).list().await.unwrap().is_empty());
}

#[sqlx::test]
async fn la_migracion_no_trae_productos(pool: PgPool) {
    assert!(PgProductos::new(pool).list().await.unwrap().is_empty());
}

#[sqlx::test]
async fn un_usuario_desactivado_no_puede_dar_de_alta(pool: PgPool) {
    let setup = setup(&pool).await;
    sqlx::query("UPDATE usuarios SET deactivated_at = now()")
        .execute(&pool)
        .await
        .unwrap();

    let result = PgProductos::new(pool.clone())
        .add(pluma(&setup, "12"), &setup.by)
        .await;

    assert_eq!(result, Err(ProductoError::UsuarioNotFound));
}

#[sqlx::test]
async fn un_alta_rechazada_no_deja_nada_en_ninguna_tabla(pool: PgPool) {
    let setup = setup(&pool).await;
    let otra_categoria = Setup {
        categoria: CategoriaId(kernel::Uuid::from_u128(u128::MAX)),
        unidad: setup.unidad,
        by: setup.by.clone(),
    };
    let repo = PgProductos::new(pool.clone());
    let nadie = Email::parse("nadie@x.mx").unwrap();

    assert!(
        repo.add(pluma(&otra_categoria, "12"), &setup.by)
            .await
            .is_err()
    );
    assert!(repo.add(pluma(&setup, "12"), &nadie).await.is_err());

    let (articulos, productos, historial): (i64, i64, i64) = sqlx::query_as(
        "SELECT (SELECT count(*) FROM articulos), (SELECT count(*) FROM productos),
                (SELECT count(*) FROM precios_historial)",
    )
    .fetch_one(&pool)
    .await
    .unwrap();
    assert_eq!((articulos, productos, historial), (0, 0, 0));
}
