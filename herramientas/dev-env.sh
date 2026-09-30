# Arma DATABASE_URL para desarrollo desde ~/secrets/ro_inventario_dev.env, sin imprimirla.
# Se carga con `.` (no se ejecuta): . herramientas/dev-env.sh
# Lo usan herramientas/revisar.sh y quien corra la aplicación en desarrollo.

_secreto="$HOME/secrets/ro_inventario_dev.env"
if [[ -r $_secreto ]]; then
    # shellcheck disable=SC1090
    . "$_secreto"
    export DATABASE_URL="postgres://ro_inventario:${DEV_DB_PASSWORD}@localhost:5432/dev_ro_inventario"
    unset DEV_DB_PASSWORD
else
    echo "falta $_secreto con DEV_DB_PASSWORD=…" >&2
fi
unset _secreto
