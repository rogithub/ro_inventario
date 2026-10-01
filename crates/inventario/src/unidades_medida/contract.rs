use super::*;

pub async fn agregada_aparece_en_la_lista(repo: &impl UnidadesMedidaRepo) {
    let added = repo
        .add(NewUnidadMedida::new("Cuartilla", true).unwrap())
        .await
        .unwrap();
    assert_eq!(added.nombre, "Cuartilla");
    assert!(added.allows_fraction);
    assert!(repo.list().await.unwrap().contains(&added));
}

pub async fn agregada_trae_su_id_y_la_lista_lo_conserva(repo: &impl UnidadesMedidaRepo) {
    let hoja = repo
        .add(NewUnidadMedida::new("Hoja", false).unwrap())
        .await
        .unwrap();
    let litro = repo
        .add(NewUnidadMedida::new("Litro", true).unwrap())
        .await
        .unwrap();

    assert_ne!(hoja.id, litro.id);
    let list = repo.list().await.unwrap();
    for added in [hoja, litro] {
        let listed = list.iter().find(|u| u.nombre == added.nombre).unwrap();
        assert_eq!(listed.id, added.id);
    }
}

pub async fn nombre_repetido_se_rechaza_sin_importar_mayusculas(repo: &impl UnidadesMedidaRepo) {
    repo.add(NewUnidadMedida::new("Hoja", false).unwrap())
        .await
        .unwrap();
    let before = repo.list().await.unwrap();

    let result = repo
        .add(NewUnidadMedida::new(" hOJA ", true).unwrap())
        .await;

    assert_eq!(
        result,
        Err(UnidadMedidaError::DuplicateNombre("hOJA".into()))
    );
    assert_eq!(repo.list().await.unwrap(), before, "no debe agregar nada");
}

pub async fn la_lista_va_en_orden_alfabetico(repo: &impl UnidadesMedidaRepo) {
    for nombre in ["Zeta", "Alfa", "Media"] {
        repo.add(NewUnidadMedida::new(nombre, false).unwrap())
            .await
            .unwrap();
    }

    let nombres: Vec<String> = repo
        .list()
        .await
        .unwrap()
        .into_iter()
        .map(|u| u.nombre)
        .collect();
    let mut sorted = nombres.clone();
    sorted.sort_by_key(|n| n.to_lowercase());
    assert_eq!(nombres, sorted);
}

/// Como Postgres con la collation `en_US.utf8` (desarrollo, CI y producción): sin importar
/// mayúsculas ni acentos, la ñ como n, y sin contar espacios ni signos.
pub async fn la_lista_ordena_como_postgres_acentos_espacios_y_signos(
    repo: &impl UnidadesMedidaRepo,
) {
    let esperado = [
        "albumes", "Álbumes", "alfa", "cob", "co-op", "coop", "Hojas a", "Hoja z", "ñandú", "nube",
        "Útiles",
    ];
    for nombre in [
        "Útiles", "Hoja z", "coop", "ñandú", "alfa", "co-op", "Álbumes", "nube", "Hojas a", "cob",
        "albumes",
    ] {
        repo.add(NewUnidadMedida::new(nombre, false).unwrap())
            .await
            .unwrap();
    }

    let nombres: Vec<String> = repo
        .list()
        .await
        .unwrap()
        .into_iter()
        .map(|u| u.nombre)
        .filter(|n| esperado.contains(&n.as_str()))
        .collect();
    assert_eq!(nombres, esperado);
}
