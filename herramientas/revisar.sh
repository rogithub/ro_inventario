#!/bin/bash
# Corre las mismas revisiones que CI (.github/workflows/ci.yml), en el mismo orden.
# Se detiene en la primera que falle.
#
# Uso: herramientas/revisar.sh
# Requiere cargo-audit (una vez: cargo install cargo-audit --locked), sqlx-cli (una vez:
# cargo install sqlx-cli --version 0.9.0 --no-default-features --features postgres --locked) y la base de desarrollo
# (herramientas/dev-db.sh): las pruebas de la base crean una base temporal por prueba.
# Los E2E necesitan Node y los navegadores de Playwright (una vez:
# cd e2e && npx playwright install firefox chromium). Corren sin interfaz gráfica.

set -euo pipefail
cd "$(dirname "$0")/.."

# Si DATABASE_URL no viene de fuera, se arma desde el secreto de desarrollo (sin imprimirla).
if [[ -z ${DATABASE_URL:-} ]]; then
    # shellcheck source=dev-env.sh
    . herramientas/dev-env.sh
fi

# Como CI: se compila con la foto de las consultas (.sqlx/), no contra la base. El paso
# "Consultas SQL al día" revisa que esa foto corresponda al esquema real.
export SQLX_OFFLINE=true

paso() { printf '\n== %s\n' "$1"; }

paso "Formato";                                    cargo fmt --check
paso "Clippy";                                     cargo clippy --all-targets -- -D warnings
paso "Dependencias con vulnerabilidades conocidas"; cargo audit
paso "Consultas SQL al día";                       sqlx migrate run --source crates/db/migrations && SQLX_OFFLINE=false cargo sqlx prepare --workspace --check
paso "Pruebas";                                    cargo test
[[ -d e2e/node_modules ]] || (cd e2e && npm ci)
paso "Tipos de las pruebas E2E";                   (cd e2e && npx tsc --noEmit)
paso "Pruebas E2E";                                (cd e2e && npx playwright test)

printf '\nTodo en orden.\n'
