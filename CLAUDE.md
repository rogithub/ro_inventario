# CLAUDE.md — ro_inventario

POS interno de papelería para 2 usuarios (propietario y esposa). Desplegado en `papeleria.xplaya.com`. Reescritura en Rust del sistema .NET (`inventario_papeleria`). Ambos proyectos comparten la misma base de datos PostgreSQL; el .NET permanece en producción sin modificaciones hasta el cutover final.

---

## Repos relacionados

| Repo | Ruta local | Rol |
|---|---|---|
| **ro_inventario** | `/mnt/storage/data/code/ro_inventario` | Este proyecto — reemplaza `inventario_papeleria` |
| **inventario_papeleria** | `/mnt/storage/data/code/inventario_papeleria` | Predecesor .NET — referencia de dominio y comportamiento |
| **Ro.Inventario.Core** | `/mnt/storage/data/code/inventario_papeleria/Ro.Inventario.Core` | Entidades y repositorios .NET — referencia canónica del esquema |
| **xplaya** | `/mnt/storage/data/code/xplaya` | Tienda pública — Rust + Axum, misma BD, arquitectura de referencia |
| **k3s-manifests** | `/mnt/storage/data/code/k3s-manifests` | GitOps — manifiestos ArgoCD del cluster k3s |

---

## Arquitectura general

```mermaid
graph TD
    Browser["Navegador\n(Firefox desktop / Safari iPad mini)"]
    Axum["ro_inventario\nRust + Axum"]
    DB[("PostgreSQL\ninventario_papeleria")]
    Minio["MinIO\ncntnt.xplaya.com"]
    Banxico["Banxico API\ntipo de cambio"]

    Browser -->|HTTP| Axum
    Axum -->|sqlx queries| DB
    Axum -->|fotos productos| Minio
    Axum -->|tipo de cambio USD| Banxico

    subgraph "Cluster k3s — namespace papeleria"
        Axum
    end
```

---

## Stack

**Backend**
- Rust + Axum — servidor HTTP
- sqlx — queries SQL crudas a PostgreSQL (sin ORM); `query!` macro verifica en compile time con `uuid` y `chrono` features activados
- axum-login + tower-sessions — autenticación por cookie de sesión
- tower-sessions-sqlx-store — sesiones en PostgreSQL (tabla propia, independiente del .NET)
- Askama — templates HTML SSR tipados en compile time; errores de template en `cargo build`
- rust-decimal — aritmética exacta para valores monetarios; sqlx mapea `NUMERIC` de PostgreSQL a `Decimal`; **nunca usar `f64` para precios, totales o comisiones**
- tracing + tracing-subscriber — logging estructurado; tower-http `TraceLayer` loguea cada request automáticamente
- dotenvy — carga del `.env` en desarrollo
- reqwest — cliente HTTP para Banxico y APIs externas
- anyhow + thiserror — manejo de errores
- serde + serde_json — serialización
- aws-sdk-s3 — fotos de productos en MinIO

**Frontend**
- htmx — listados con filtros, formularios CRUD simples
- Alpine.js — carrito / cálculos en tiempo real (x-data, getters)
- Bootstrap 5 — CSS (igual al .NET predecesor)
- Chart.js — gráficas (sin cambio)
- Sin build step — htmx y Alpine desde CDN; Bootstrap desde CDN

**Infra**
- PostgreSQL compartida — base `inventario_papeleria`
- MinIO — `cntnt.xplaya.com` (sin cliente MinIO en runtime, solo construcción de URL; aws-sdk-s3 solo para upload)
- Despliegue: contenedor OCI ARM64 en k3s vía ArgoCD
- Namespace: `papeleria`

---

## Estructura del proyecto

```
ro_inventario/
├── src/
│   ├── main.rs              # Startup: config, pool, router, auth, middleware
│   ├── config.rs            # Variables de entorno (DATABASE_URL, PORT, etc.)
│   ├── error.rs             # AppError que implementa IntoResponse
│   ├── auth/
│   │   ├── mod.rs
│   │   ├── backend.rs       # axum-login AuthnBackend impl
│   │   └── routes.rs        # GET/POST /login, GET /logout
│   ├── db/
│   │   └── mod.rs           # Pool sqlx
│   └── modules/
│       ├── ventas/
│       │   ├── mod.rs
│       │   ├── routes.rs    # Axum handlers
│       │   ├── queries.rs   # sqlx queries
│       │   └── models.rs    # structs de dominio
│       ├── compras/
│       ├── productos/
│       ├── clientes/
│       ├── proveedores/
│       ├── pedidos/
│       ├── ordenes_compra/
│       ├── ajustes/
│       ├── toners/
│       └── reportes/
├── templates/
│   ├── base.html            # Layout: Bootstrap 5 + htmx + Alpine desde CDN
│   ├── auth/
│   │   └── login.html
│   ├── ventas/
│   │   ├── index.html       # Listado
│   │   ├── nueva.html       # Formulario con Alpine.js
│   │   └── partials/        # Fragmentos htmx
│   └── shared/
│       └── _nav.html
├── static/
│   └── css/
│       └── main.css
├── Cargo.toml
├── Containerfile            # Multi-stage, ARM64
├── .env.example
└── CLAUDE.md
```

