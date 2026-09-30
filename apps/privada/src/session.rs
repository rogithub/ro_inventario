//! La sesión en la web: la cookie, el guardia que pide sesión, entrar y salir.
//! Las reglas (bloqueo, vigencia, contraseñas) viven en `usuarios::sessions`.

use askama::Template;
use axum::Form;
use axum::extract::{Query, Request, State};
use axum::http::{HeaderMap, HeaderValue, StatusCode, header};
use axum::middleware::Next;
use axum::response::{Html, IntoResponse, Redirect, Response};
use serde::Deserialize;
use usuarios::sessions::{LoginError, SESSION_DURATION, current_user, login, logout};

use crate::{AppState, internal_error};

const COOKIE: &str = "session";

/// El valor de una cookie del encabezado `Cookie`.
pub fn read_cookie<'a>(headers: &'a HeaderMap, name: &str) -> Option<&'a str> {
    headers
        .get_all(header::COOKIE)
        .iter()
        .filter_map(|value| value.to_str().ok())
        .flat_map(|value| value.split(';'))
        .filter_map(|pair| pair.trim().split_once('='))
        .find(|(key, _)| *key == name)
        .map(|(_, value)| value.trim())
}

/// `HttpOnly`: JavaScript no la lee. `Secure`: solo viaja por https (o localhost).
/// `SameSite=Lax`: otro sitio no puede mandar formularios con ella.
fn session_cookie(token: &str, max_age_secs: u64) -> String {
    format!("{COOKIE}={token}; HttpOnly; Secure; SameSite=Lax; Path=/; Max-Age={max_age_secs}")
}

/// A dónde ir después de entrar: solo rutas de este sitio. `//otro.com`, `https://…` o una ruta
/// con caracteres raros van a `/`.
pub fn safe_destination(next: Option<&str>) -> &str {
    match next {
        // Sin espacios, controles ni `\`: el navegador los quita o los cambia por `/`, y
        // "/\t/otro.com" terminaría siendo "//otro.com".
        Some(path)
            if path.starts_with('/')
                && !path.starts_with("//")
                && !path
                    .chars()
                    .any(|c| c.is_whitespace() || c.is_control() || c == '\\') =>
        {
            path
        }
        _ => "/",
    }
}

/// Para poner una ruta dentro de `?next=`.
pub fn percent_encode(text: &str) -> String {
    text.bytes()
        .map(|b| match b {
            b'A'..=b'Z' | b'a'..=b'z' | b'0'..=b'9' | b'-' | b'_' | b'.' | b'~' => {
                char::from(b).to_string()
            }
            _ => format!("%{b:02X}"),
        })
        .collect()
}

/// Guardia de las rutas con sesión: deja pasar con el usuario en las extensiones de la petición,
/// o manda a entrar. Cada respuesta renueva la cookie, igual que la sesión se renueva al usarse.
pub async fn require_session(
    State(state): State<AppState>,
    mut request: Request,
    next: Next,
) -> Response {
    let headers = request.headers().clone();
    let cookie = read_cookie(&headers, COOKIE).unwrap_or_default();
    match current_user(&state.usuarios, &state.sessions, cookie).await {
        Ok(Some(usuario)) => {
            request.extensions_mut().insert(usuario);
            let mut response = next.run(request).await;
            // `current_user` ya validó el formato del token: es seguro devolverlo.
            if let Ok(value) =
                HeaderValue::from_str(&session_cookie(cookie, SESSION_DURATION.as_secs()))
            {
                response.headers_mut().append(header::SET_COOKIE, value);
            }
            response
        }
        Ok(None) => to_login(&request),
        Err(error) => internal_error(&headers, &error),
    }
}

/// Una página va a la pantalla de entrar recordando a dónde iba; a htmx se le pide cambiar
/// la página completa (si solo reemplazara un pedazo, el formulario de entrar quedaría adentro).
fn to_login(request: &Request) -> Response {
    if request.headers().contains_key("hx-request") {
        return (StatusCode::UNAUTHORIZED, [("hx-redirect", "/login")]).into_response();
    }
    let path = request
        .uri()
        .path_and_query()
        .map_or("/", |path| path.as_str());
    Redirect::to(&format!("/login?next={}", percent_encode(path))).into_response()
}

