//! Aplicación privada (punto de venta): arma el router y sus piezas. Sin reglas del negocio.

use axum::extract::State;
use axum::http::{Request, StatusCode};
use axum::routing::get;
use axum::{Json, Router};
use db::PgPool;
use serde::Serialize;
use tower_http::request_id::{MakeRequestUuid, PropagateRequestIdLayer, SetRequestIdLayer};
use tower_http::trace::{DefaultOnResponse, TraceLayer};
use tracing::Level;

/// Versión de esta compilación; aparece en `/health` y en el log de arranque.
pub const VERSION: &str = env!("CARGO_PKG_VERSION");

/// Todas las rutas de la aplicación, con el id de petición y el log por petición.
pub fn router(pool: PgPool) -> Router {
    Router::new()
        .route("/health", get(health))
        .with_state(pool)
        .layer(
            tower::ServiceBuilder::new()
                // Cada petición recibe un id (x-request-id) que va en sus logs y en la respuesta:
                // "falló la venta, error a1b2c3" lleva directo a sus líneas de log.
                .layer(SetRequestIdLayer::x_request_id(MakeRequestUuid))
                .layer(
                    TraceLayer::new_for_http()
                        .make_span_with(span_de_peticion)
                        // Cada respuesta queda en info (estado y cuánto tardó), para rastrearla por su id.
                        .on_response(DefaultOnResponse::new().level(Level::INFO)),
                )
                .layer(PropagateRequestIdLayer::x_request_id()),
        )
}

fn span_de_peticion<B>(request: &Request<B>) -> tracing::Span {
    let request_id = request
        .headers()
        .get("x-request-id")
        .and_then(|v| v.to_str().ok())
        .unwrap_or("-");
    tracing::info_span!(
        "request",
        method = %request.method(),
        uri = %request.uri(),
        request_id = %request_id,
    )
}

#[derive(Serialize)]
struct Health {
    status: &'static str,
    version: &'static str,
    db: &'static str,
}

/// Para las sondas de Kubernetes: 503 si la base no responde.
async fn health(State(pool): State<PgPool>) -> (StatusCode, Json<Health>) {
    let (code, status, db) = if db::is_db_alive(&pool).await {
        (StatusCode::OK, "ok", "ok")
    } else {
        (StatusCode::SERVICE_UNAVAILABLE, "error", "error")
    };
    (
        code,
        Json(Health {
            status,
            version: VERSION,
            db,
        }),
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use axum::body::Body;
    use http_body_util::BodyExt;
    use sqlx::postgres::PgPoolOptions;
    use tower::ServiceExt;

    async fn get(pool: PgPool, uri: &str) -> axum::response::Response {
        router(pool)
            .oneshot(Request::get(uri).body(Body::empty()).unwrap())
            .await
            .unwrap()
    }

    async fn json(respuesta: axum::response::Response) -> serde_json::Value {
        let cuerpo = respuesta.into_body().collect().await.unwrap().to_bytes();
        serde_json::from_slice(&cuerpo).unwrap()
    }

    /// Un pool que apunta a un puerto donde no hay nadie: la base "caída".
    fn pool_sin_base() -> PgPool {
        PgPoolOptions::new()
            .acquire_timeout(std::time::Duration::from_secs(1))
            .connect_lazy("postgres://nadie@127.0.0.1:1/nada")
            .unwrap()
    }

    #[sqlx::test(migrator = "db::MIGRATOR")]
    async fn health_responde_ok_con_la_version_y_la_base(pool: PgPool) {
        let respuesta = get(pool, "/health").await;
        assert_eq!(respuesta.status(), StatusCode::OK);

        let json = json(respuesta).await;
        assert_eq!(json["status"], "ok");
        assert_eq!(json["version"], VERSION);
        assert_eq!(json["db"], "ok");
    }

    #[tokio::test]
    async fn health_responde_503_si_la_base_no_responde() {
        let respuesta = get(pool_sin_base(), "/health").await;
        assert_eq!(respuesta.status(), StatusCode::SERVICE_UNAVAILABLE);

        let json = json(respuesta).await;
        assert_eq!(json["status"], "error");
        assert_eq!(json["version"], VERSION);
        assert_eq!(json["db"], "error");
    }

    #[tokio::test]
    async fn cada_respuesta_lleva_su_id_de_peticion() {
        let respuesta = get(pool_sin_base(), "/health").await;
        let id = respuesta.headers().get("x-request-id");
        assert!(id.is_some_and(|v| !v.is_empty()), "falta x-request-id");
    }
}
