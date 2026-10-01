# Herramienta para consultar los logs de producción

**Salió de:** CLAUDE.md (sección de logs), 2026-10-01 · **Cuándo:** cuando la v2 esté en producción

## Qué pasa
La IA debe consultar los logs antes de pedirle un error al dueño, pero `herramientas/logs.sh` todavía no existe. Mientras tanto se usa la de la versión uno: `/home/ro/code/inventario_papeleria/herramientas/logs-prod.sh`.

## Por qué importa
Sin ella, cada error de producción de la v2 empieza con el dueño buscando el id de petición a mano.

## Propuesta
Hacer `herramientas/logs.sh` a partir de la de la v1. Lee el token de Grafana (rol Viewer) de `~/secrets/grafana.env` sin imprimirlo y busca por el campo `request_id`. Hace falta hasta que haya producción de la v2.
