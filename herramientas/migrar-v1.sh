#!/bin/bash
# Migra la copia de la v1 (`dev_inventario_papeleria`) a la base de desarrollo de la v2
# (`dev_ro_inventario`), las dos en el Postgres de kukulkan. Solo escribe en una base vacía:
# para correrla otra vez, primero herramientas/dev-db.sh (y después herramientas/dev-usuario-ia.sh).
#
# Uso: herramientas/migrar-v1.sh
# Las URL llevan la contraseña del secreto de desarrollo: se arman aquí, solo en el entorno de
# este proceso, y no se imprimen. `ro_inventario` solo puede leer la v1 (herramientas/dev-v1-lectura.sh).

set -euo pipefail
cd "$(dirname "$0")/.."

SECRETO=.secretos/dev.env
[[ -r $SECRETO ]] || SECRETO="$HOME/secrets/ro_inventario_dev.env"
[[ -r $SECRETO ]] || { echo "falta .secretos/dev.env (o ~/secrets/ro_inventario_dev.env) con DEV_DB_PASSWORD=…" >&2; exit 1; }
# shellcheck disable=SC1090
. "$SECRETO"

export V1_DATABASE_URL="postgres://ro_inventario:${DEV_DB_PASSWORD}@localhost:5432/dev_inventario_papeleria"
export DATABASE_URL="postgres://ro_inventario:${DEV_DB_PASSWORD}@localhost:5432/dev_ro_inventario"
unset DEV_DB_PASSWORD

exec cargo run --quiet --release -p migracion-v1
