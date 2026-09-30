//! Aplicación privada (punto de venta): arma el router y sus piezas. Sin reglas del negocio.

pub mod comandos;
mod estaticos;
mod health;
mod session;
mod unidades;

use std::sync::Arc;

use axum::Router;
use axum::http::{HeaderMap, Request, StatusCode};
use axum::middleware;
use axum::response::{Html, IntoResponse, Redirect, Response};
use axum::routing::{get, post};
use db::{PgPool, PgSessions, PgUnidadesMedida, PgUsuarios};
use tower_http::request_id::{MakeRequestUuid, PropagateRequestIdLayer, SetRequestIdLayer};
use tower_http::trace::{DefaultOnResponse, TraceLayer};
use tracing::Level;

/// Versión de esta compilación; aparece en `/health` y en el log de arranque.
pub const VERSION: &str = env!("CARGO_PKG_VERSION");

/// Lo que comparten las rutas: la base, los repositorios y los datos del negocio para las pantallas.
#[derive(Clone)]
pub struct AppState {
    pool: PgPool,
    unidades: PgUnidadesMedida,
    usuarios: PgUsuarios,
    sessions: PgSessions,
    /// Nombre del negocio (negocio.toml), para el título y la barra de las pantallas.
    negocio: Arc<str>,
}

impl AppState {
    pub fn new(pool: PgPool, negocio: &str) -> Self {
        Self {
            unidades: PgUnidadesMedida::new(pool.clone()),
            usuarios: PgUsuarios::new(pool.clone()),
            sessions: PgSessions::new(pool.clone()),
            pool,
            negocio: negocio.into(),
        }
    }
}

/// Todas las rutas de la aplicación, con el id de petición y el log por petición.
/// Todo pide sesión, salvo lo que se necesita antes de entrar.
pub fn router(state: AppState) -> Router {
    let con_sesion = Router::new()
        // Mientras no haya página de inicio.
        .route("/", get(|| async { Redirect::to("/unidades") }))
        .route("/unidades", get(unidades::page).post(unidades::add))
        .route_layer(middleware::from_fn_with_state(
            state.clone(),
            session::require_session,
        ));
    Router::new()
        .merge(con_sesion)
        .route("/health", get(health::health))
        .route("/static/{*ruta}", get(estaticos::archivo))
        .route(
            "/login",
            get(session::login_page).post(session::login_submit),
        )
        .route("/logout", post(session::logout_submit))
        .with_state(state)
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

/// Algo falló que no es culpa del usuario: el detalle va al log (con el id de la petición, por el
/// span) y la pantalla solo muestra el id para reportarlo.
fn falla_interna(headers: &HeaderMap, error: &dyn std::fmt::Display) -> Response {
    tracing::error!(%error, "falla interna");
    let id = headers
        .get("x-request-id")
        .and_then(|v| v.to_str().ok())
        .unwrap_or("-");
    (
        StatusCode::INTERNAL_SERVER_ERROR,
        Html(format!(
            r#"<div class="alert alert-danger">Algo falló. Repórtalo con el error <code>{id}</code>.</div>"#
        )),
    )
        .into_response()
}

#[cfg(test)]
mod test_support {
    use super::*;
    use axum::body::Body;
    use http_body_util::BodyExt;
    use sqlx::postgres::PgPoolOptions;
    use tower::ServiceExt;

    pub fn state(pool: PgPool) -> AppState {
        AppState::new(pool, "Papelería de prueba")
    }

    /// Un pool que apunta a un puerto donde no hay nadie: la base "caída".
    pub fn pool_sin_base() -> PgPool {
        PgPoolOptions::new()
            .acquire_timeout(std::time::Duration::from_secs(1))
            .connect_lazy("postgres://nadie@127.0.0.1:1/nada")
            .unwrap()
    }

    pub async fn send(pool: PgPool, request: Request<Body>) -> Response {
        router(state(pool)).oneshot(request).await.unwrap()
    }

    pub async fn get(pool: PgPool, uri: &str) -> Response {
        send(pool, Request::get(uri).body(Body::empty()).unwrap()).await
    }

    pub const CONTRASENA: &str = "caja-de-lapices";

    /// Crea un usuario con ese rol, entra y regresa el encabezado Cookie de su sesión.
    pub async fn cookie(pool: &PgPool, email: &str, rol: &str) -> String {
        use usuarios::usuarios::{NuevoUsuario, UsuariosRepo};
        let usuarios = PgUsuarios::new(pool.clone());
        usuarios
            .add(NuevoUsuario::new(email, "Ana López", rol, CONTRASENA).unwrap())
            .await
            .unwrap();
        let (_, token) =
            usuarios::sessions::login(&usuarios, &PgSessions::new(pool.clone()), email, CONTRASENA)
                .await
                .unwrap();
        format!("sesion={}", token.as_str())
    }

    /// Un GET con la sesión de esa cookie.
    pub async fn get_con(pool: PgPool, uri: &str, cookie: &str) -> Response {
        let request = Request::get(uri)
            .header("cookie", cookie)
            .body(Body::empty())
            .unwrap();
        send(pool, request).await
    }

    pub async fn body_text(respuesta: Response) -> String {
        let bytes = respuesta.into_body().collect().await.unwrap().to_bytes();
        String::from_utf8(bytes.to_vec()).unwrap()
    }
}
