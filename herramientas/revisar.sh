#!/bin/bash
# Corre las mismas revisiones que CI (.github/workflows/ci.yml), en el mismo orden.
# Se detiene en la primera que falle.
#
# Uso: herramientas/revisar.sh
# Requiere cargo-audit (una vez: cargo install cargo-audit --locked).

set -euo pipefail
cd "$(dirname "$0")/.."

paso() { printf '\n== %s\n' "$1"; }

paso "Formato";                                    cargo fmt --check
paso "Clippy";                                     cargo clippy --all-targets -- -D warnings
paso "Dependencias con vulnerabilidades conocidas"; cargo audit
paso "Pruebas";                                    cargo test

printf '\nTodo en orden.\n'
