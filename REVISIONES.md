# Revisiones

---

## Paso 2 — POST /ventas (transacción de venta)

**Archivos a mirar:**
- `src/modules/ventas/models.rs` — structs `LineaPayload` y `NuevaVentaPayload`
- `src/modules/ventas/queries.rs` — función `crear_venta()` + struct privado `FilaMonedero`
- `src/modules/ventas/routes.rs` — handler `crear()`
- `src/main.rs` — ruta `/ventas` ahora acepta GET y POST

**Qué hace:**
`POST /ventas` recibe el carrito como JSON y lo registra en una sola transacción sqlx con cuatro pasos:

1. **INSERT ajustes** — encabezado de la venta. `tipoajuste = 0` (venta), `ivaventa` desde `Settings`.
2. **INSERT ajustesproductos** — una fila por cada línea del carrito.
3. **INSERT monederogenerados** — solo si hay cliente registrado (`cliente_id IS NOT NULL`) y la línea no es ingreso trasladado. `dinerodigital = cantidad × precio × tipo_cambio_monedero`; `fechaexpiracion = ahora + dias_vigencia_monedero`.
4. **INSERT monederoredimidos** — solo si `pago_monedero > 0`. Consulta `v_ajuste_producto_monedero` para obtener las filas de balance ordenadas por antigüedad (`monederogeneradosid ASC`) y las consume de menor a mayor hasta cubrir el monto.

Devuelve `{ "id": "<uuid-de-la-venta>" }`.

**Notas de dominio:**
- En el .NET esto era 3 transacciones separadas (`AjustesRepo.Save` → `AjustesProductosRepo.BulkSave` → `MonederoRepo.PagarConMonederos`). Aquí todo va en una sola — si falla cualquier paso, todo se revierte.
- `MonederoGenerados.Id` es `SERIAL` (entero), no UUID. `FilaMonedero.monederogeneradosid: i32` refleja eso.
- Los ingresos trasladados se excluyen del monedero: antes de la transacción se carga `v_ingresos_trasladados` como `HashSet<Uuid>` para hacer la verificación en O(1) por línea.
- La subconsulta de redención: `balancedinero` en la vista es una suma acumulativa por cliente ordenada por `g.FechaCreado, g.Id`. La subconsulta encuentra el primer registro donde el acumulado cubre el pago; el outer query trae todos los registros hasta ese punto.

**Notas de Rust:**
- Las queries nuevas usan `sqlx::query(...)` y `sqlx::query_as` **sin `!`** — no necesitan entrada en el caché `.sqlx/` ni `cargo sqlx prepare`. El tradeoff es que el SQL no se verifica en compile time (solo en runtime contra la BD).
- `sqlx::query_scalar::<_, Uuid>(sql)` — versión sin `!` para la query de ingresos trasladados.
- `&mut *tx` al pasar la transacción como executor — sqlx necesita `&mut <impl Executor>` y `Transaction<Postgres>` implementa `Deref<Target=PgConnection>`, así que `&mut *tx` lo desreferencia correctamente.
- `auth: crate::auth::AuthSession` como extractor en el handler — funciona porque el middleware `login_required!` ya garantiza que `auth.user` es `Some`.

---

## Paso 1 — Endpoints JSON para nueva venta

**Archivos a mirar:**
- `src/modules/productos/` — módulo nuevo: `models.rs`, `queries.rs`, `routes.rs`
- `src/modules/clientes/` — módulo nuevo: `models.rs`, `queries.rs`, `routes.rs`
- `src/modules/ventas/routes.rs` — agregados `tipo_cambio` y `monedero`
- `src/modules/ventas/queries.rs` — agregada `balance_monedero()`
- `src/db/settings.rs` — agregado campo `banxico_api_token`
- `src/main.rs` — 4 rutas nuevas bajo `/api/`

