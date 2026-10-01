# Migrar las categorías desde la v1

**Salió de:** plan del paso 6, 2026-10-01 · **Cuándo:** con artículos

## Qué pasa
La v2 ya tiene categorías, pero `tools/migracion-v1` todavía no existe. El dueño decidió posponer la migración hasta que haya artículos, que es cuando cuadrar los datos contra la copia de la v1 rinde más.

## Por qué importa
La VISION pide escribir la migración "desde el primer día" y correrla seguido contra una copia de producción, para que las inconsistencias salgan mientras se diseña.

## Propuesta
Al construir artículos, arrancar `tools/migracion-v1` con categorías y artículos juntos. Regla del diseño 01: cada producto conserva su categoría original, y las categorías que se queden sin artículos no se migran. Una prueba compara los conteos contra `dev_inventario_papeleria`.
