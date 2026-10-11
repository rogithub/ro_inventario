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
- [ ] **7c** Editar y descontinuar. Plan aprobado el 2026-10-06; se partió en tres por tamaño:
  - [x] **7c1a** Editar en el área y en Postgres, sin pantalla: `get_by_nid` y `update`. El historial recibe un renglón solo si el precio cambió; uno con precio no puede quedar sin precio (`CannotRemovePrecio`: lo que ya no se vende se descontinúa).
  - [x] **7c1b** Pantalla `GET`/`POST /productos/{nid}` con el mismo formulario parcial; el nombre en la lista es el enlace (solo con `editar_catalogo`). Al guardar se queda en la página con el aviso; la flecha regresa a la lista con la búsqueda. Sin `autofocus` en la edición (en el iPad abriría el teclado). `added_notice` del alta pasa a usar `get_by_nid`. E2E: cambiar el precio desde la lista y verlo de regreso.
  - [x] **7c2** Descontinuar con motivo (texto libre hasta 300 caracteres, con ejemplo en el placeholder; `CHECK` en la migración 0010) y «Volver a manejar». Plan aprobado el 2026-10-07; se partió en cuatro por tamaño:
    - [x] **7c2a** Fechas: `chrono` + `chrono-tz` ([decisión](../decisiones/2026-10-07-fechas-con-chrono.md); descartadas `time`, sin zonas con nombre, y `jiff`, sin soporte en sqlx 0.9). `time_zone` de `negocio.toml` se valida al arrancar como `Tz` y llega en `AppState`; `dates::short_date` escribe «3 oct 2026».
    - [x] **7c2b** Área y Postgres, sin pantalla: migración 0010 (`CHECK` de 300), `Producto.descontinuado` (fecha y motivo), `discontinue` (rechaza uno ya descontinuado, para no perder la fecha) y `resume` (en uno que se maneja no hace nada; el motivo anterior se borra, sin historial por ahora). Anotan quién. Agrega la función `chrono` de sqlx.
    - [x] **7c2c** Lista y búsqueda: los descontinuados no salen, salvo con la casilla «Ver descontinuados» (`descontinuados=1` en la dirección; `ListSearch` la recuerda) o al teclear su NID exacto; salen en gris con la insignia «Descontinuado». La E2E prueba la casilla en los dos anchos; la de punta a punta pasa a 7c2d, porque sin su pantalla la E2E no puede descontinuar.
    - [x] **7c2d** Pantalla de edición (`POST /productos/{nid}/descontinuar` y `/volver-a-manejar`, en su propio módulo): sin descontinuar, motivo y botón; descontinuado, la franja «Descontinuado el 3 oct 2026: …» y «Volver a manejar». Quita los `expect(dead_code)` de `dates` y `AppState.time_zone`. E2E, también la que quedó de 7c2c: al descontinuar, el producto deja de salir en la lista y vuelve con «Ver descontinuados», en gris y con su insignia (revisar la captura). Sin htmx: al terminar regresa a la edición con la búsqueda. Sin confirmación (el motivo obligatorio ya detiene). Con `descontinuados` en la dirección, sea cual sea su valor (también `0`), la casilla cuenta como marcada: así son las casillas de HTML y la pantalla siempre manda `1` (dueño, 2026-10-11).
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
