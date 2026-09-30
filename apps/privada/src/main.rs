//! Arranque de la aplicación privada: lee el entorno (capa 1), valida `negocio.toml` (capa 2),
//! inicia los logs y atiende peticiones. Si la configuración no es válida, no arranca.

use std::net::SocketAddr;
use std::path::PathBuf;
use std::process::ExitCode;

use negocio_config::load_negocio_config;
use privada::{VERSION, router};
use tracing_subscriber::EnvFilter;

/// Variables de entorno (capa 1: infraestructura y secretos).
struct Entorno {
    /// `DATABASE_URL`: obligatoria, sin valor por omisión. Lleva la contraseña: nunca va al log.
    database_url: String,
    /// `NEGOCIO_CONFIG`: ruta del archivo del negocio.
    negocio_config: PathBuf,
    /// `PORT`: puerto donde escucha.
    port: u16,
    /// `LOG_FORMAT`: `json` en producción; cualquier otro valor, legible para humanos.
    logs_json: bool,
}

impl Entorno {
    fn read() -> Result<Self, String> {
        let port = match std::env::var("PORT") {
            Ok(texto) => texto
                .parse()
                .map_err(|_| format!("PORT no es un número de puerto: {texto}"))?,
            Err(_) => 5100,
        };
        let database_url =
            std::env::var("DATABASE_URL").map_err(|_| "falta DATABASE_URL".to_string())?;
        Ok(Self {
            database_url,
            negocio_config: std::env::var("NEGOCIO_CONFIG")
                .unwrap_or_else(|_| "negocio.toml".into())
                .into(),
            port,
            logs_json: std::env::var("LOG_FORMAT").is_ok_and(|v| v == "json"),
        })
    }
}

fn init_logs(json: bool) {
    // RUST_LOG elige el nivel (por omisión, info).
    let filter = EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("info"));
    let logs = tracing_subscriber::fmt().with_env_filter(filter);
    if json {
        logs.json()
            .with_current_span(true)
            .with_span_list(false)
            .init();
    } else {
        logs.init();
    }
}

#[tokio::main]
async fn main() -> ExitCode {
    let entorno = match Entorno::read() {
        Ok(entorno) => entorno,
        Err(error) => {
            eprintln!("no se pudo arrancar: {error}");
            return ExitCode::FAILURE;
        }
    };
    init_logs(entorno.logs_json);

    let config = match load_negocio_config(&entorno.negocio_config) {
        Ok(config) => config,
        Err(error) => {
            tracing::error!(%error, "configuración inválida; la aplicación no arranca");
            return ExitCode::FAILURE;
        }
    };

    // Lo que corre, antes de todo lo demás. Sin secretos.
    tracing::info!(
        version = VERSION,
        negocio = %config.negocio.nombre,
        time_zone = %config.negocio.time_zone,
        port = entorno.port,
        "arrancando"
    );

    // El error de sqlx no incluye la URL ni la contraseña.
    let pool = match db::connect(&entorno.database_url).await {
        Ok(pool) => pool,
        Err(error) => {
            tracing::error!(%error, "no se pudo conectar a la base; la aplicación no arranca");
            return ExitCode::FAILURE;
        }
    };
    if let Err(error) = db::run_migrations(&pool).await {
        tracing::error!(%error, "falló una migración; la aplicación no arranca");
        return ExitCode::FAILURE;
    }
    tracing::info!("base de datos al día");

    let address = SocketAddr::from(([0, 0, 0, 0], entorno.port));
    let listener = match tokio::net::TcpListener::bind(address).await {
        Ok(listener) => listener,
        Err(error) => {
            tracing::error!(%error, %address, "no se pudo abrir el puerto");
            return ExitCode::FAILURE;
        }
    };
    tracing::info!(%address, "escuchando");

    if let Err(error) = axum::serve(
        listener,
        router(privada::AppState::new(pool, &config.negocio.nombre)),
    )
    .with_graceful_shutdown(shutdown_signal())
    .await
    {
        tracing::error!(%error, "el servidor se detuvo con error");
        return ExitCode::FAILURE;
    }
    tracing::info!("detenido");
    ExitCode::SUCCESS
}

/// Termina ordenadamente con Ctrl+C o con la señal que manda Kubernetes (SIGTERM).
async fn shutdown_signal() {
    let ctrl_c = tokio::signal::ctrl_c();
    let sigterm = async {
        match tokio::signal::unix::signal(tokio::signal::unix::SignalKind::terminate()) {
            Ok(mut signal) => {
                signal.recv().await;
            }
            Err(_) => std::future::pending::<()>().await,
        }
    };
    tokio::select! {
        _ = ctrl_c => {},
        _ = sigterm => {},
    }
}