#[derive(Template)]
#[template(path = "login.html")]
struct LoginPage<'a> {
    negocio: &'a str,
    usuario_nombre: Option<&'a str>,
    email: &'a str,
    next: &'a str,
    error: Option<String>,
}

#[derive(Deserialize)]
pub struct LoginQuery {
    next: Option<String>,
}

#[derive(Deserialize)]
pub struct LoginForm {
    email: String,
    password: String,
    next: Option<String>,
}

pub async fn login_page(
    State(state): State<AppState>,
    headers: HeaderMap,
    Query(query): Query<LoginQuery>,
) -> Response {
    let next = safe_destination(query.next.as_deref());
    render_login(&state, &headers, "", next, None, StatusCode::OK)
}

fn render_login(
    state: &AppState,
    headers: &HeaderMap,
    email: &str,
    next: &str,
    error: Option<String>,
    status: StatusCode,
) -> Response {
    let page = LoginPage {
        negocio: &state.negocio,
        usuario_nombre: None,
        email,
        next,
        error,
    };
    match page.render() {
        Ok(html) => (status, Html(html)).into_response(),
        Err(error) => internal_error(headers, &error),
    }
}

pub async fn login_submit(
    State(state): State<AppState>,
    headers: HeaderMap,
    Form(form): Form<LoginForm>,
) -> Response {
    let next = safe_destination(form.next.as_deref());
    match login(
        &state.usuarios,
        &state.sessions,
        &form.email,
        &form.password,
    )
    .await
    {
        Ok((usuario, token)) => {
            tracing::info!(usuario = usuario.email.as_str(), "entró");
            let cookie = session_cookie(token.as_str(), SESSION_DURATION.as_secs());
            ([(header::SET_COOKIE, cookie)], Redirect::to(next)).into_response()
        }
        Err(LoginError::Repo(error)) => internal_error(&headers, &error),
        Err(error) => {
            tracing::warn!(%error, "no pudo entrar");
            render_login(
                &state,
                &headers,
                form.email.trim(),
                next,
                Some(error.to_string()),
                StatusCode::UNPROCESSABLE_ENTITY,
            )
        }
    }
}

