#!/bin/bash
# Crea en la base de desarrollo el usuario de pruebas de la IA (rol Dueño) con E2E_USER y E2E_PASS
# del perfil del dueño, sin imprimirlos. Si ya existe, no cambia nada.
#
# Uso (en kukulkan las variables viven en el perfil de zsh): zsh -ic 'herramientas/dev-usuario-ia.sh'

set -euo pipefail
cd "$(dirname "$0")/.."

[[ -n ${E2E_USER:-} && -n ${E2E_PASS:-} ]] || {
    echo "faltan E2E_USER y E2E_PASS (en kukulkan: correr dentro de zsh -ic '…')" >&2
    exit 1
}
# shellcheck source=dev-env.sh
. herramientas/dev-env.sh

# La contraseña va por la entrada del comando (printf es interno de bash: no aparece en la lista
# de procesos). Nada de lo que se imprime lleva el email ni la contraseña.
if salida=$(printf '%s\n' "$E2E_PASS" | cargo run -q -p privada -- crear-usuario \
        --email "$E2E_USER" --nombre "IA de pruebas" --rol Dueño 2>&1); then
    echo "Usuario de pruebas creado."
elif [[ $salida == *"Ya existe"* ]]; then
    echo "El usuario de pruebas ya existe; no cambia nada."
else
    echo "${salida//"$E2E_USER"/(E2E_USER)}" >&2
    exit 1
fi
