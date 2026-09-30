//! Aplicación privada (punto de venta): arma el router y sus piezas. Sin reglas del negocio.

use axum::http::Request;
use axum::routing::get;
use axum::{Json, Router};
use serde::Serialize;
use tower_http::request_id::{MakeRequestUuid, PropagateRequestIdLayer, SetRequestIdLayer};
use tower_http::trace::{DefaultOnResponse, TraceLayer};
use tracing::Level;

/// Versión de esta compilación; aparece en `/health` y en el log de arranque.
pub const VERSION: &str = env!("CARGO_PKG_VERSION");

/// Todas las rutas de la aplicación, con el id de petición y el log por petición.
pub fn router() -> Router {
    Router::new().route("/health", get(health)).layer(
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
}

/// Para las sondas de Kubernetes. El estado de la base se agrega cuando haya base.
async fn health() -> Json<Health> {
    Json(Health {
        status: "ok",
        version: VERSION,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use axum::body::Body;
    use axum::http::{Request, StatusCode};
    use http_body_util::BodyExt;
    use tower::ServiceExt;

    async fn get(uri: &str) -> axum::response::Response {
        router()
            .oneshot(Request::get(uri).body(Body::empty()).unwrap())
            .await
            .unwrap()
    }

    #[tokio::test]
    async fn health_responde_ok_con_la_version() {
        let respuesta = get("/health").await;
        assert_eq!(respuesta.status(), StatusCode::OK);

        let cuerpo = respuesta.into_body().collect().await.unwrap().to_bytes();
        let json: serde_json::Value = serde_json::from_slice(&cuerpo).unwrap();
        assert_eq!(json["status"], "ok");
        assert_eq!(json["version"], VERSION);
    }

    #[tokio::test]
    async fn cada_respuesta_lleva_su_id_de_peticion() {
        let respuesta = get("/health").await;
        let id = respuesta.headers().get("x-request-id");
        assert!(id.is_some_and(|v| !v.is_empty()), "falta x-request-id");
    }
}
