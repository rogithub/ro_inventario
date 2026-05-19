use axum::{response::Redirect, routing::get, Router};
use sqlx::PgPool;
use tower::ServiceBuilder;
use tower_http::{services::ServeDir, trace::TraceLayer};

mod config;
mod error;

// AppState se clona en cada request — todos los campos deben ser baratos de clonar.
// PgPool es un Arc interno, Config son strings. Ambos son Clone sin costo.
#[derive(Clone)]
pub struct AppState {
    pub pool: PgPool,
    pub config: config::Config,
}

#[tokio::main]
async fn main() {
    // dotenvy carga el .env solo si existe — en producción las vars vienen del entorno directamente
    dotenvy::dotenv().ok();

    // tracing_subscriber::fmt imprime los logs al stdout en formato legible
    tracing_subscriber::fmt::init();

    let cfg = config::Config::from_env();
    let addr = format!("0.0.0.0:{}", cfg.port);

    let pool = PgPool::connect(&cfg.database_url)
        .await
        .expect("No se pudo conectar a la base de datos");

    let state = AppState { pool, config: cfg };

    let app = Router::new()
        .route("/", get(|| async { Redirect::permanent("/ventas") }))
        .nest_service("/static", ServeDir::new("static"))
        // TraceLayer loguea método, path, status y latencia de cada request via tracing
        .layer(ServiceBuilder::new().layer(TraceLayer::new_for_http()))
        .with_state(state);

    let listener = tokio::net::TcpListener::bind(&addr).await.unwrap();
    tracing::info!("Servidor en http://{}", addr);
    axum::serve(listener, app).await.unwrap();
}
