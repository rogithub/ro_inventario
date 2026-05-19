use askama::Template;
use axum::{
    Form,
    response::{IntoResponse, Redirect, Response},
};

use super::backend::{AuthSession, Credentials};
use crate::templates;

// --- Templates ---

#[derive(Template)]
#[template(path = "auth/login.html")]
struct LoginTemplate {
    error: Option<String>,
}

// --- Handlers ---

pub async fn login_get(auth: AuthSession) -> Response {
    if auth.user.is_some() {
        return Redirect::to("/ventas").into_response();
    }
    templates::render(LoginTemplate { error: None })
}

pub async fn login_post(
    mut auth: AuthSession,
    Form(creds): Form<Credentials>,
) -> Response {
    match auth.authenticate(creds).await {
        Ok(Some(user)) => {
            auth.login(&user).await.unwrap();
            Redirect::to("/ventas").into_response()
        }
        Ok(None) => templates::render(LoginTemplate {
            error: Some("Correo o contraseña incorrectos.".into()),
        }),
        Err(e) => {
            tracing::error!("Error en autenticación: {e}");
            templates::render(LoginTemplate {
                error: Some("Error interno. Intenta de nuevo.".into()),
            })
        }
    }
}

pub async fn logout(mut auth: AuthSession) -> impl IntoResponse {
    auth.logout().await.unwrap();
    Redirect::to("/login")
}
