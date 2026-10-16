# Las fechas, con `chrono` y `chrono-tz`: se guardan en UTC y se muestran en la zona del negocio

**Fecha:** 2026-10-07 · **Estado:** vigente

## Contexto
Descontinuar un producto (paso 7c2) es la primera pantalla que muestra una fecha: «Descontinuado el 3 oct 2026». Después vienen las ventas, el corte de caja y el kárdex, donde importa qué día es para el negocio: una venta a las 21:00 en Cancún ya es el día siguiente en UTC. Hasta aquí el código no tenía librería de fechas.

## Decisión
- **`chrono`** para los instantes (`DateTime<Utc>`, que `kernel` reexporta para las áreas) y **`chrono-tz`** para las zonas con nombre (`America/Cancun`). Aprobadas por el dueño el 2026-10-06.
- **Un instante se guarda y viaja en UTC** (`timestamptz` en Postgres; `now()` lo pone la base). Se pasa a la zona del negocio solo al mostrarlo, en la aplicación (`apps/privada/src/dates.rs`).
- **La zona sale de `negocio.toml`** (`time_zone`), se valida al arrancar como `Tz` y llega a las pantallas en `AppState`: una zona mal escrita no deja arrancar la aplicación. Nadie vuelve a preguntar por ella con `if`.
- Las fechas se escriben en español con una tabla propia de meses (`3 oct 2026`), no con la opción de idiomas de `chrono` (`unstable-locales`), que no es estable.
- `chrono` va sin sus funciones por omisión, solo con `std` y `clock` (la hora actual, para las implementaciones en memoria de las pruebas).

## Descartado
- **`time`:** no trae zonas con nombre; habría que manejar a mano el desfase de cada zona.
- **`jiff`:** buena librería, pero sqlx 0.9 no la soporta; habría que convertir en cada consulta.
- **Guardar la hora local en la base** (`timestamp` sin zona): un negocio en otra zona (la frontera) o un cambio de horario la volverían ambigua.

## Consecuencias
- `chrono-tz` incluye en el binario la base de zonas de IANA completa, aunque se use una sola.
- Cuando cambien las reglas de una zona (como pasó con el horario de verano en México en 2022), hay que actualizar `chrono-tz`.
- Las fechas de la v1 no llevan zona (`timestamp`). `tools/migracion-v1` las lee como hora de Cancún y las guarda en UTC; si una hora no existe o es ambigua en Cancún, la migración se detiene. La zona va fija en la herramienta (`TIME_ZONE_V1`), no sale de `negocio.toml`: la v1 solo tuvo un negocio (paso 7d2, 2026-10-16).
