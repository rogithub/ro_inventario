use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
};

// Los handlers devuelven Result<impl IntoResponse, AppError>.
// anyhow::Error cubre errores inesperados internos;
// thiserror da variantes tipadas para errores de dominio.
#[derive(Debug, thiserror::Error)]
pub enum AppError {
    #[error("Error de base de datos: {0}")]
    Db(#[from] sqlx::Error),
    #[error("Error interno: {0}")]
    Internal(#[from] anyhow::Error),
    #[error("No encontrado")]
    NotFound,
}

impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        let status = match &self {
            AppError::NotFound => StatusCode::NOT_FOUND,
            _ => StatusCode::INTERNAL_SERVER_ERROR,
        };
        tracing::error!("{}", self);
        (status, self.to_string()).into_response()
    }
}
