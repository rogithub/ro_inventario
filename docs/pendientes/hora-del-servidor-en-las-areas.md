# Las áreas pueden leer la hora del servidor

**Salió de:** revisor de 7c2a, 2026-10-07 · **Cuándo:** con ventas, cuando una regla del negocio dependa de la hora

## Qué pasa
`chrono` va con la opción `clock` para todo el workspace (la usan las implementaciones en memoria de las pruebas), y `kernel` reexporta `Utc`. Nada impide que un área llame `Utc::now()` y meta la hora del servidor en una regla. Por ahora es convención: las fechas las pone la base con `now()` (decidido por el dueño el 2026-10-07).

## Por qué importa
Una regla que lee el reloj por su cuenta es difícil de probar (el corte de caja a medianoche, una promoción por fecha) y puede no coincidir con la hora que guardó la base.

## Propuesta
Al llegar ventas: quitar `clock` de las dependencias normales y que quien necesite la hora la reciba desde afuera (un parámetro o un trait de reloj que implemente la aplicación).
