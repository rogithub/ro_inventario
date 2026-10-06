# Dos personas editando el mismo producto a la vez

**Salió de:** plan de la parte 7c, 2026-10-06 · **Cuándo:** si llega a pasar, o cuando haya más usuarios que editen el catálogo

## Qué pasa
`PgProductos::update` (`crates/db/src/productos/update.rs`) bloquea el renglón mientras guarda, así que el historial de precios siempre compara contra el precio de ese momento. Pero si dos personas abren el mismo producto y guardan, gana la última: los cambios de la primera se pierden sin aviso.

## Por qué importa
Hoy editan el catálogo el dueño y, a lo más, un encargado: es raro. Con más usuarios, un precio corregido podría regresar al anterior.

## Propuesta
Que el formulario mande el `updated_at` que leyó y `update` rechace si cambió («Alguien más lo cambió; recarga para ver lo nuevo»).
