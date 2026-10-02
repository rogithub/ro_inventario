# Paso 7: artículos y productos, en cinco partes

**Salió de:** plan del paso 7, 2026-10-01 · **Cuándo:** se marca cada parte al cerrarla; se borra al cerrar la 7e

## Qué pasa
El diseño 01 no cabe en un cambio de ~500 líneas. El dueño aprobó partirlo así:

- [x] **7a** Esquema y área, sin pantalla. Se partió en tres por tamaño:
  - [x] **7a1** `rust_decimal`; migración 0007 (`articulos` con `kind` producto/servicio/kit, `productos`, `precios_historial`, `uuidv7()` como default en las tablas nuevas y en las existentes); pruebas del esquema; nota de decisión del UUID v7.
  - [x] **7a2** Tipos puros del área: `ArticuloId`, `Nid`, `PrecioVenta` (rechaza en lugar de redondear; siempre con 2 decimales) y `NewProducto` validado.
  - [x] **7a3** `ProductosRepo` (listar y agregar, con quién lo hizo), en memoria, contrato y `PgProductos`; el alta con precio escribe `precios_historial`.
- [x] **7b** `/productos`. Se partió en dos por tamaño:
  - [x] **7b1** Lista con búsqueda por nombre o NID (en Postgres con `unaccent`; `Nid::parse` rechaza 0 y negativos), filtro por categoría y tope de 100 con el total. Migración 0009 (`unaccent`, `pg_trgm`) y [decisión](../decisiones/2026-10-01-busqueda-de-productos.md).
  - [x] **7b2** Alta en `/productos/nuevo`. Se partió en dos por tamaño:
    - [x] **7b2a** El alta con `editar_catalogo`: formulario en plantilla parcial (`productos/form.html`, para reusarlo desde compras), precio opcional (un producto sin precio dice «Sin precio» en la lista), Pieza como unidad por omisión, marca, modelo, color y descripción plegados en «Más datos». Al guardar, el formulario regresa vacío con el aviso «Se agregó el NID … «…»» y conserva la categoría y la unidad. Los parecidos quedan para compras ([pendiente](productos-parecidos.md)).
    - [x] **7b2b** E2E de búsqueda con productos creados desde la pantalla (por nombre, sin acentos, por NID y por categoría), en los dos anchos.
- [ ] **7c** Editar (un cambio de precio escribe `precios_historial`) y descontinuar con motivo.
- [ ] **7d** `tools/migracion-v1`: categorías, productos e historial de precios. Cuadra los conteos al correr y se detiene si no coinciden. **Al terminar, adelanta la secuencia del NID** (`setval(pg_get_serial_sequence('articulos', 'nid'), max(nid))`): si no, la primera alta después de migrar recibe el NID 1 y choca (revisor de 7a1); con su prueba. Cierra `migracion-v1-categorias` y `nombres-de-la-v1`.
- [ ] **7e** Presentaciones y códigos de barras, con su migración desde la v1 (la hoja, NID 40; `codigobarrasitem`/`caja`). Agrega `presentacion_id` (opcional) a `precios_historial`, que en la 0007 no está porque aún no existe `presentaciones`. Si pasa de 500 líneas, se parte.

## Respuestas del dueño (2026-10-01)
1. **`rust_decimal` aprobado.** Precios de venta con 2 decimales (`numeric(12,2)`); cantidades y factores con 3; costo unitario con 6 (llega con el kárdex). Lo que se cobra se redondea una sola vez, en el servidor.
2. **Hay alta en el catálogo.** Un producto dado de alta sin recepción quedará "por llegar" cuando exista el kárdex (diseño 02); el formulario se reusa desde compras.
3. **Los nombres de la v1 se migran tal cual, en mayúsculas.** Lo que se capture se guarda como se escriba. Los dobles espacios sí se juntan al migrar.
4. **Ids:** se conservan los uuid de la v1; los nuevos son UUID v7 (`uuidv7()` de Postgres 18).
5. **`kind` acepta producto, servicio y kit desde ahora;** en el paso 7 solo se crean productos.
6. **La migración se verifica al correr** (conteos contra la copia de la v1), no en `cargo test`: CI no tiene la v1. Lee la v1 con `sqlx::query` sin macros.
7. **El kárdex va con compras,** no en el paso 7.

## Por qué importa
Para no perder el plan entre sesiones.