---

## Variables de entorno

| Variable | Requerida | Default | Descripción |
|---|---|---|---|
| `DATABASE_URL` | Sí | — | `postgres://user:pass@host/inventario_papeleria` |
| `PORT` | No | `3000` | Puerto HTTP |
| `CONTENT_BASE_URL` | No | `https://cntnt.xplaya.com` | Base URL para fotos en MinIO: `{CONTENT_BASE_URL}/papeleria-fotos-productos/{filename}` |
| `SESSION_SECRET` | Sí | — | Clave para firmar cookies de sesión |

Copiar `.env.example` a `.env` para desarrollo. `.env` está en `.gitignore`.

---

## Desarrollo local

```bash
cp .env.example .env          # completar DATABASE_URL y SESSION_SECRET
cargo watch -x run            # reinicia el servidor automáticamente al guardar
# http://localhost:3000
```

Antes de cada commit:
```bash
cargo clippy
```

- **No usar `#![deny(clippy::all)]`** — usa `#[allow(clippy::nombre_del_lint)]` en la línea específica si un lint no aplica.

---

## Despliegue

- Namespace k3s: `papeleria`
- Imagen: `ghcr.io/rogithub/ro-inventario:latest` (ARM64), siempre `latest`
- Manifiestos en `k3s-manifests/workloads/papeleria/`
- Secrets vía SealedSecrets — **nunca commitear secrets en texto plano**
- CI/CD: GitHub Actions build ARM64 + push a `ghcr.io`; ArgoCD deploy automático

---

## Dominio de negocio

### Entidades y tablas principales

| Tabla | Descripción |
|---|---|
| `Ajustes` | Ventas del POS. `TipoAjuste`: 0=Venta, 1=Merma, 2=IngresoSinCompra |
| `AjustesProductos` | Líneas de cada venta |
| `Compras` | Órdenes de compra a proveedor |
| `ComprasProductos` | Líneas de cada compra |
| `Productos` | Artículos de inventario. `Nid` (int, búsqueda rápida), `Id` (UUID) |
| `Clientes` | Clientes con monedero electrónico (`Contactos` con `Tipo=0`) |
| `Proveedores` | Proveedores (`Contactos` con `Tipo=1`) |
| `Pedidos` | Cotizaciones/pedidos previos a venta |
| `OrdenesCompra` | Órdenes de compra a proveedor |
| `MonederoGenerados` | Cashback generado por línea de venta |
| `MonederoRedimidos` | Cashback usado |
| `Settings` | Configuración clave-valor del sistema |
| `v_stock` | Vista — stock actual por producto (**usar siempre, nunca recalcular**) |
| `v_inventario` | Vista — stock con detalles de producto |
| `v_ingresos_mensuales` | Vista — ingresos agrupados sin trampa del JOIN |

### Enums (columnas INT en la BD)

| Enum | Columna | Valores |
|---|---|---|
| `TipoAjuste` | `Ajustes.TipoAjuste` | 0=Venta, 1=Merma, 2=IngresoSinCompra |
| `MetodoPago` | `Compras.MetodoPago`, `Toners.MetodoPago` | 0=Efectivo, 1=Transferencia, 2=Tarjeta |
| `TipoContacto` | `Contactos.Tipo` | 0=Cliente, 1=Proveedor |
| `EstatusPedido` | `Pedidos.Estatus` | 0=Nuevo, 1=Pagado, 2=Entregado |
| `OrigenPedido` | `Pedidos.Origen` | 0=Tienda, 1=EnLinea |
| `OrdenCompraEstatus` | `OrdenesCompra.Estatus` | 0=Nueva, 1=Pagada, 2=EnCamino, 3=Recibida, 4=Cancelada, 5=Procesada |

---

## Reglas críticas de dominio

### Stock — nunca recalcular manualmente

El stock no es una columna. Usar siempre las vistas `v_stock` o `v_inventario`. Reconstruirlo a mano rompe la coherencia con compras, mermas e ingresos sin compra.

### Trampa del JOIN en queries de pagos

