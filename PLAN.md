# Plan de Desarrollo — ro_inventario

POS interno de papelería. Rust + Axum + sqlx + Askama + htmx + Alpine.js + Bootstrap 5.
Reemplaza `inventario_papeleria` (ASP.NET Core MVC) en `papeleria.xplaya.com`.

**Alcance:** Solo el POS interno. Las páginas públicas (monedero del cliente, recibos, cotizaciones, catálogo) se sirven desde `xplaya` — no pertenecen a este proyecto.

---

## Fases

```
Fase 0 — Infraestructura  →  Fase 1 — Auth
    ↓
Fase 2 — Ventas
    ↓
Fase 3 — Compras
    ↓
Fase 4 — Impresoras
    ↓
Fase 5 — Accesorios  (orden se define al llegar)
    ↓
Fase 6 — Cutover a producción
```

---

## Fase 0 — Infraestructura base *(en curso)*

- [x] `cargo init`, dependencias: axum, sqlx, tokio, tower-http, askama, axum-login, tower-sessions, tower-sessions-sqlx-store, rust-decimal, tracing, tracing-subscriber, dotenvy, anyhow, thiserror, serde, serde_json, reqwest
- [x] `src/config.rs` — `DATABASE_URL`, `PORT`, `SESSION_SECRET`, `CONTENT_BASE_URL`
- [x] `src/error.rs` — `AppError` que implementa `IntoResponse`
- [x] `src/main.rs` — servidor mínimo: pool sqlx, TraceLayer, ServeDir, `GET /` → redirect `/ventas`
- [x] `templates/base.html` — Bootstrap 5, htmx y Alpine.js desde CDN
- [x] `Containerfile` multi-stage ARM64
- [x] `.env.example`
- [x] GitHub Actions: build ARM64 + push a `ghcr.io`

## Fase 1 — Auth

- [x] Sesiones PostgreSQL: tabla `tower_sessions` via tower-sessions-sqlx-store (`.migrate()` al arrancar)
- [x] `src/auth/backend.rs` — `AuthnBackend` contra tabla `Users`; HMAC-SHA512 igual al .NET
- [x] `src/auth/routes.rs` — `GET /login`, `POST /login`, `GET /logout`
- [x] `templates/auth/login.html`
- [x] Middleware de autenticación — `login_required!` protege todas las rutas excepto `/login` y `/static`
- [x] `src/templates.rs` — helper `render<T: Template>()` usado por todos los handlers
- [x] `src/filters.rs` — módulo vacío requerido por Askama cuando se usan filtros
- [ ] `src/db/settings.rs` — leer tabla `Settings`; cargar al arrancar en `AppState`
- [ ] ArgoCD configurado (deploy a staging)

## Fase 2 — Ventas

El módulo más complejo. Se divide en dos subfases.

**2a — Backend**
- [x] `src/modules/ventas/models.rs` — `Venta`, `VentaLinea`, `ResumenDia`
- [x] `src/modules/ventas/queries.rs`
  - [x] Listado por fecha con JOIN eficiente (evita N+1 y trampa del JOIN)
  - [ ] Detalle de venta por `Id`
  - [ ] Crear venta (`Ajustes` + `AjustesProductos` + `MonederoGenerados` en transacción)
  - [ ] Tipo de cambio USD vía Banxico (reqwest)
- [x] `src/modules/ventas/routes.rs`
  - [x] `GET /ventas` — listado con filtro por fecha
  - [ ] `GET /ventas/nueva` — formulario
  - [ ] `POST /ventas` — crear venta
  - [ ] `GET /ventas/:id` — detalle
  - [ ] Endpoints JSON para el frontend (productos por Nid/barcode, tipo de cambio, settings)
- [ ] Templates Askama en `templates/ventas/`
- [ ] Validar reglas de negocio críticas:
  - [ ] Comisión TC: cálculo iterativo en dos pasos; línea de producto; tasa desde `Settings`
  - [ ] Monedero como método de pago
  - [ ] Pago en dólares con tipo de cambio desde Banxico
  - [ ] Solo efectivo y dólares dan cambio
  - [ ] `MonederoGenerado` por línea si `TipoAjuste=0` y `ClienteId IS NOT NULL`; tasa desde `Settings`

**2b — Frontend Alpine.js**
- [ ] Reemplazar KnockoutJS con Alpine.js en formulario de nueva venta
- [ ] htmx para listado y detalle
- [ ] Carrito con cálculos en tiempo real (comisión, monedero, cambio)

## Fase 3 — Compras

- [ ] `src/modules/compras/models.rs`
- [ ] `src/modules/compras/queries.rs` — compras a proveedor, líneas, documentos
- [ ] `src/modules/ordenes_compra/queries.rs` — CRUD + estado inferido de fechas
- [ ] Rutas: listado, detalle, crear, recibir compra
- [ ] Templates: htmx para listados, Alpine.js para formulario de compra
- [ ] Validar: estado de `OrdenCompra` inferido de `FechaPago`/`FechaEnvio`/`FechaLlegada`/`CompraId`

## Fase 4 — Impresoras

- [ ] `src/modules/impresoras/` — CRUD de impresoras
- [ ] `src/modules/toners/` — ciclo de vida: compra, instalación, retiro, contadores
- [ ] `src/modules/servicios_impresora/` — mantenimientos, refacciones, reparaciones
- [ ] Documentos adjuntos (toners y servicios) — upload a MinIO via aws-sdk-s3

## Fase 5 — Accesorios

*Orden se define al llegar a esta fase.*

- [ ] **Productos** — CRUD + fotos MinIO + búsqueda full-text (`search_vector`)
- [ ] **Clientes** — CRUD + balance de monedero (generados vs. redimidos vs. expirados)
- [ ] **Proveedores** — CRUD de `Contactos` con `Tipo=1`
- [ ] **Pedidos** — listado, detalle, conversión a venta
- [ ] **Órdenes de compra** — CRUD (relacionado con Compras)
- [ ] **Ajustes de inventario** — mermas e ingresos sin compra (`TipoAjuste=1,2`); usar `v_stock`
- [ ] **Módulo financiero** — `ConceptoMovimiento` + `MovimientoFinanciero` + documentos
- [ ] **Reportes y dashboard** — ventas del día/mes, stock bajo, ingresos; Chart.js

## Fase 6 — Cutover a producción

- [ ] Validación completa en staging (todos los módulos)
- [ ] Apagar `inventario_papeleria` (.NET) en k3s
- [ ] Arrancar `ro_inventario` apuntando a la misma BD y MinIO de producción
- [ ] Smoke test: login → venta completa → compra → reportes
- [ ] Rollback disponible: apagar Rust, encender .NET (la BD no cambia de esquema)

---

> Marcar cada item con `[x]` al completarlo.
