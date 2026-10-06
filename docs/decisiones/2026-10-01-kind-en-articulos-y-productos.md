# `kind` va en `articulos` y también en `productos`: la base impide que un servicio tenga renglón de producto

**Fecha:** 2026-10-01 (anotada el 2026-10-06) · **Estado:** vigente

## Contexto
Lo común de todo lo que se vende en caja (nombre, NID, precio) vive en `articulos`, y su tipo en `articulos.kind` (`producto`, `servicio` o `kit`). Lo propio de cada tipo va en su tabla: `productos` (marca, modelo, color; después, stock), `servicios` y los componentes de un kit (diseño 03). Una llave foránea simple de `productos.articulo_id` a `articulos.id` dejaría que un servicio o un kit tuviera renglón en `productos`: un servicio con marca, y más adelante con stock en el kárdex. Al dueño le pareció un dato repetido; esta nota explica por qué no lo es.

## Decisión
- `articulos` declara única la pareja `(id, kind)` (`articulos_id_kind_unique`).
- `productos.kind` siempre vale `'producto'` (`CHECK (kind = 'producto')`, y ese valor por omisión: la aplicación no lo escribe).
- La llave foránea de `productos` es la pareja: `FOREIGN KEY (articulo_id, kind) REFERENCES articulos (id, kind)`.
- Así, `productos` solo acepta artículos de tipo producto, y un artículo con renglón en `productos` no puede cambiar de tipo. Lo fijan dos pruebas del esquema (`crates/db/src/articulos_schema_tests.rs`): `solo_un_articulo_de_tipo_producto_tiene_renglon_en_productos` y `un_producto_con_renglon_en_productos_no_puede_cambiar_de_kind`.
- `servicios` y lo que agreguen los kits repiten el patrón, con su propio valor.

Migración: `crates/db/migrations/0007_articulos.sql`.

## Descartado
- **Llave simple y confiar en la aplicación:** el área solo crea productos por su repositorio, pero una migración, un script o un error futuro podría ligar un servicio, y lo descubriríamos en el kárdex o en un reporte.
- **Un trigger que revise el tipo:** la regla queda escondida en código procedural, se olvida al leer el esquema y cuesta más probarla. Con llaves y `CHECK` se ve en la definición de la tabla.
- **Una sola tabla con todas las columnas de todos los tipos:** columnas que solo valen para un tipo y `CHECK` cruzados por `kind`; ya se decidió separar lo común de lo propio (diseño 01, respuesta 4 del dueño).

## Consecuencias
- Una columna que siempre dice lo mismo en cada tabla de tipo, y un índice único de más en `articulos`. Es barato.
- Cambiar el tipo de un artículo exige primero quitar su renglón de la tabla del tipo anterior. Si algún día hace falta convertir un producto en kit, se hace a propósito, con su migración o su pantalla.