```sql
-- MAL: si el ticket tiene 8 productos, SUM(Pago) se multiplica 8 veces
SELECT SUM(a.Pago) FROM Ajustes a JOIN AjustesProductos ap ON a.Id = ap.AjusteId
```

Calcular pagos solo desde `Ajustes`; líneas de producto desde `AjustesProductos` en subqueries separados. Ver `v_ingresos_mensuales` como referencia.

### Comisión por tarjeta — cálculo iterativo

La comisión se calcula **dos veces sobre el monto en tarjeta**: primero sobre el subtotal, luego sobre `(subtotal + primera comisión)`. Se agrega como **línea de producto** (no como campo separado). `comisionTcServicioId` es el UUID del producto-servicio que representa la comisión.

### Monedero — cuándo se genera

Se genera un `MonederoGenerado` por cada línea de venta cuando:
- `TipoAjuste = 0` (venta, no merma)
- `ClienteId IS NOT NULL` (cliente registrado, no venta anónima)

Es inválido (excluir del balance) si: `FechaExpiracion <= NOW()` o `DevolucionId IS NOT NULL`.

### OrdenCompra — estado real inferido de fechas

El campo `Estatus` se persiste pero el estado real se infiere:
- `CompraId IS NOT NULL` → Procesada
- `Estatus = 4` → Cancelada
- `FechaPago` presente, `FechaEnvio` NULL → Pagada
- `FechaEnvio` presente, `FechaLlegada` NULL → EnCamino
- `FechaLlegada` presente → Recibida
- Sin fechas → Nueva

Solo actualizable si `CompraId IS NULL`.

### Métodos de pago en venta

```
Pago             → efectivo (MXN)
PagoMonedero     → monedero electrónico del cliente
PagoTarjeta      → tarjeta (lleva comisión iterativa)
PagoTransferencia → transferencia bancaria
PagoDolares      → USD (con TipoCambioDolares desde Banxico)
Cambio           → cambio entregado al cliente
```

Solo efectivo y dólares pueden dar cambio; transferencia y tarjeta son monto exacto.

---

## Convenciones

- Código en inglés (variables, funciones, módulos, structs); textos de UI en español.
- Un módulo por entidad de negocio en `src/modules/`; cada módulo tiene `routes.rs`, `queries.rs`, `models.rs`.
- Un archivo por tema en `db/` si hay lógica compartida entre módulos.
- Templates en `templates/`; fragmentos htmx en `templates/*/partials/`.
- CSS/JS propio (mínimo) en `static/`; librerías externas desde CDN.
- No añadir capas de abstracción que no aporten funcionalidad real.
- Imágenes siempre `latest` (proyecto propio, un solo consumer).

### Frontend por tipo de página

| Tipo de página | Enfoque |
|---|---|
| Listados con filtros | htmx (`hx-get`, `hx-target`) |
| Formularios CRUD simples | htmx + validación server-side |
| Carrito / cálculos en tiempo real | Alpine.js (`x-data`, getters) |
| Gráficas | Chart.js (sin cambio vs .NET) |

### Manejo de errores en handlers

```rust
// AppError implementa IntoResponse — los handlers devuelven Result<impl IntoResponse, AppError>
// anyhow para errores internos, thiserror para errores de dominio
```

---

## Entorno de uso — terminales

| Terminal | Dispositivo | Navegador |
|---|---|---|
| Caja principal | Desktop Linux Debian | Firefox |
| Caja secundaria | iPad mini | Safari/iPadOS |

**Implicaciones de UI:**
- iPad mini ~744px en portrait — entre `sm` (576px) y `md` (768px) de Bootstrap
- Usar `d-none d-md-inline` (no `d-sm-inline`) para ocultar texto en móvil e iPad
- Tablas compactas (`table-sm`); botones suficientemente grandes para touch

---

## Modo de trabajo con AI

- **Avance real**: construir a ritmo normal, mezclar conceptos está bien.
- **Explicar al escribir**: al introducir algo nuevo (Axum, sqlx, Askama, htmx, Alpine), explicar brevemente qué hace y por qué aquí. Sin pausas formales.
- **El usuario pregunta**: no hacer preguntas de comprensión. El usuario revisa y pregunta si algo no queda claro.
- **PLAN.md**: hoja de ruta con pasos. Marcar cada paso como completado al terminarlo.
- **REVISIONES.md**: actualizar después de cada commit — qué archivos mirar, qué hace el cambio. Entradas más recientes arriba.
- **El usuario dirige**: proponer opciones ante decisiones de diseño, no tomarlas solo.
- **Claridad sobre sofisticación**: no abstraer hasta que la repetición lo justifique.
