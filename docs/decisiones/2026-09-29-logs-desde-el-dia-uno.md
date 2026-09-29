# Logs estructurados desde el día uno, legibles por la IA

**Fecha:** 2026-09-29 · **Estado:** vigente

## Contexto
El desarrollo es asistido por IA, y la IA solo puede ayudar con un problema de producción si lo ve. En la versión uno esto llegó tarde: los logs de ASP.NET se volvieron legibles después, y la herramienta de consulta (`/home/ro/code/inventario_papeleria/herramientas/logs-prod.sh`) busca por el nombre del archivo del pod y filtra texto libre, lo que es frágil (un filtro con corchetes rompió la consulta). La infraestructura ya existe: Loki recoge los logs de todos los pods y Grafana da acceso con un token de solo lectura (rol Viewer) en `~/secrets/grafana.env`.

## Decisión
- **Logs estructurados con `tracing`:** en producción, una línea JSON por evento; en desarrollo, formato legible para humanos. Mismo código, lo elige el entorno.
- **Campos en cada línea:** fecha, nivel, módulo, mensaje; más el **id de la petición** y, cuando aplica, el id del objeto del negocio (`venta_id`, `compra_id`) como campo, no dentro del texto.
- **Cada petición HTTP lleva un id** que aparece en todas sus líneas y en la pantalla de error. El dueño reporta "falló la venta, error `a1b2c3`" y la IA encuentra exactamente esas líneas.
- **Niveles con significado:**
  | Nivel | Cuándo |
  |---|---|
  | `error` | Algo falló y una persona debe enterarse |
  | `warn` | Algo raro que el sistema resolvió solo |
  | `info` | Eventos que cuentan la historia: arranque, migraciones aplicadas, venta guardada (id y total), recálculos |
  | `debug` | Detalle para depurar; apagado en producción |
- **Al arrancar se registra** la versión (commit), las migraciones aplicadas y la configuración cargada **sin secretos**: la IA sabe qué está corriendo antes de leer lo demás.
- **Nunca en los logs:** contraseñas, tokens, llaves, cookies ni encabezados de autorización. (Lección del cluster: un token de R2 vivió 271 días en logs de un respaldo.)
- **Herramienta de consulta en el repo** (`herramientas/logs.sh`), heredera de la de la versión uno: mismo token de solo lectura; filtra por negocio (cada instancia vive en su namespace) y por campos JSON (`level`, `venta_id`, `request_id`) en lugar de texto libre.
- **La IA la usa antes de pedirle un error al dueño.** Tiene acceso de solo lectura a los logs de producción; nunca a su base de datos.
- **Salud:** cada aplicación expone `/health` (versión y estado de la base) para las sondas de Kubernetes.

## Descartado
- **Texto libre** (como la versión uno): obliga a buscar con expresiones regulares frágiles y mezcla el dato con la frase.
- **Un servicio externo de logs** (Datadog, Sentry): Loki y Grafana ya están en el cluster.
- **Métricas desde el día uno** (Prometheus): útiles, pero los logs responden primero "qué falló". Se agregan cuando haga falta medir tendencias.

## Consecuencias
- Cada error de producción se puede rastrear de la pantalla del usuario a sus líneas de log con un id.
- Escribir un log es parte de escribir una funcionalidad: los eventos importantes del negocio llevan su `info`.
- Revisar que un log nuevo no filtre secretos es parte de la revisión de cada cambio.
