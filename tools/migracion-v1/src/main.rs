//! Copia los datos de la versión uno a una base vacía de la v2 (paso 7d: categorías; productos e
//! historial de precios en 7d2). Lee la v1 de `V1_DATABASE_URL` y escribe en `DATABASE_URL`;
//! las dos llevan contraseña: nunca se imprimen. En desarrollo: herramientas/migrar-v1.sh.
//!
//! Primero aplica las migraciones de la v2, como la aplicación al arrancar. Si un dato de la v1 no
//! pasa las reglas de la v2, o los conteos no cuadran, termina con error y no escribe nada.

mod read;
mod transform;
mod write;

use std::process::ExitCode;

use db::PgUnidadesMedida;
use inventario::unidades_medida::UnidadesMedidaRepo;

#[tokio::main]
async fn main() -> ExitCode {
    match run().await {
        Ok(()) => ExitCode::SUCCESS,
        Err(errors) => {
            for error in errors {
                eprintln!("{error}");
            }
            ExitCode::FAILURE
        }
    }
}

async fn run() -> Result<(), Vec<String>> {
    let v1_url = env("V1_DATABASE_URL")?;
    let v2_url = env("DATABASE_URL")?;

    let v1 = db::connect(&v1_url)
        .await
        .map_err(|e| vec![format!("No se pudo conectar a la v1: {e}")])?;
    let v2 = db::connect(&v2_url)
        .await
        .map_err(|e| vec![format!("No se pudo conectar a la v2: {e}")])?;
    db::run_migrations(&v2).await.map_err(|e| {
        vec![format!(
            "No se pudieron aplicar las migraciones de la v2: {e}"
        )]
    })?;

    let v1_data = read::read(&v1)
        .await
        .map_err(|e| vec![format!("No se pudo leer la v1: {e}")])?;
    let categorias = transform::categorias(&v1_data.categorias)?;
    let unidades = PgUnidadesMedida::new(v2.clone())
        .list()
        .await
        .map_err(|e| vec![format!("No se pudieron leer las unidades de la v2: {e}")])?;
    // Antes de escribir: si un producto no pasa las reglas, no queda nada, ni las categorías.
    let productos = transform::productos::productos(
        &v1_data.productos,
        &v1_data.precios,
        &categorias.by_v1_id,
        &unidades,
    )?;
    let written = write::write(&v2, &categorias, v1_data.expected)
        .await
        .map_err(|e| vec![e.to_string()])?;

    println!(
        "Listo, cuadra con la v1: {} categorías. {} productos y {} precios de su historial pasan \
         las reglas (todavía no se escriben: paso 7d2b).",
        written.categorias,
        productos.len(),
        productos.iter().map(|p| p.historial.len()).sum::<usize>()
    );
    Ok(())
}

fn env(name: &str) -> Result<String, Vec<String>> {
    std::env::var(name).map_err(|_| vec![format!("falta {name}")])
}
