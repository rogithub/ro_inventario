#!/bin/bash
# Crea (o recrea desde cero) la base de desarrollo de la v2, `dev_ro_inventario`, en el Postgres de
# kukulkan (contenedor podman `postgres`, localhost:5432), con su usuario `ro_inventario`.
# Nunca toca producción ni `dev_inventario_papeleria`.
# Las migraciones se aplican solas al arrancar la aplicación.
#
# Uso: herramientas/dev-db.sh
# Requiere ~/secrets/ro_inventario_dev.env (chmod 600) con DEV_DB_PASSWORD=… (solo letras y números).
# La contraseña no se imprime ni aparece en la lista de procesos: va a psql por su entrada.

set -euo pipefail

SECRETO="$HOME/secrets/ro_inventario_dev.env"
[[ -r $SECRETO ]] || { echo "falta $SECRETO con DEV_DB_PASSWORD=…" >&2; exit 1; }
# shellcheck disable=SC1090
. "$SECRETO"
[[ ${DEV_DB_PASSWORD:-} =~ ^[A-Za-z0-9]{16,}$ ]] || { echo "DEV_DB_PASSWORD debe tener 16+ letras o números" >&2; exit 1; }

psql_admin() { podman exec -i postgres psql -U postgres -v ON_ERROR_STOP=1 -q "$@"; }

# Usuario de la v2: puede crear bases porque sqlx::test crea una por prueba.
psql_admin <<SQL
DO \$\$ BEGIN
  IF NOT EXISTS (SELECT FROM pg_roles WHERE rolname = 'ro_inventario') THEN
    CREATE ROLE ro_inventario LOGIN CREATEDB;
  END IF;
END \$\$;
ALTER ROLE ro_inventario PASSWORD '$DEV_DB_PASSWORD';
SQL

psql_admin -c "DROP DATABASE IF EXISTS dev_ro_inventario WITH (FORCE);"
psql_admin -c "CREATE DATABASE dev_ro_inventario OWNER ro_inventario;"

echo "Listo: dev_ro_inventario creada desde cero. Las migraciones se aplican al arrancar la aplicación."
