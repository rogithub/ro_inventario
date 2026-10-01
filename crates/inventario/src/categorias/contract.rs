use super::*;

pub async fn agregada_aparece_en_la_lista(repo: &impl CategoriasRepo) {
    let added = repo
        .add(NewCategoria::new("Cuadernos").unwrap())
        .await
        .unwrap();
    assert_eq!(added.nombre, "Cuadernos");
    assert!(repo.list().await.unwrap().contains(&added));
}

pub async fn agregada_trae_su_id_y_la_lista_lo_conserva(repo: &impl CategoriasRepo) {
    let plumas = repo
        .add(NewCategoria::new("Plumas").unwrap())
        .await
        .unwrap();
    let hojas = repo.add(NewCategoria::new("Hojas").unwrap()).await.unwrap();

    assert_ne!(plumas.id, hojas.id);
    let list = repo.list().await.unwrap();
    for added in [plumas, hojas] {
        let listed = list.iter().find(|c| c.nombre == added.nombre).unwrap();
        assert_eq!(listed.id, added.id);
    }
}

pub async fn renombrar_cambia_el_nombre_y_conserva_el_id(repo: &impl CategoriasRepo) {
    let plumas = repo
        .add(NewCategoria::new("Plumas").unwrap())
        .await
        .unwrap();

    let renamed = repo
        .rename(plumas.id, NewCategoria::new("Bolígrafos").unwrap())
        .await
        .unwrap();

    assert_eq!(renamed.id, plumas.id);
    assert_eq!(renamed.nombre, "Bolígrafos");
    let list = repo.list().await.unwrap();
    assert!(list.contains(&renamed));
    assert!(!list.iter().any(|c| c.nombre == "Plumas"));
}

pub async fn renombrar_a_un_nombre_que_ya_existe_se_rechaza(repo: &impl CategoriasRepo) {
    repo.add(NewCategoria::new("Plumas").unwrap())
        .await
        .unwrap();
    let hojas = repo.add(NewCategoria::new("Hojas").unwrap()).await.unwrap();
    let before = repo.list().await.unwrap();

    let result = repo
        .rename(hojas.id, NewCategoria::new("PLUMAS").unwrap())
        .await;

    assert_eq!(
        result,
        Err(CategoriaError::DuplicateNombre("PLUMAS".into()))
    );
    assert_eq!(repo.list().await.unwrap(), before, "no debe cambiar nada");
}

pub async fn renombrar_cambiando_solo_mayusculas_se_permite(repo: &impl CategoriasRepo) {
    let plumas = repo
        .add(NewCategoria::new("plumas").unwrap())
        .await
        .unwrap();

    let renamed = repo
        .rename(plumas.id, NewCategoria::new("Plumas").unwrap())
        .await
        .unwrap();

    assert_eq!(renamed.nombre, "Plumas");
}

pub async fn renombrar_una_que_no_existe_avisa_aunque_el_nombre_este_ocupado(
    repo: &impl CategoriasRepo,
) {
    repo.add(NewCategoria::new("Plumas").unwrap())
        .await
        .unwrap();
    let nadie = CategoriaId(Uuid::from_u128(u128::MAX));

    let result = repo
        .rename(nadie, NewCategoria::new("Plumas").unwrap())
        .await;

    assert_eq!(result, Err(CategoriaError::NotFound));
}

pub async fn renombrar_una_que_no_existe_avisa(repo: &impl CategoriasRepo) {
    let nadie = CategoriaId(Uuid::from_u128(u128::MAX));

    let result = repo
        .rename(nadie, NewCategoria::new("Plumas").unwrap())
        .await;

    assert_eq!(result, Err(CategoriaError::NotFound));
}

pub async fn nombre_repetido_se_rechaza_sin_importar_mayusculas(repo: &impl CategoriasRepo) {
    repo.add(NewCategoria::new("Plumas").unwrap())
        .await
        .unwrap();
    let before = repo.list().await.unwrap();

    let result = repo.add(NewCategoria::new(" pLUMAS ").unwrap()).await;

    assert_eq!(
        result,
        Err(CategoriaError::DuplicateNombre("pLUMAS".into()))
    );
    assert_eq!(repo.list().await.unwrap(), before, "no debe agregar nada");
}

pub async fn la_lista_va_en_orden_alfabetico(repo: &impl CategoriasRepo) {
    for nombre in ["Zeta", "alfa", "Media"] {
        repo.add(NewCategoria::new(nombre).unwrap()).await.unwrap();
    }

    let nombres: Vec<String> = repo
        .list()
        .await
        .unwrap()
        .into_iter()
        .map(|c| c.nombre)
        .collect();
    assert_eq!(nombres, ["alfa", "Media", "Zeta"]);
}

/// Como Postgres con la collation `en_US.utf8` (desarrollo, CI y producción): sin importar
/// mayúsculas ni acentos, la ñ como n, y sin contar espacios ni signos.
pub async fn la_lista_ordena_como_postgres_acentos_espacios_y_signos(repo: &impl CategoriasRepo) {
    let esperado = [
        "albumes", "Álbumes", "alfa", "cob", "co-op", "coop", "Hojas a", "Hoja z", "ñandú", "nube",
        "Útiles",
    ];
    for nombre in [
        "Útiles", "Hoja z", "coop", "ñandú", "alfa", "co-op", "Álbumes", "nube", "Hojas a", "cob",
        "albumes",
    ] {
        repo.add(NewCategoria::new(nombre).unwrap()).await.unwrap();
    }

    let nombres: Vec<String> = repo
        .list()
        .await
        .unwrap()
        .into_iter()
        .map(|c| c.nombre)
        .filter(|n| esperado.contains(&n.as_str()))
        .collect();
    assert_eq!(nombres, esperado);
}
