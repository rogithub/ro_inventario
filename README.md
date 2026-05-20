# ro_inventario

POS e inventario para papelería — Rust + Axum. Reescritura del sistema .NET (`inventario_papeleria`). Comparte la misma base de datos PostgreSQL; el .NET permanece en producción hasta el cutover final.

Desplegado en `papeleria.xplaya.com`.

## Stack

| Capa | Tecnología |
|---|---|
| Web framework | Axum |
| Base de datos | sqlx (raw SQL, PostgreSQL) |
| Autenticación | axum-login + tower-sessions |
| Session store | tower-sessions-sqlx-store |
| Templates | Askama |
| Frontend interactivo | htmx + Alpine.js |
| CSS | Bootstrap 5 |
| Archivos / fotos | aws-sdk-s3 (MinIO) |
| HTTP cliente | reqwest (Banxico) |

## Desarrollo local

```bash
cp .env.example .env   # completar DATABASE_URL y SESSION_SECRET
cargo watch -x run     # auto-reload al guardar; instalar con: cargo install cargo-watch
# http://localhost:3000
```

### Antes de cada commit

```bash
cargo clippy
```

Si agregaste o modificaste algún `sqlx::query!`, también:

```bash
cargo sqlx prepare   # actualiza el caché en .sqlx/
git add .sqlx/
```

### ¿Por qué existe `.sqlx/`?

`sqlx::query!` verifica el SQL contra Postgres **en tiempo de compilación**. Para que CI y otras máquinas puedan compilar sin BD, `cargo sqlx prepare` guarda esa información en archivos JSON dentro de `.sqlx/`. Esos archivos se committean al repo.

Si CI falla con `set DATABASE_URL to use query macros online, or run cargo sqlx prepare`, es que hay un `query!` nuevo sin su JSON. Solución: correr `cargo sqlx prepare` con la BD activa y commitear `.sqlx/`.

Para instalar `sqlx-cli`:
```bash
cargo install sqlx-cli --no-default-features --features postgres
```

Para compilar sin BD (verificar que el caché está al día):
```bash
SQLX_OFFLINE=true cargo build
```

## Despliegue

Imagen ARM64 multi-stage → `ghcr.io/rogithub/ro-inventario:latest`. GitHub Actions hace el build en push a `main`; ArgoCD hace el deploy al cluster k3s (namespace `papeleria`).

## Plan de desarrollo

Ver [PLAN.md](PLAN.md).

## Contexto de dominio y convenciones

Ver [CLAUDE.md](CLAUDE.md).
