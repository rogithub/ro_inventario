//! El contrato de buscar por NID y de editar un producto.

use super::*;

/// Lo que se manda al editar: todos los campos, como el formulario.
fn changed(setup: &Setup, nombre: &str, precio: &str) -> NewProducto {
    NewProducto::new(ProductoFields {
        nombre,
        categoria_id: Some(setup.otra_categoria),
        unidad_medida_id: setup.unidad,
        precio_venta: precio,
        descripcion: "Punto fino",
        marca: "Pelikan",
        modelo: "",
        color: "Negro",
    })
    .unwrap()
}

pub async fn se_encuentra_por_su_nid(repo: &impl ProductosRepo, setup: &Setup) {
    let added = repo
        .add(new_producto(setup, "PLUMA AZUL", "12"), &setup.by)
        .await
        .unwrap();

    assert_eq!(repo.get_by_nid(added.nid).await.unwrap(), Some(added));
    assert_eq!(repo.get_by_nid(Nid(999)).await.unwrap(), None);
}

pub async fn editar_cambia_todos_sus_datos_y_conserva_id_y_nid(
    repo: &impl ProductosRepo,
    setup: &Setup,
) {
    let added = repo
        .add(new_producto(setup, "PLUMA AZUL", "12"), &setup.by)
        .await
        .unwrap();

    let updated = repo
        .update(added.id, changed(setup, "PLUMA NEGRA", "13.50"), &setup.by)
        .await
        .unwrap();

    assert_eq!((updated.id, updated.nid), (added.id, added.nid));
    assert_eq!(updated.nombre, "PLUMA NEGRA");
    assert_eq!(updated.categoria_id, setup.otra_categoria);
    assert_eq!(
        updated.precio_venta.map(|p| p.value().to_string()),
        Some("13.50".into())
    );
    assert_eq!(updated.descripcion.as_deref(), Some("Punto fino"));
    assert_eq!(updated.marca.as_deref(), Some("Pelikan"));
    assert_eq!(updated.modelo, None);
    assert_eq!(updated.color.as_deref(), Some("Negro"));
    assert_eq!(repo.list().await.unwrap(), [updated]);
}

pub async fn a_uno_sin_precio_se_le_puede_poner_precio(repo: &impl ProductosRepo, setup: &Setup) {
    let added = repo
        .add(new_producto(setup, "PLUMA AZUL", ""), &setup.by)
        .await
        .unwrap();

    let updated = repo
        .update(added.id, changed(setup, "PLUMA AZUL", "12"), &setup.by)
        .await
        .unwrap();

    assert_eq!(
        updated.precio_venta.map(|p| p.value().to_string()),
        Some("12.00".into())
    );
}

pub async fn uno_con_precio_no_puede_quedar_sin_precio(repo: &impl ProductosRepo, setup: &Setup) {
    let added = repo
        .add(new_producto(setup, "PLUMA AZUL", "12"), &setup.by)
        .await
        .unwrap();

    let result = repo
        .update(added.id, changed(setup, "PLUMA NEGRA", ""), &setup.by)
        .await;

    assert_eq!(result, Err(ProductoError::CannotRemovePrecio));
    assert_eq!(repo.list().await.unwrap(), [added]);
}

pub async fn editar_uno_que_no_existe_se_rechaza(repo: &impl ProductosRepo, setup: &Setup) {
    let result = repo
        .update(
            ArticuloId(Uuid::from_u128(999)),
            changed(setup, "PLUMA", "12"),
            &setup.by,
        )
        .await;

    assert_eq!(result, Err(ProductoError::NotFound));
}

pub async fn editar_con_una_categoria_que_no_existe_se_rechaza_sin_cambiar_nada(
    repo: &impl ProductosRepo,
    setup: &Setup,
) {
    let added = repo
        .add(new_producto(setup, "PLUMA AZUL", "12"), &setup.by)
        .await
        .unwrap();
    let otra = Setup {
        otra_categoria: CategoriaId(Uuid::from_u128(999)),
        ..setup.clone()
    };

    let result = repo
        .update(added.id, changed(&otra, "PLUMA NEGRA", "13"), &setup.by)
        .await;

    assert_eq!(result, Err(ProductoError::CategoriaNotFound));
    assert_eq!(repo.list().await.unwrap(), [added]);
}

pub async fn editar_con_una_unidad_que_no_existe_se_rechaza_sin_cambiar_nada(
    repo: &impl ProductosRepo,
    setup: &Setup,
) {
    let added = repo
        .add(new_producto(setup, "PLUMA AZUL", "12"), &setup.by)
        .await
        .unwrap();
    let otra = Setup {
        unidad: UnidadMedidaId(Uuid::from_u128(999)),
        ..setup.clone()
    };

    let result = repo
        .update(added.id, changed(&otra, "PLUMA NEGRA", "13"), &setup.by)
        .await;

    assert_eq!(result, Err(ProductoError::UnidadMedidaNotFound));
    assert_eq!(repo.list().await.unwrap(), [added]);
}

pub async fn editar_con_un_usuario_que_no_existe_se_rechaza_sin_cambiar_nada(
    repo: &impl ProductosRepo,
    setup: &Setup,
) {
    let added = repo
        .add(new_producto(setup, "PLUMA AZUL", "12"), &setup.by)
        .await
        .unwrap();
    let nadie = Email::parse("nadie@x.mx").unwrap();

    let result = repo
        .update(added.id, changed(setup, "PLUMA NEGRA", "13"), &nadie)
        .await;

    assert_eq!(result, Err(ProductoError::UsuarioNotFound));
    assert_eq!(repo.list().await.unwrap(), [added]);
}
