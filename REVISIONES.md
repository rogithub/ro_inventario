# Revisiones

---

## Fase 1 — Auth completo

**Archivos a mirar:**
- `src/auth/backend.rs` — `User`, `AuthBackend`, verificación HMAC-SHA512
- `src/auth/routes.rs` — handlers de login/logout
- `src/templates.rs` — helper `render()` para convertir templates Askama a Response
- `templates/auth/login.html` — formulario de login (sin navbar, sin sesión)
- `src/main.rs` — setup de sesiones PostgreSQL y capas de auth

**Qué hace:**
`POST /login` verifica el email y password contra la tabla `Users`. Si las credenciales son válidas, axum-login escribe el `User` serializado en la tabla `tower_sessions` de PostgreSQL y setea una cookie de sesión. Todos los requests subsecuentes leen esa cookie, cargan el usuario de la sesión y lo inyectan como extractor `AuthSession` en los handlers. Rutas sin sesión activa se redirigen a `/login` automáticamente.

**Notas de dominio:**
- El .NET usa HMAC-SHA512: `PasswordSalt` es la key del HMAC, `PasswordHash` es `HMAC(password_utf8)`. La función `verify_hmac_sha512` en `backend.rs` replica exactamente `VerifyPasswordHash` del .NET — los usuarios existentes pueden hacer login sin cambiar su password.
- `isactive` no tiene `NOT NULL` en el esquema, así que sqlx lo infiere como `Option<bool>`. Se trata con `.unwrap_or(false)`.

**Notas de Rust:**
- `AuthSession` es un extractor de Axum (igual que `State<T>` o `Form<T>`). Axum lo inyecta automáticamente en los handlers que lo declaren como parámetro — no hay que buscarlo manualmente.
- El `with_secure(false)` en la cookie de sesión es correcto para desarrollo local (HTTP). En producción, el TLS termina en el proxy de k3s antes de llegar al contenedor, así que el flag puede seguir en `false` (el cookie nunca viaja en claro fuera del cluster).
- `templates::render()` — en vez de depender del crate `askama_axum` (que tiene versiones frágiles), implementamos el helper nosotros: llama a `t.render()` de Askama y envuelve en `Html(...)`. Se aplica a cualquier struct que derive `Template`.

---

Entradas más recientes arriba. Qué archivos mirar y qué hace cada cambio.

---

## Infraestructura base — servidor mínimo corriendo

**Archivos a mirar:**
- `Cargo.toml` — todas las dependencias del proyecto
- `src/main.rs` — startup: carga config, conecta a la BD, monta el router
- `src/config.rs` — lee variables de entorno con valores razonables por defecto
- `src/error.rs` — `AppError` centralizado que implementa `IntoResponse`
- `templates/base.html` — layout con Bootstrap 5, htmx y Alpine.js desde CDN
- `Containerfile` — build multi-stage: cross-compila en amd64, corre en arm64
- `.github/workflows/build.yaml` — CI: push a `main` → build ARM64 → push a `ghcr.io`

**Qué hace:**
El servidor arranca, conecta a PostgreSQL y responde. `GET /` redirige a `/ventas` (placeholder — la ruta no existe todavía, devuelve 404). Los assets estáticos se sirven desde `/static/`.

**Notas de dominio / Rust:**
- `AppState` contiene `PgPool` y `Config`. `PgPool` es internamente un `Arc` (pool compartido entre todos los requests concurrentes) — clonarlo es barato, no crea una conexión nueva.
- `TraceLayer` de tower-http loguea automáticamente cada request con método, path, status y latencia. Los logs se ven en stdout gracias a `tracing_subscriber::fmt::init()`.
- `dotenvy::dotenv().ok()` — el `.ok()` descarta el error si no existe `.env`; en producción las vars vienen del entorno del contenedor directamente.
- `AppError` usa `thiserror` para derivar `Display` y `From` automáticamente. Las variantes `From<sqlx::Error>` y `From<anyhow::Error>` permiten usar `?` en los handlers sin conversión manual.
- El feature `rustls` en reqwest evita la dependencia de OpenSSL — importante para builds ARM64 reproducibles.