/// Siempre termina en la pantalla de entrar, aunque la sesión ya no existiera.
pub async fn logout_submit(State(state): State<AppState>, headers: HeaderMap) -> Response {
    let cookie = read_cookie(&headers, COOKIE).unwrap_or_default();
    if let Err(error) = logout(&state.sessions, cookie).await {
        tracing::error!(%error, "no se pudo borrar la sesión al salir");
    }
    (
        [(header::SET_COOKIE, session_cookie("", 0))],
        Redirect::to("/login"),
    )
        .into_response()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test_support::*;
    use axum::body::Body;
    use axum::http::Request;
    use db::PgPool;

    // --- piezas puras ---

    #[test]
    fn lee_la_cookie_entre_varias() {
        let mut headers = HeaderMap::new();
        headers.insert(
            header::COOKIE,
            HeaderValue::from_static("tema=claro; session=abc123 ; x=1"),
        );
        assert_eq!(read_cookie(&headers, "session"), Some("abc123"));
        assert_eq!(read_cookie(&headers, "tema"), Some("claro"));
        assert_eq!(read_cookie(&headers, "otra"), None);
        assert_eq!(read_cookie(&HeaderMap::new(), "session"), None);
    }

    #[test]
    fn la_cookie_lleva_sus_banderas() {
        assert_eq!(
            session_cookie("abc", 604800),
            "session=abc; HttpOnly; Secure; SameSite=Lax; Path=/; Max-Age=604800"
        );
    }

    #[test]
    fn solo_se_regresa_a_rutas_de_este_sitio() {
        assert_eq!(safe_destination(Some("/unidades?x=1")), "/unidades?x=1");
        assert_eq!(safe_destination(None), "/");
        assert_eq!(safe_destination(Some("")), "/");
        assert_eq!(safe_destination(Some("//otro.com")), "/");
        assert_eq!(safe_destination(Some("/\\otro.com")), "/");
        assert_eq!(safe_destination(Some("https://otro.com")), "/");
        assert_eq!(safe_destination(Some("unidades")), "/");
        // El navegador quita tabuladores y saltos de línea de una URL: "/\t/otro.com" es "//otro.com".
        assert_eq!(safe_destination(Some("/\t/otro.com")), "/");
        assert_eq!(safe_destination(Some("/\n/otro.com")), "/");
        assert_eq!(safe_destination(Some("/\r/otro.com")), "/");
        assert_eq!(safe_destination(Some("/ /otro.com")), "/");
        assert_eq!(safe_destination(Some("/a\\b")), "/");
    }

    #[test]
    fn codifica_la_ruta_para_la_url() {
        assert_eq!(
            percent_encode("/unidades?x=1&y=ñ"),
            "%2Funidades%3Fx%3D1%26y%3D%C3%B1"
        );
        assert_eq!(percent_encode("abc-_.~"), "abc-_.~");
    }

    // --- sin sesión ---

    #[sqlx::test(migrator = "db::MIGRATOR")]
    async fn sin_sesion_una_pagina_lleva_a_entrar_con_la_ruta(pool: PgPool) {
        let response = get(pool, "/unidades").await;
        assert_eq!(response.status(), StatusCode::SEE_OTHER);
        assert_eq!(
            response.headers()[header::LOCATION],
            "/login?next=%2Funidades"
        );
    }

    #[sqlx::test(migrator = "db::MIGRATOR")]
    async fn sin_sesion_htmx_recibe_la_orden_de_ir_a_entrar(pool: PgPool) {
        let request = Request::get("/unidades")
            .header("hx-request", "true")
            .body(Body::empty())
            .unwrap();
        let response = send(pool, request).await;
        assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
        assert_eq!(response.headers()["hx-redirect"], "/login");
    }

    #[sqlx::test(migrator = "db::MIGRATOR")]
    async fn una_cookie_que_no_es_de_nadie_tambien_lleva_a_entrar(pool: PgPool) {
        let response =
            get_with_cookie(pool, "/unidades", &format!("session={}", "a".repeat(64))).await;
        assert_eq!(response.status(), StatusCode::SEE_OTHER);
    }

    #[tokio::test]
    async fn si_la_base_falla_al_revisar_la_sesion_se_ve_el_id_para_reportarlo() {
        let cookie = format!("session={}", "a".repeat(64));
        let response = get_with_cookie(unreachable_pool(), "/unidades", &cookie).await;
        assert_eq!(response.status(), StatusCode::INTERNAL_SERVER_ERROR);

        let id = response.headers()["x-request-id"]
            .to_str()
            .unwrap()
            .to_string();
        assert!(body_text(response).await.contains(&id));
    }

    #[sqlx::test(migrator = "db::MIGRATOR")]
    async fn la_pantalla_de_entrar_no_pide_sesion(pool: PgPool) {
        let response = get(pool, "/login?next=%2Funidades").await;
        assert_eq!(response.status(), StatusCode::OK);
        let html = body_text(response).await;
        assert!(html.contains(r#"<form method="post" action="/login">"#));
        assert!(html.contains(r#"name="next" value="/unidades""#));
        assert!(!html.contains("Salir"), "sin sesión no hay botón de salir");
    }

    // --- entrar ---

    fn post_login(form: &str) -> Request<Body> {
        Request::post("/login")
            .header(header::CONTENT_TYPE, "application/x-www-form-urlencoded")
            .body(Body::from(form.to_string()))
            .unwrap()
    }

    #[sqlx::test(migrator = "db::MIGRATOR")]
    async fn entrar_pone_la_cookie_y_lleva_a_donde_iba(pool: PgPool) {
        cookie(&pool, "ana@x.mx", "Cajero").await;

        let response = send(
            pool,
            post_login("email=ana%40x.mx&password=caja-de-lapices&next=%2Funidades"),
        )
        .await;

        assert_eq!(response.status(), StatusCode::SEE_OTHER);
        assert_eq!(response.headers()[header::LOCATION], "/unidades");
        let cookie = response.headers()[header::SET_COOKIE].to_str().unwrap();
        assert!(cookie.starts_with("session="));
        assert!(cookie.ends_with("; HttpOnly; Secure; SameSite=Lax; Path=/; Max-Age=604800"));
    }

    #[sqlx::test(migrator = "db::MIGRATOR")]
    async fn entrar_no_lleva_fuera_del_sitio(pool: PgPool) {
        cookie(&pool, "ana@x.mx", "Cajero").await;

        let response = send(
            pool,
            post_login("email=ana%40x.mx&password=caja-de-lapices&next=%2F%2Fotro.com"),
        )
        .await;

        assert_eq!(response.headers()[header::LOCATION], "/");
    }

    #[sqlx::test(migrator = "db::MIGRATOR")]
    async fn una_contrasena_mala_dice_por_que_y_conserva_el_email(pool: PgPool) {
        cookie(&pool, "ana@x.mx", "Cajero").await;

        let response = send(pool, post_login("email=ana%40x.mx&password=otra-cosa-1234")).await;

        assert_eq!(response.status(), StatusCode::UNPROCESSABLE_ENTITY);
        assert!(response.headers().get(header::SET_COOKIE).is_none());
        let html = body_text(response).await;
        assert!(html.contains("Email o contraseña incorrectos."));
        assert!(html.contains(r#"value="ana@x.mx""#));
    }

    #[sqlx::test(migrator = "db::MIGRATOR")]
    async fn al_sexto_intento_dice_que_espere(pool: PgPool) {
        for _ in 0..5 {
            send(
                pool.clone(),
                post_login("email=nadie%40x.mx&password=otra-cosa-1234"),
            )
            .await;
        }
        let response = send(
            pool,
            post_login("email=nadie%40x.mx&password=otra-cosa-1234"),
        )
        .await;
        assert!(body_text(response).await.contains("Demasiados intentos."));
    }

    // --- con sesión ---

    #[sqlx::test(migrator = "db::MIGRATOR")]
    async fn cada_respuesta_con_sesion_renueva_la_cookie(pool: PgPool) {
        let cookie = cookie(&pool, "ana@x.mx", "Cajero").await;

        let response = get_with_cookie(pool, "/unidades", &cookie).await;

        assert_eq!(response.status(), StatusCode::OK);
        let renewed = response.headers()[header::SET_COOKIE].to_str().unwrap();
        assert!(renewed.starts_with(&format!("{cookie};")));
        assert!(renewed.ends_with("Max-Age=604800"));
    }

    #[sqlx::test(migrator = "db::MIGRATOR")]
    async fn la_barra_muestra_quien_esta_y_como_salir(pool: PgPool) {
        let cookie = cookie(&pool, "ana@x.mx", "Cajero").await;
        let html = body_text(get_with_cookie(pool, "/unidades", &cookie).await).await;
        assert!(html.contains("Ana López"));
        assert!(html.contains(r#"action="/logout""#));
    }

    #[sqlx::test(migrator = "db::MIGRATOR")]
    async fn la_raiz_lleva_a_unidades(pool: PgPool) {
        let cookie = cookie(&pool, "ana@x.mx", "Cajero").await;
        let response = get_with_cookie(pool, "/", &cookie).await;
        assert_eq!(response.headers()[header::LOCATION], "/unidades");
    }

    #[sqlx::test(migrator = "db::MIGRATOR")]
    async fn salir_cierra_la_sesion_y_borra_la_cookie(pool: PgPool) {
        let cookie = cookie(&pool, "ana@x.mx", "Cajero").await;
        let request = Request::post("/logout")
            .header("cookie", &cookie)
            .body(Body::empty())
            .unwrap();

        let response = send(pool.clone(), request).await;

        assert_eq!(response.status(), StatusCode::SEE_OTHER);
        assert_eq!(response.headers()[header::LOCATION], "/login");
        assert!(
            response.headers()[header::SET_COOKIE]
                .to_str()
                .unwrap()
                .ends_with("Max-Age=0")
        );
        let after = get_with_cookie(pool, "/unidades", &cookie).await;
        assert_eq!(
            after.status(),
            StatusCode::SEE_OTHER,
            "la cookie vieja ya no sirve"
        );
    }
}
