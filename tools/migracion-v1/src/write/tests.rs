use std::collections::HashMap;

use inventario::categorias::{CategoriaId, NewCategoria};
use kernel::Uuid;

use super::*;

fn two_categorias() -> Categorias {
    Categorias {
        to_insert: vec![
            (
                CategoriaId(Uuid::from_u128(1)),
                NewCategoria::new("CINTA").unwrap(),
            ),
            (
                CategoriaId(Uuid::from_u128(2)),
                NewCategoria::new("GOMA").unwrap(),
            ),
        ],
        by_v1_id: HashMap::new(),
    }
}

async fn get_categorias(pool: &PgPool) -> Vec<(Uuid, String)> {
    sqlx::query_as("SELECT id, nombre FROM categorias ORDER BY nombre")
        .fetch_all(pool)
        .await
        .unwrap()
}

#[sqlx::test(migrator = "db::MIGRATOR")]
async fn escribe_las_categorias_con_el_uuid_de_la_v1(pool: PgPool) {
    let written = write(&pool, &two_categorias(), RecordCounts { categorias: 2 })
        .await
        .unwrap();

    assert_eq!(written, RecordCounts { categorias: 2 });
    assert_eq!(
        get_categorias(&pool).await,
        [
            (Uuid::from_u128(1), "CINTA".to_string()),
            (Uuid::from_u128(2), "GOMA".to_string()),
        ]
    );
}

#[sqlx::test(migrator = "db::MIGRATOR")]
async fn si_los_conteos_no_cuadran_no_deja_nada(pool: PgPool) {
    let error = write(&pool, &two_categorias(), RecordCounts { categorias: 3 })
        .await
        .unwrap_err();

    assert!(
        matches!(error, WriteError::CountMismatch { .. }),
        "{error:?}"
    );
    assert_eq!(get_categorias(&pool).await, []);
}

#[sqlx::test(migrator = "db::MIGRATOR")]
async fn se_niega_si_la_base_ya_tiene_categorias_y_no_la_toca(pool: PgPool) {
    sqlx::query("INSERT INTO categorias (nombre) VALUES ('Plumas')")
        .execute(&pool)
        .await
        .unwrap();

    let error = write(&pool, &two_categorias(), RecordCounts { categorias: 2 })
        .await
        .unwrap_err();

    assert!(
        matches!(
            error,
            WriteError::NotEmpty {
                categorias: 1,
                articulos: 0
            }
        ),
        "{error:?}"
    );
    let nombres: Vec<String> = get_categorias(&pool)
        .await
        .into_iter()
        .map(|(_, nombre)| nombre)
        .collect();
    assert_eq!(nombres, ["Plumas"]);
}
