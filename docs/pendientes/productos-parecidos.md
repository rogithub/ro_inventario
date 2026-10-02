# Mostrar los productos parecidos antes de dar de alta uno nuevo

**Salió de:** plan de la parte 7b2, 2026-10-02 · **Cuándo:** con compras (al reusar el formulario de alta), o antes si aparecen duplicados

## Qué pasa
El alta de `/productos/nuevo` (parte 7b2) guarda sin buscar si ya existe uno parecido. El diseño 02 pide que, al capturar una compra, «Crear producto nuevo» aparezca después de ver los parecidos: en la v1 se dio de alta otra vez la cartulina (NID 52) con el nombre de otro proveedor, y se tuvo que limpiar con scripts.

## Por qué importa
Un duplicado parte el stock, el costo y el historial de un mismo producto en dos.

## Propuesta
Al reusar el formulario (`apps/privada/templates/productos/form.html`) desde compras, mostrar los parecidos mientras se escribe el nombre, con la búsqueda que ya existe (`ProductosRepo::search`). Hay que decidir si el alta del catálogo también los muestra. Por ahora el dueño decidió que no (2026-10-02).
