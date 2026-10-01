use kernel::Uuid;

use super::*;

/// Lo que cada prueba necesita que ya exista: una categoría, una unidad y quien da de alta.
pub struct Setup {
    pub categoria: CategoriaId,
    pub unidad: UnidadMedidaId,
    pub by: Email,
}

fn new_producto(setup: &Setup, nombre: &str, precio: &str) -> NewProducto {
    NewProducto::new(ProductoFields {
        nombre,
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

pub async fn agregado_aparece_en_la_lista_con_todos_sus_datos(
    repo: &impl ProductosRepo,
    setup: &Setup,
) {
    let new = NewProducto::new(ProductoFields {
        nombre: "PLUMA AZUL",
        categoria_id: setup.categoria,
        unidad_medida_id: setup.unidad,
        precio_venta: "12.50",
        descripcion: "Punto mediano",
        marca: "Bic",
        modelo: "Cristal",
        color: "Azul",
    })
    .unwrap();

    let added = repo.add(new, &setup.by).await.unwrap();

    assert_eq!(added.nombre, "PLUMA AZUL");
    assert_eq!(added.categoria_id, setup.categoria);
    assert_eq!(added.unidad_medida_id, setup.unidad);
    assert_eq!(
        added.precio_venta.map(|p| p.value().to_string()),
        Some("12.50".into())
    );
    assert_eq!(added.descripcion.as_deref(), Some("Punto mediano"));
    assert_eq!(added.marca.as_deref(), Some("Bic"));
    assert_eq!(added.modelo.as_deref(), Some("Cristal"));
    assert_eq!(added.color.as_deref(), Some("Azul"));
    assert_eq!(repo.list().await.unwrap(), [added]);
}

pub async fn sin_precio_queda_por_llegar(repo: &impl ProductosRepo, setup: &Setup) {
    let added = repo
        .add(new_producto(setup, "PLUMA", ""), &setup.by)
        .await
        .unwrap();

    assert_eq!(added.precio_venta, None);
    assert_eq!(added.marca, None);
    assert_eq!(repo.list().await.unwrap(), [added]);
}

pub async fn cada_alta_recibe_un_nid_mayor_que_el_anterior(
    repo: &impl ProductosRepo,
    setup: &Setup,
) {
    let azul = repo
        .add(new_producto(setup, "PLUMA AZUL", "12"), &setup.by)
        .await
        .unwrap();
    let roja = repo
        .add(new_producto(setup, "PLUMA ROJA", "12"), &setup.by)
        .await
        .unwrap();

    assert!(roja.nid > azul.nid);
    assert_ne!(roja.id, azul.id);
}

pub async fn dos_productos_pueden_llamarse_igual_y_se_ordenan_por_nid(
    repo: &impl ProductosRepo,
    setup: &Setup,
) {
    let primero = repo
        .add(new_producto(setup, "FOLDER", "5"), &setup.by)
        .await
        .unwrap();
    let segundo = repo
        .add(new_producto(setup, "FOLDER", "6"), &setup.by)
        .await
        .unwrap();

    assert_eq!(repo.list().await.unwrap(), [primero, segundo]);
}

pub async fn la_lista_ordena_como_postgres(repo: &impl ProductosRepo, setup: &Setup) {
    for nombre in ["Útiles", "Hoja z", "ñandú", "alfa", "Hojas a", "nube"] {
        repo.add(new_producto(setup, nombre, "1"), &setup.by)
            .await
            .unwrap();
    }

    let nombres: Vec<String> = repo
        .list()
        .await
        .unwrap()
        .into_iter()
        .map(|p| p.nombre)
        .collect();
    assert_eq!(
        nombres,
        ["alfa", "Hojas a", "Hoja z", "ñandú", "nube", "Útiles"]
    );
}

pub async fn con_una_categoria_que_no_existe_se_rechaza_sin_agregar_nada(
    repo: &impl ProductosRepo,
    setup: &Setup,
) {
    let nadie = Setup {
        categoria: CategoriaId(Uuid::from_u128(u128::MAX)),
        unidad: setup.unidad,
        by: setup.by.clone(),
    };

    let result = repo
        .add(new_producto(&nadie, "PLUMA", "12"), &setup.by)
        .await;

    assert_eq!(result, Err(ProductoError::CategoriaNotFound));
    assert!(repo.list().await.unwrap().is_empty());
}

pub async fn con_una_unidad_que_no_existe_se_rechaza_sin_agregar_nada(
    repo: &impl ProductosRepo,
    setup: &Setup,
) {
    let nadie = Setup {
        categoria: setup.categoria,
        unidad: UnidadMedidaId(Uuid::from_u128(u128::MAX)),
        by: setup.by.clone(),
    };

    let result = repo
        .add(new_producto(&nadie, "PLUMA", "12"), &setup.by)
        .await;

    assert_eq!(result, Err(ProductoError::UnidadMedidaNotFound));
    assert!(repo.list().await.unwrap().is_empty());
}

pub async fn con_un_usuario_que_no_existe_se_rechaza_sin_agregar_nada(
    repo: &impl ProductosRepo,
    setup: &Setup,
) {
    let nadie = Email::parse("nadie@x.mx").unwrap();

    let result = repo.add(new_producto(setup, "PLUMA", "12"), &nadie).await;

    assert_eq!(result, Err(ProductoError::UsuarioNotFound));
    assert!(repo.list().await.unwrap().is_empty());
}
