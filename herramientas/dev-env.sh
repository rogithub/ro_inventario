# Arma DATABASE_URL para desarrollo desde el secreto, sin imprimirla: `.secretos/dev.env` dentro del
# repo o, si no existe, ~/secrets/ro_inventario_dev.env.
# Se carga con `.` desde bash o zsh (no se ejecuta): . herramientas/dev-env.sh
# Lo usan herramientas/revisar.sh y quien corra la aplicación en desarrollo.

_secreto="$(git rev-parse --show-toplevel 2>/dev/null)/.secretos/dev.env"
[[ -r $_secreto ]] || _secreto="$HOME/secrets/ro_inventario_dev.env"
if [[ -r $_secreto ]]; then
    # shellcheck disable=SC1090
    . "$_secreto"
    export DATABASE_URL="postgres://ro_inventario:${DEV_DB_PASSWORD}@localhost:5432/dev_ro_inventario"
    unset DEV_DB_PASSWORD
else
    echo "falta .secretos/dev.env (o ~/secrets/ro_inventario_dev.env) con DEV_DB_PASSWORD=…" >&2
fi
unset _secreto
