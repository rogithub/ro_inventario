//! CSS, JavaScript y fuentes de terceros, dentro del binario (static/LEEME.md).

use axum::extract::Path;
use axum::http::{StatusCode, header};
use axum::response::{IntoResponse, Response};

/// Ruta bajo /static/, tipo y contenido. Solo estos: ninguna ruta llega al disco.
const ARCHIVOS: &[(&str, &str, &[u8])] = &[
    (
        "bootstrap.min.css",
        "text/css",
        include_bytes!("../static/bootstrap.min.css"),
    ),
    (
        "bootstrap.bundle.min.js",
        "text/javascript",
        include_bytes!("../static/bootstrap.bundle.min.js"),
    ),
    (
        "bootstrap-icons.min.css",
        "text/css",
        include_bytes!("../static/bootstrap-icons.min.css"),
    ),
    (
        "fonts/bootstrap-icons.woff2",
        "font/woff2",
        include_bytes!("../static/fonts/bootstrap-icons.woff2"),
    ),
    (
        "htmx.min.js",
        "text/javascript",
        include_bytes!("../static/htmx.min.js"),
    ),
];

pub async fn archivo(Path(ruta): Path<String>) -> Response {
    match ARCHIVOS.iter().find(|(nombre, _, _)| *nombre == ruta) {
        Some((_, tipo, contenido)) => (
            [
                (header::CONTENT_TYPE, *tipo),
                // Un día: cambian solo al actualizar una versión, y eso llega con un deploy.
                (header::CACHE_CONTROL, "public, max-age=86400"),
            ],
            *contenido,
        )
            .into_response(),
        None => StatusCode::NOT_FOUND.into_response(),
    }
}

#[cfg(test)]
mod tests {
    use crate::test_support::*;
    use axum::http::{StatusCode, header};

    #[tokio::test]
    async fn los_archivos_estaticos_se_sirven_con_su_tipo() {
        let respuesta = get(pool_sin_base(), "/static/htmx.min.js").await;
        assert_eq!(respuesta.status(), StatusCode::OK);
        assert_eq!(respuesta.headers()[header::CONTENT_TYPE], "text/javascript");

        let respuesta = get(pool_sin_base(), "/static/fonts/bootstrap-icons.woff2").await;
        assert_eq!(respuesta.status(), StatusCode::OK);
        assert_eq!(respuesta.headers()[header::CONTENT_TYPE], "font/woff2");
    }

    #[tokio::test]
    async fn un_archivo_que_no_existe_da_404() {
        let respuesta = get(pool_sin_base(), "/static/../Cargo.toml").await;
        assert_eq!(respuesta.status(), StatusCode::NOT_FOUND);
    }
}
