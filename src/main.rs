use axum::{routing::get, Router};
use axum_login::AuthManagerLayerBuilder;
use sqlx::PgPool;
use tower::ServiceBuilder;
use tower_http::{services::ServeDir, trace::TraceLayer};
use tower_sessions::{Expiry, SessionManagerLayer};
use tower_sessions_sqlx_store::PostgresStore;

mod auth;
mod config;
mod db;
mod error;
mod filters;
mod modules;
mod templates;

// AppState se clona en cada request — todos los campos son baratos de clonar.
// PgPool es internamente un Arc (pool compartido, no una conexión por clone).
#[derive(Clone)]
pub struct AppState {
    pub pool: PgPool,
    pub config: config::Config,
    pub settings: db::settings::Settings,
}

#[tokio::main]
async fn main() {
    // dotenvy carga el .env solo si existe — en producción las vars vienen del entorno
    dotenvy::dotenv().ok();
    // JSON estructurado: en producción Loki parsea cada línea con `| json` en LogQL.
    // En desarrollo se puede leer igual, o poner RUST_LOG=debug para más detalle.
    tracing_subscriber::fmt().json().init();

    let cfg = config::Config::from_env();
    let addr = format!("0.0.0.0:{}", cfg.port);

    let pool = PgPool::connect(&cfg.database_url)
        .await
        .expect("No se pudo conectar a la base de datos");

    // --- Sesiones en PostgreSQL ---
    // PostgresStore crea/actualiza la tabla `tower_sessions` automáticamente con .migrate()
    let session_store = PostgresStore::new(pool.clone());
    session_store
        .migrate()
        .await
        .expect("No se pudo inicializar la tabla de sesiones");

    // OnInactivity: la sesión expira si el usuario no hace requests en 30 días
    let session_layer = SessionManagerLayer::new(session_store)
        .with_secure(false) // true solo con HTTPS — en k3s el TLS termina en el proxy
        .with_expiry(Expiry::OnInactivity(time::Duration::days(30)));

    // --- Auth ---
    // AuthBackend verifica credenciales y carga el usuario en cada request con sesión activa
    let auth_backend = auth::AuthBackend::new(pool.clone());
    let auth_layer = AuthManagerLayerBuilder::new(auth_backend, session_layer).build();

    let settings = db::settings::load(&pool)
        .await
        .expect("No se pudieron cargar los Settings desde la base de datos");

    let state = AppState { pool, config: cfg, settings };

    // Rutas protegidas: login_required! redirige a /login si no hay sesión
    let protected = Router::new()
        .route("/", get(modules::home::index))
        .route("/ventas", get(modules::ventas::index).post(modules::ventas::routes::crear))
        .route("/ventas/nueva", get(modules::ventas::routes::nueva))
        .route("/api/productos/buscar", get(modules::productos::routes::buscar))
        .route("/api/clientes/buscar", get(modules::clientes::routes::buscar))
        .route("/api/tipo-cambio", get(modules::ventas::routes::tipo_cambio))
        .route("/api/monedero/{cliente_id}", get(modules::ventas::routes::monedero))
        .route_layer(axum_login::login_required!(
            auth::AuthBackend,
            login_url = "/login"
        ));

    // Rutas públicas: login y assets estáticos
    let public = Router::new()
        .route("/login", get(auth::routes::login_get).post(auth::routes::login_post))
        .route("/logout", get(auth::routes::logout))
        .nest_service("/static", ServeDir::new("static"));

    let app = Router::new()
        .merge(protected)
        .merge(public)
        // ServiceBuilder aplica las capas en el orden escrito (de arriba a abajo)
        .layer(
            ServiceBuilder::new()
                .layer(TraceLayer::new_for_http())
                .layer(auth_layer),
        )
        .with_state(state);

    let listener = tokio::net::TcpListener::bind(&addr).await.unwrap();
    tracing::info!("Servidor en http://{}", addr);
    axum::serve(listener, app).await.unwrap();
}
