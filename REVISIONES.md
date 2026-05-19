# Revisiones

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