**Qué hace:**
Cuatro endpoints JSON protegidos por autenticación, consumidos por el formulario de nueva venta:
- `GET /api/productos/buscar?q=&stock=` — búsqueda en cascada: entero→NID, UUID→ID propio (QR), texto exacto→código de barras, FTS+ILIKE, trigramas. `stock=true` filtra a productos con existencia.
- `GET /api/clientes/buscar?q=` — detecta si el input es teléfono (≥70% dígitos) y normaliza a los últimos 10; si no, busca por nombre/empresa/email con ILIKE y `unaccent`.
- `GET /api/tipo-cambio` — llama a Banxico (`SF43718`); si falla devuelve el valor cacheado en `Settings`.
- `GET /api/monedero/{cliente_id}` — balance vigente del cliente: generado (no expirado, sin devolución) menos redimido. El join llega al cliente vía `monederogenerados → ajustesproductos → ajustes.clienteid`.

**Notas de Rust:**
- `ProductoBuscado` y `ClienteBuscado` derivan `sqlx::FromRow` + `serde::Serialize` — sqlx mapea columnas por nombre, serde serializa a JSON directamente con `Json(vec)`.
- Las queries de búsqueda de producto usan `sqlx::query_as` con string dinámica (no `query!`) porque la condición de stock se construye en runtime. La interpolación es segura: `stock_cond` es una constante interna, no input del usuario.
- En Axum 0.8 los parámetros de ruta usan `{param}`, no `:param`.

---

## Actualización de dependencias

**Archivos a mirar:**
- `Cargo.toml` — versiones actualizadas
- `src/filters.rs` — nuevo atributo `#[askama::filter_fn]`
- `src/auth/backend.rs` — removido `#[async_trait::async_trait]`, agregado `KeyInit`

**Qué cambió:**

| Crate | Antes | Ahora |
|---|---|---|
| `askama` | `0.12` | `0.16` |
| `axum-login` | `0.17` | `0.18` |
| `hmac` | `0.12` | `0.13` |
| `sha2` | `0.10` | `0.11` |
| `async-trait` | `0.1` | eliminado |
| `tower-sessions` | `0.14` | `0.14` (sin cambio) |

**Notas:**
- Askama 0.16: los filtros custom requieren `#[askama::filter_fn]` sobre la función y un segundo parámetro `_: &dyn askama::Values`. Sin el atributo, el derive macro busca un tipo, no una función, y falla en compilación.
- axum-login 0.18 eliminó `async_trait` y usa RPITIT nativo. Quitar `#[async_trait::async_trait]` del impl es suficiente; `async fn` en el impl es compatible en edition 2024.
- hmac 0.13: `KeyInit` ya no está en scope por defecto — importar explícitamente.
- `tower-sessions` no se actualizó porque `tower-sessions-sqlx-store 0.15` todavía depende de `tower-sessions-core 0.14`. Cuando el store publique soporte para core 0.15, es cambio de una línea.

---

## Correcciones al resumen del día + UX

**Archivos a mirar:**
- `src/modules/ventas/queries.rs` — campo `es_ingreso_trasladado` en `VentaRow`
- `src/modules/ventas/models.rs` — campo `es_ingreso_trasladado` en `VentaLinea`; campo `ingresos_trasladados` en `ResumenDia`
- `templates/ventas/index.html` — fila "Ingresos trasladados" condicional; eliminado `[nid]` junto al nombre de producto
- `templates/base.html` — botones de scroll ↑/↓ fijos en bottom-right con Alpine.js

**Qué hace:**
- **Ingresos trasladados:** productos con `esservicio = true AND preciocomprapromedio = ultimoprecioventa` (vista `v_ingresos_trasladados`). El resumen los separaba del total de venta en el .NET pero los sumaba todos juntos en Rust. Ahora cada línea lleva `es_ingreso_trasladado` y `ResumenDia` los divide: "Venta productos" y "Ingresos trasladados" son filas separadas. `efectivo_en_caja` sigue siendo correcto: suma ambos antes de restar métodos de pago.
- **Scroll:** dos botones Bootstrap fijos (bottom-right, z-index 1030). Alpine detecta posición con `@scroll.window`; ↑ aparece al bajar 80px, ↓ desaparece al llegar al fondo. `d-print-none` los excluye al imprimir. Viven en `base.html` — disponibles en todas las páginas sin tocar nada más.

