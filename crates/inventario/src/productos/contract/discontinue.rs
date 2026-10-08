//! El contrato de descontinuar un producto y de volver a manejarlo.

use super::*;
use crate::productos::discontinue::MotivoDescontinuado;

fn motivo(text: &str) -> MotivoDescontinuado {
    MotivoDescontinuado::parse(text).unwrap()
}

pub async fn descontinuar_anota_la_fecha_y_el_motivo_y_se_ve_al_buscarlo(
    repo: &impl ProductosRepo,
    setup: &Setup,
) {
    let added = repo
        .add(new_producto(setup, "PLUMA AZUL", "12"), &setup.by)
        .await
        .unwrap();
    let before = kernel::Utc::now();

    let discontinued = repo
        .discontinue(added.id, motivo("el proveedor ya no lo vende"), &setup.by)
        .await
        .unwrap();

    let descontinuado = discontinued.descontinuado.clone().unwrap();
    assert_eq!(descontinuado.motivo, "el proveedor ya no lo vende");
    // La base guarda microsegundos; un segundo de holgura cubre cualquier reloj.
    assert!(descontinuado.at.timestamp() >= before.timestamp() - 1);
    assert_eq!(
        discontinued,
        Producto {
            descontinuado: Some(descontinuado),
            ..added.clone()
        }
    );
    assert_eq!(
        repo.get_by_nid(added.nid).await.unwrap(),
        Some(discontinued)
    );
}

pub async fn uno_ya_descontinuado_no_se_vuelve_a_descontinuar_y_conserva_su_fecha(
    repo: &impl ProductosRepo,
    setup: &Setup,
) {
    let added = repo
        .add(new_producto(setup, "PLUMA AZUL", "12"), &setup.by)
        .await
        .unwrap();
    let discontinued = repo
        .discontinue(added.id, motivo("ya no se vende"), &setup.by)
        .await
        .unwrap();

    let result = repo
        .discontinue(added.id, motivo("otro motivo"), &setup.by)
        .await;

    assert_eq!(result, Err(ProductoError::AlreadyDescontinuado));
    assert_eq!(
        repo.get_by_nid(added.nid).await.unwrap(),
        Some(discontinued)
    );
}

pub async fn volver_a_manejar_borra_la_fecha_y_el_motivo(repo: &impl ProductosRepo, setup: &Setup) {
    let added = repo
        .add(new_producto(setup, "PLUMA AZUL", "12"), &setup.by)
        .await
        .unwrap();
    repo.discontinue(added.id, motivo("ya no se vende"), &setup.by)
        .await
        .unwrap();

    let resumed = repo.resume(added.id, &setup.by).await.unwrap();

    assert_eq!(resumed, added);
    assert_eq!(repo.get_by_nid(added.nid).await.unwrap(), Some(added));
}

pub async fn volver_a_manejar_uno_que_se_maneja_no_cambia_nada(
    repo: &impl ProductosRepo,
    setup: &Setup,
) {
    let added = repo
        .add(new_producto(setup, "PLUMA AZUL", "12"), &setup.by)
        .await
        .unwrap();

    let resumed = repo.resume(added.id, &setup.by).await.unwrap();

    assert_eq!(resumed, added);
    assert_eq!(repo.list().await.unwrap(), [added]);
}

pub async fn descontinuar_o_volver_a_manejar_uno_que_no_existe_se_rechaza(
    repo: &impl ProductosRepo,
    setup: &Setup,
) {
    let nadie = ArticuloId(Uuid::from_u128(999));

    let discontinued = repo
        .discontinue(nadie, motivo("ya no se vende"), &setup.by)
        .await;
    let resumed = repo.resume(nadie, &setup.by).await;

    assert_eq!(discontinued, Err(ProductoError::NotFound));
    assert_eq!(resumed, Err(ProductoError::NotFound));
}

pub async fn con_un_usuario_que_no_existe_se_rechaza_sin_cambiar_nada(
    repo: &impl ProductosRepo,
    setup: &Setup,
) {
    let pluma = repo
        .add(new_producto(setup, "PLUMA AZUL", "12"), &setup.by)
        .await
        .unwrap();
    let lapiz = repo
        .add(new_producto(setup, "LAPIZ", "5"), &setup.by)
        .await
        .unwrap();
    let lapiz = repo
        .discontinue(lapiz.id, motivo("ya no se vende"), &setup.by)
        .await
        .unwrap();
    let nadie = Email::parse("nadie@x.mx").unwrap();

    let discontinued = repo
        .discontinue(pluma.id, motivo("ya no se vende"), &nadie)
        .await;
    let resumed = repo.resume(lapiz.id, &nadie).await;

    assert_eq!(discontinued, Err(ProductoError::UsuarioNotFound));
    assert_eq!(resumed, Err(ProductoError::UsuarioNotFound));
    assert_eq!(repo.list().await.unwrap(), [lapiz, pluma]);
}

pub async fn editar_un_descontinuado_lo_deja_descontinuado(
    repo: &impl ProductosRepo,
    setup: &Setup,
) {
    let added = repo
        .add(new_producto(setup, "PLUMA AZUL", "12"), &setup.by)
        .await
        .unwrap();
    let discontinued = repo
        .discontinue(added.id, motivo("ya no se vende"), &setup.by)
        .await
        .unwrap();

    let updated = repo
        .update(
            added.id,
            new_producto(setup, "PLUMA NEGRA", "12"),
            &setup.by,
        )
        .await
        .unwrap();

    assert_eq!(updated.descontinuado, discontinued.descontinuado);
    assert_eq!(repo.list().await.unwrap(), [updated]);
}
