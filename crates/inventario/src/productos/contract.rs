use kernel::Uuid;

use super::*;

/// Lo que cada prueba necesita que ya exista: dos categorías, una unidad y quien da de alta.
pub struct Setup {
    pub categoria: CategoriaId,
    pub otra_categoria: CategoriaId,
    pub unidad: UnidadMedidaId,
    pub by: Email,
}

fn new_producto(setup: &Setup, nombre: &str, precio: &str) -> NewProducto {
    NewProducto::new(ProductoFields {
        nombre,
        categoria_id: Some(setup.categoria),
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
        categoria_id: Some(setup.categoria),
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
        otra_categoria: setup.otra_categoria,
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
        otra_categoria: setup.otra_categoria,
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

/// Da de alta productos con esos nombres, en la categoría principal, y regresa los nuevos.
async fn add_all(repo: &impl ProductosRepo, setup: &Setup, nombres: &[&str]) -> Vec<Producto> {
    let mut added = Vec::new();
    for nombre in nombres {
        added.push(
            repo.add(new_producto(setup, nombre, "1"), &setup.by)
                .await
                .unwrap(),
        );
    }
    added
}

/// Los nombres encontrados, en orden, y el total.
async fn found(
    repo: &impl ProductosRepo,
    text: &str,
    categoria: Option<CategoriaId>,
) -> (Vec<String>, u64) {
    let results = repo
        .search(&SearchQuery::new(text), categoria, 100)
        .await
        .unwrap();
    let nombres = results.productos.into_iter().map(|p| p.nombre).collect();
    (nombres, results.total)
}

pub async fn se_busca_por_todas_las_palabras_en_cualquier_orden_y_sin_mayusculas(
    repo: &impl ProductosRepo,
    setup: &Setup,
) {
    add_all(
        repo,
        setup,
        &["CINTA ADHESIVA VERDE", "CINTA ROJA", "PEGAMENTO VERDE"],
    )
    .await;

    assert_eq!(
        found(repo, "verde cinta", None).await,
        (vec!["CINTA ADHESIVA VERDE".to_string()], 1)
    );
    assert_eq!(found(repo, "cinta", None).await.1, 2);
    assert_eq!(found(repo, "azul", None).await, (vec![], 0));
}

pub async fn se_busca_sin_importar_acentos_y_la_enie_cuenta_como_n(
    repo: &impl ProductosRepo,
    setup: &Setup,
) {
    add_all(repo, setup, &["LÁPIZ", "PIÑATA", "goma de borrar"]).await;

    for (text, nombre) in [
        ("lapiz", "LÁPIZ"),
        ("Lápiz", "LÁPIZ"),
        ("pinata", "PIÑATA"),
        ("piñata", "PIÑATA"),
        ("GOMA", "goma de borrar"),
        ("BÓRRAR", "goma de borrar"),
    ] {
        assert_eq!(
            found(repo, text, None).await.0,
            [nombre],
            "buscando «{text}»"
        );
    }
}

pub async fn el_nid_buscado_va_primero_y_tambien_salen_los_nombres_con_ese_numero(
    repo: &impl ProductosRepo,
    setup: &Setup,
) {
    let zapato = &add_all(repo, setup, &["ZAPATO"]).await[0];
    let cuaderno = format!("CUADERNO {} HOJAS", zapato.nid);
    add_all(repo, setup, &[&cuaderno, "OTRO"]).await;

    assert_eq!(
        found(repo, &zapato.nid.to_string(), None).await,
        (vec!["ZAPATO".to_string(), cuaderno], 2)
    );
}

pub async fn sin_busqueda_salen_todos_en_el_orden_de_la_lista(
    repo: &impl ProductosRepo,
    setup: &Setup,
) {
    add_all(repo, setup, &["nube", "Útiles", "alfa"]).await;

    assert_eq!(
        found(repo, "  ", None).await,
        (vec!["alfa".into(), "nube".into(), "Útiles".into()], 3)
    );
}

pub async fn se_puede_filtrar_por_categoria_con_o_sin_busqueda(
    repo: &impl ProductosRepo,
    setup: &Setup,
) {
    add_all(repo, setup, &["PLUMA AZUL", "PLUMA ROJA"]).await;
    let otra = Setup {
        categoria: setup.otra_categoria,
        otra_categoria: setup.categoria,
        unidad: setup.unidad,
        by: setup.by.clone(),
    };
    add_all(repo, &otra, &["PLUMÓN AZUL", "CUADERNO"]).await;

    assert_eq!(
        found(repo, "", Some(setup.otra_categoria)).await,
        (vec!["CUADERNO".into(), "PLUMÓN AZUL".into()], 2)
    );
    assert_eq!(
        found(repo, "azul", Some(setup.categoria)).await,
        (vec!["PLUMA AZUL".into()], 1)
    );
}

pub async fn el_limite_corta_los_resultados_pero_el_total_los_cuenta_todos(
    repo: &impl ProductosRepo,
    setup: &Setup,
) {
    add_all(repo, setup, &["PLUMA C", "PLUMA A", "PLUMA B"]).await;

    let results = repo
        .search(&SearchQuery::new("pluma"), None, 2)
        .await
        .unwrap();

    let nombres: Vec<_> = results.productos.iter().map(|p| &p.nombre).collect();
    assert_eq!(nombres, ["PLUMA A", "PLUMA B"]);
    assert_eq!(results.total, 3);
}

pub async fn los_signos_se_buscan_tal_cual(repo: &impl ProductosRepo, setup: &Setup) {
    add_all(
        repo,
        setup,
        &["DESCUENTO 50%", "DESCUENTO 500", "HOJA_A4", "HOJA A4"],
    )
    .await;

    assert_eq!(found(repo, "50%", None).await.0, ["DESCUENTO 50%"]);
    assert_eq!(found(repo, "hoja_", None).await.0, ["HOJA_A4"]);
    assert_eq!(found(repo, "\\", None).await.0, Vec::<String>::new());
}
