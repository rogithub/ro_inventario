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
cargo run
# http://localhost:3000
```

```bash
cargo clippy   # antes de cada commit
```

## Despliegue

Imagen ARM64 multi-stage → `ghcr.io/rogithub/ro-inventario:latest`. GitHub Actions hace el build en push a `main`; ArgoCD hace el deploy al cluster k3s (namespace `papeleria`).

## Plan de desarrollo

Ver [PLAN.md](PLAN.md).

## Contexto de dominio y convenciones

Ver [CLAUDE.md](CLAUDE.md).
