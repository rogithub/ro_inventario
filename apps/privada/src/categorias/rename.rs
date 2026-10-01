//! Renombrar una categoría desde su renglón de la lista.

use axum::extract::{Path, State};
use axum::http::{HeaderMap, StatusCode};
use axum::response::{Html, IntoResponse, Redirect, Response};
use axum::{Extension, Form};
use inventario::categorias::{CategoriaError, CategoriaId, CategoriasRepo, NewCategoria};
use kernel::Uuid;
use usuarios::permisos::Permiso;
use usuarios::usuarios::Usuario;

use super::{CategoriaForm, Editing, show};
use crate::{AppState, internal_error, is_htmx};

pub async fn rename(
    State(state): State<AppState>,
    Extension(usuario): Extension<Usuario>,
    headers: HeaderMap,
    Path(id): Path<String>,
    Form(form): Form<CategoriaForm>,
) -> Response {
    if !usuario.can(Permiso::EditarCatalogo) {
        return (
            StatusCode::FORBIDDEN,
            Html(
                r#"<div class="alert alert-danger">No tienes permiso para renombrar categorías.</div>"#,
            ),
        )
            .into_response();
    }
    // Un id que no es uuid no es de ninguna categoría: se trata igual que uno que ya no existe.
    let id = CategoriaId::parse(&id).unwrap_or(CategoriaId(Uuid::nil()));
    let result = match NewCategoria::new(&form.nombre) {
        Ok(new_categoria) => state.categorias.rename(id, new_categoria).await.map(|_| ()),
        Err(error) => Err(error),
    };
    match result {
        // Sin JavaScript: de vuelta a la lista, para que recargar no reenvíe el formulario.
        Ok(()) if !is_htmx(&headers) => Redirect::to("/categorias").into_response(),
        Ok(()) => show(&state, &usuario, &headers, "", None, None).await,
        Err(CategoriaError::Repo(error)) => internal_error(&headers, &error),
        Err(error) => {
            let editing = Editing {
                id,
                nombre: form.nombre,
                error: Some(error.to_string()),
            };
            show(&state, &usuario, &headers, "", None, Some(editing)).await
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::test_support::*;
    use axum::body::Body;
    use axum::http::{Request, StatusCode, header};
    use db::{PgCategorias, PgPool};
    use inventario::categorias::{Categoria, CategoriasRepo, NewCategoria};

    async fn add(pool: &PgPool, nombre: &str) -> Categoria {
        PgCategorias::new(pool.clone())
            .add(NewCategoria::new(nombre).unwrap())
            .await
            .unwrap()
    }

    fn post(uri: &str, cookie: &str, form: &str, htmx: bool) -> Request<Body> {
        let mut request = Request::post(uri)
            .header("cookie", cookie)
            .header(header::CONTENT_TYPE, "application/x-www-form-urlencoded");
        if htmx {
            request = request.header("hx-request", "true");
        }
        request.body(Body::from(form.to_string())).unwrap()
    }

    #[sqlx::test(migrator = "db::MIGRATOR")]
    async fn el_lapiz_muestra_el_renglon_listo_para_renombrar(pool: PgPool) {
        let cookie = cookie(&pool, "ana@x.mx", "Dueño").await;
        let plumas = add(&pool, "Plumas").await;

        let uri = format!("/categorias?edit={}", plumas.id);
        let html = body_text(get_with_cookie(pool, &uri, &cookie).await).await;

        assert!(html.contains(&format!(r#"action="/categorias/{}""#, plumas.id)));
        assert!(html.contains(r#"value="Plumas""#));
    }

    #[sqlx::test(migrator = "db::MIGRATOR")]
    async fn renombrar_con_htmx_regresa_la_seccion_con_el_nombre_nuevo(pool: PgPool) {
        let cookie = cookie(&pool, "ana@x.mx", "Dueño").await;
        let plumas = add(&pool, "Plumas").await;
        let uri = format!("/categorias/{}", plumas.id);

        let response = send(pool, post(&uri, &cookie, "nombre=Bol%C3%ADgrafos", true)).await;

        assert_eq!(response.status(), StatusCode::OK);
        let html = body_text(response).await;
        assert!(html.contains("<td>Bolígrafos</td>"));
        assert!(!html.contains("Plumas"));
        assert!(!html.contains("<html"), "debe ser solo la sección");
    }

    #[sqlx::test(migrator = "db::MIGRATOR")]
    async fn renombrar_sin_htmx_regresa_a_la_lista(pool: PgPool) {
        let cookie = cookie(&pool, "ana@x.mx", "Dueño").await;
        let plumas = add(&pool, "Plumas").await;
        let uri = format!("/categorias/{}", plumas.id);

        let response = send(pool, post(&uri, &cookie, "nombre=Lapiceros", false)).await;

        assert_eq!(response.status(), StatusCode::SEE_OTHER);
        assert_eq!(response.headers()[header::LOCATION], "/categorias");
    }

    #[sqlx::test(migrator = "db::MIGRATOR")]
    async fn renombrar_a_uno_que_ya_existe_regresa_422_en_el_renglon(pool: PgPool) {
        let cookie = cookie(&pool, "ana@x.mx", "Dueño").await;
        add(&pool, "Plumas").await;
        let hojas = add(&pool, "Hojas").await;
        let uri = format!("/categorias/{}", hojas.id);

        let response = send(pool, post(&uri, &cookie, "nombre=plumas", true)).await;

        assert_eq!(response.status(), StatusCode::UNPROCESSABLE_ENTITY);
        let html = body_text(response).await;
        assert!(html.contains("Ya existe la categoría «plumas»."));
        assert!(html.contains(r#"value="plumas""#), "conserva lo escrito");
        assert!(html.contains(&format!(r#"action="/categorias/{}""#, hojas.id)));
    }

    #[sqlx::test(migrator = "db::MIGRATOR")]
    async fn renombrar_una_que_ya_no_existe_avisa(pool: PgPool) {
        let cookie = cookie(&pool, "ana@x.mx", "Dueño").await;
        let uri = "/categorias/00000000-0000-0000-0000-000000000099";

        let response = send(pool, post(uri, &cookie, "nombre=Plumas", true)).await;

        assert_eq!(response.status(), StatusCode::UNPROCESSABLE_ENTITY);
        assert!(
            body_text(response)
                .await
                .contains("Esa categoría ya no existe. Recarga la página.")
        );
    }

    #[sqlx::test(migrator = "db::MIGRATOR")]
    async fn un_cajero_no_ve_el_lapiz(pool: PgPool) {
        let plumas = add(&pool, "Plumas").await;
        let cookie = cookie(&pool, "beto@x.mx", "Cajero").await;

        let uri = format!("/categorias?edit={}", plumas.id);
        let html = body_text(get_with_cookie(pool, &uri, &cookie).await).await;

        assert!(html.contains("<td>Plumas</td>"));
        assert!(!html.contains("?edit="));
        assert!(!html.contains(&format!(r#"action="/categorias/{}""#, plumas.id)));
    }

    #[sqlx::test(migrator = "db::MIGRATOR")]
    async fn un_cajero_no_puede_renombrar_aunque_mande_el_formulario(pool: PgPool) {
        let plumas = add(&pool, "Plumas").await;
        let cookie = cookie(&pool, "beto@x.mx", "Cajero").await;
        let uri = format!("/categorias/{}", plumas.id);

        let response = send(pool.clone(), post(&uri, &cookie, "nombre=Hackeada", true)).await;

        assert_eq!(response.status(), StatusCode::FORBIDDEN);
        let html = body_text(get_with_cookie(pool, "/categorias", &cookie).await).await;
        assert!(html.contains("<td>Plumas</td>"));
    }
}
