#!/bin/bash
# Le da a `ro_inventario` permiso de solo lectura sobre la copia de la v1, `dev_inventario_papeleria`,
# en el Postgres de kukulkan (contenedor podman `postgres`). Sirve para consultar datos reales al
# decidir y como origen de la migración desde la v1.
#
# Uso: herramientas/dev-v1-lectura.sh
# Se puede correr las veces que sea. Refrescar la copia desde producción la recrea y borra estos
# permisos: después de cada refresco se corre otra vez.
# Solo da permisos: no crea, cambia ni borra datos. Para quitarlos, basta con refrescar la copia.

set -euo pipefail

podman exec -i postgres psql -U postgres -d dev_inventario_papeleria -v ON_ERROR_STOP=1 -q <<'SQL'
GRANT CONNECT ON DATABASE dev_inventario_papeleria TO ro_inventario;
GRANT USAGE ON SCHEMA public TO ro_inventario;
GRANT SELECT ON ALL TABLES IN SCHEMA public TO ro_inventario;
-- Aunque alguien le diera más permisos, sus transacciones en esta base son de solo lectura.
ALTER ROLE ro_inventario IN DATABASE dev_inventario_papeleria SET default_transaction_read_only = on;
SQL

echo "Listo: ro_inventario puede leer dev_inventario_papeleria. Consultas: herramientas/v1.sh \"SELECT …\""