---

## Fase 2a — Pulido visual + Settings

**Archivos a mirar:**
- `src/filters.rs` — filtro `pesos<T: Display>`: dos decimales fijos; genérico porque Askama pasa `&T` para campos de loop y `T` para resultados de métodos
- `src/db/settings.rs` + `src/db/mod.rs` — carga los 7 parámetros de negocio desde tabla `settings` al arrancar
- `src/main.rs` — `AppState` ahora incluye `settings: db::settings::Settings`
- `templates/base.html` — navbar estilo .NET (blanco, border-bottom, shadow, Alpine dropdown usuario), footer © 2022
- `templates/ventas/index.html` — filtro `|pesos` en todos los montos; anchos fijos en columnas (5rem cantidad, 7rem precio/total); tfoot en una fila con pago en columna ancha

**Qué hace:**
- `src/filters.rs`: `{{ valor|pesos }}` en cualquier template da exactamente dos decimales. Requiere `use crate::filters;` en el archivo donde se deriva el Template.
- `src/db/settings.rs`: una sola query con `ANY($1)` trae todos los keys; falla en startup si falta alguno (fail-fast). Los keys reales tienen el typo original del .NET: `TARJETRA` en lugar de `TARJETA`.

**Bugs corregidos:**
- `monederosgenerados` → `monederogenerados` (tabla real en PostgreSQL)

---

## Fase 2a — Ventas: landing + listado por fecha

**Archivos a mirar:**
- `src/modules/ventas/models.rs` — `Venta`, `VentaLinea`, `ResumenDia`; métodos `total()` y `hora()`
- `src/modules/ventas/queries.rs` — `ventas_del_dia()`: JOIN único que evita N+1
- `src/modules/ventas/routes.rs` — `GET /ventas?fecha=YYYY-MM-DD`
- `templates/ventas/index.html` — listado con selector de fecha y resumen del día
- `templates/index.html` — landing interno post-login con tiles de módulos
- `src/modules/home/mod.rs` — handler del landing
- `src/filters.rs` — módulo vacío requerido por Askama al usar filtros built-in

**Qué hace:**
`GET /ventas` carga todas las ventas del día (o la fecha seleccionada) en una sola query JOIN, las agrupa por venta en Rust, calcula el monedero generado por venta y muestra el resumen del día al pie. Un `<input type="date">` con `onchange="this.form.submit()"` permite navegar entre días sin JS adicional.

**Notas de dominio:**
- `reportes.sql` documenta la trampa del JOIN en `v_ingresos_mensuales`: nunca `SUM(Pago)` desde un JOIN con `AjustesProductos`. Nuestra query lo respeta: suma de monedero viene de subquery correlated, y los totales del resumen se calculan sobre el array de `Venta` ya agrupado.
- `AjustesProductos.PrecioUnitarioVenta` (no `PrecioUnitario` como en el .NET) — el schema SQL es la referencia canónica.
- `v_stock`, `v_inventario`, `v_ingresos_trasladados` son vistas clave para futuros módulos; están definidas en `reportes.sql`.

**Notas de Rust:**
- `sqlx::query_as::<_, VentaRow>(SQL_STRING).bind(fecha).fetch_all(pool)` — versión sin `!` (sin chequeo en tiempo de compilación), misma que usa xplaya para queries complejas. Las columnas se mapean por nombre al struct `#[derive(sqlx::FromRow)]`.
- Askama 0.12: `length` y `strftime` no están en `BUILT_IN_FILTERS` del derive macro, por lo que generan `filters::name()` buscando un módulo local. Solución: no usar esos filtros en templates — usar `.len()` como método y pre-formatear fechas en Rust con `chrono::NaiveDate::format()`.
- `Decimal::is_zero()` es el modo correcto de comparar decimales con cero en templates Askama (no `> 0` que causa error de tipos `Decimal vs integer`).

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
