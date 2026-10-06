# Cambiar la unidad base de un producto con movimientos en el kárdex

**Salió de:** plan de la parte 7c, 2026-10-06 · **Cuándo:** con compras (al llegar el kárdex)

## Qué pasa
La edición de un producto (`ProductosRepo::update`, parte 7c) deja cambiar su unidad. Hoy no importa: no hay kárdex.

## Por qué importa
En la unidad base se cuentan el stock y el costo (diseño 01). Si un producto con movimientos pasa de pieza a paquete, los renglones viejos del kárdex cambian de significado sin que nadie los toque.

## Propuesta
Cuando exista el kárdex, `update` rechaza el cambio de unidad de un producto con movimientos (con su error y su mensaje). Cambiar la forma de vender va con presentaciones.
