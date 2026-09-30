use axum::Json;
use axum::extract::State;
use axum::http::StatusCode;
use serde::Serialize;

use crate::{AppState, VERSION};

#[derive(Serialize)]
pub struct Health {
    status: &'static str,
    version: &'static str,
    db: &'static str,
}

/// Para las sondas de Kubernetes: 503 si la base no responde.
pub async fn health(State(state): State<AppState>) -> (StatusCode, Json<Health>) {
    let (code, status, db) = if db::is_db_alive(&state.pool).await {
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
    use crate::VERSION;
    use crate::test_support::*;
    use axum::http::StatusCode;
    use db::PgPool;

    async fn json(respuesta: axum::response::Response) -> serde_json::Value {
        serde_json::from_str(&body_text(respuesta).await).unwrap()
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
