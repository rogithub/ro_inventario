#!/bin/bash
# Consulta la copia de la v1 (`dev_inventario_papeleria`) como `ro_inventario`, que ahí solo puede leer
# (permisos de herramientas/dev-v1-lectura.sh).
#
# Uso: herramientas/v1.sh "SELECT …"
# Solo SQL, una consulta por llamada: sin psql interactivo ni comandos con barra invertida (`\c`
# conectaría a otro servidor con este secreto, y el hook de producción no lo vería). Las
# transacciones son de solo lectura, también desde este lado.
# La contraseña sale del mismo secreto de desarrollo que dev-env.sh y va a psql por PGPASSWORD,
# no en la línea de comando: no se imprime ni aparece en la lista de procesos.

set -euo pipefail

SECRETO="$(dirname "$0")/../.secretos/dev.env"
[[ -r $SECRETO ]] || SECRETO="$HOME/secrets/ro_inventario_dev.env"
[[ -r $SECRETO ]] || { echo "falta .secretos/dev.env (o ~/secrets/ro_inventario_dev.env) con DEV_DB_PASSWORD=…" >&2; exit 1; }
# shellcheck disable=SC1090
. "$SECRETO"
[[ $# -eq 1 ]] || { echo "uso: herramientas/v1.sh \"SELECT …\"" >&2; exit 1; }
[[ ${1#"${1%%[![:space:]]*}"} != \\* ]] || { echo "solo SQL: los comandos de psql (\\…) no se aceptan" >&2; exit 1; }

export PGPASSWORD="$DEV_DB_PASSWORD"
export PGOPTIONS="-c default_transaction_read_only=on"
unset DEV_DB_PASSWORD

exec psql -h localhost -p 5432 -U ro_inventario -d dev_inventario_papeleria -v ON_ERROR_STOP=1 -c "$1"
