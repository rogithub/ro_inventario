#!/bin/bash
# Corre las mismas revisiones que CI (.github/workflows/ci.yml), en el mismo orden.
# Se detiene en la primera que falle.
#
# Uso: herramientas/revisar.sh
# Requiere cargo-audit (una vez: cargo install cargo-audit --locked) y la base de desarrollo
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

paso() { printf '\n== %s\n' "$1"; }

paso "Formato";                                    cargo fmt --check
paso "Clippy";                                     cargo clippy --all-targets -- -D warnings
paso "Dependencias con vulnerabilidades conocidas"; cargo audit
paso "Pruebas";                                    cargo test
[[ -d e2e/node_modules ]] || (cd e2e && npm ci)
paso "Tipos de las pruebas E2E";                   (cd e2e && npx tsc --noEmit)
paso "Pruebas E2E";                                (cd e2e && npx playwright test)

printf '\nTodo en orden.\n'
