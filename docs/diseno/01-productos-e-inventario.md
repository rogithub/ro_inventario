# Diseño 01 — Productos e inventario

**Estado:** propuesta para revisión del dueño. Todavía no es SQL: primero se acuerda el modelo, después se escribe la migración. Los servicios tienen su propio diseño: [03 — Servicios](03-servicios.md).

## Lo que ya se decidió (2026-09-29)
- **Kárdex único:** todo lo que mueve inventario escribe renglones en una sola tabla de movimientos. El stock y el costo salen de ahí.
- **Artículo, producto y servicio:** todo lo que se vende en caja es un **artículo** (NID, nombre, categoría, precio). Un artículo es **producto** (lo físico que se compra y se revende, o se consume en un servicio; lleva stock), **servicio** (lo que se hace) o **kit** (se vende como varias partidas juntas); servicios y kits en el diseño 03. Lo común vive en `articulos`; lo propio, en `productos` o `servicios`. *(Cambio del 2026-09-29: antes era una sola tabla con un campo de tipo, y hubo un tercer tipo, kit.)*
- **Una categoría por artículo.**
- **Los nombres del catálogo tienen hasta 150 caracteres** (categorías, unidades, artículos, presentaciones y los catálogos que vengan después): el área lo valida con su mensaje (`kernel::nombres::MAX_NOMBRE_CATALOGO`) y la tabla lo repite con un `CHECK`. El nombre más largo de la v1 tiene 68. *(Decidido el 2026-10-01.)*
- **Códigos de barras en su propia tabla**, apuntando al producto o a una de sus presentaciones.
- **Un artículo se descontinúa, no se borra**, y queda anotado por qué (el proveedor ya no lo vende, ya no se resurte).
- **Categorías:** cada producto conserva su categoría original; las que queden sin artículos no se migran. El intento de recategorizar con IA se abandonó y no se toma en cuenta.
- **Las presentaciones son para vender por mayoreo:** paquetes más grandes con precio de descuento, para los clientes que empiezan a preguntar por volumen (la mayoría compra por pieza).

## Datos de la versión uno que guiaron el diseño
- 2,336 productos: 109 servicios, 5 "kits", 1 de precio libre. **Los 5 kits son en realidad servicios que consumen material** (ver diseño 03). 97 % se venden por pieza, 64 por metro, 2 por gramos.
- 2,335 de 2,336 productos tienen una sola categoría (275 categorías; 93 con un solo producto).
- Solo la hoja (NID 40) usa presentaciones: paquetes de 20, 50, 100 y 300.
- Los códigos de barras vivían en tres lugares: `CodigoBarrasItem` (el producto suelto), `CodigoBarrasCaja` (el empaque del proveedor, p. ej. el tubo de lápices Mirado) y en cada presentación. Lo que no traía código se etiquetaba con un código generado del NID.
- Las 244 ventas de "piezas" con decimales son todas la comisión de tarjeta, registrada como producto de $1 con cantidad = monto. En la versión dos la comisión será un concepto propio (se diseña con ventas).

## Tablas propuestas

### `articulos` (lo que se vende en caja)
| Columna | Qué es |
|---|---|
| `id` | Identificador interno (UUID). |
| `nid` | Número corto para buscarlo en caja y para la etiqueta; se asigna solo, consecutivo, único entre productos y servicios. Los NID de la versión uno se conservan al migrar. |
| `nombre` | Nombre del artículo. |
| `kind` | `producto`, `servicio` o `kit`. Un producto tiene su renglón en `productos`; un servicio, en `servicios`; un kit, sus componentes en `kit_componentes` (diseño 03). |
| `categoria_id` | Su categoría (obligatoria). |
| `unidad_medida_id` | Unidad en que se vende: para un producto es su **unidad base** (en ella se cuenta el stock y se calcula el costo: pieza, hoja, metro…); para un servicio, en qué se cobra (hoja, hora, página…). |
| `precio_venta` | Precio actual de una unidad. |
| `descripcion` | Texto para el catálogo; opcional. |
| `descontinuado_at` | Cuándo se descontinuó. Vacío = se sigue manejando. Un descontinuado no se ofrece en caja ni en el catálogo, y no aparece en los reportes de resurtido; su historia se conserva. |
| `motivo_descontinuado` | Por qué ya no se maneja ("el proveedor ya no lo vende"). Obligatorio al descontinuar. |
| `created_at`, `updated_at`, `updated_by` | Auditoría. |

### `productos` (lo propio de lo físico)
| Columna | Qué es |
|---|---|
| `articulo_id` | El artículo que es este producto. |
| `marca`, `modelo`, `color` | Datos del catálogo; opcionales. |

Lo demás de un producto vive en sus propias tablas: presentaciones, códigos de barras, kárdex y ubicaciones (dónde puede estar: mostrador, bodeguita, bodega; diseño 08).

### `unidades_medida`
| Columna | Qué es |
|---|---|
| `id`, `nombre` | Pieza, hoja, metro, gramos, hora… |
| `allows_fraction` | Si se puede vender 1.5 (metro, gramos) o solo enteros (pieza). La caja lo valida. |

### `categorias`
`id`, `nombre` (único). Plana, sin jerarquía. Productos y servicios comparten la tabla.

### `presentaciones`
Formas de vender (y, si hace falta, de comprar) un producto en grupo. **Su uso principal es el mayoreo:** un paquete con precio de descuento frente a la suma de piezas sueltas.
| Columna | Qué es |
|---|---|
| `id`, `producto_id`, `nombre` | "Paquete de 100 hojas", "Caja con 12". |
| `factor` | Cuántas unidades base trae (100, 12, 5000). |
| `precio_venta` | Precio de la presentación completa, normalmente menor que `factor × precio de la pieza`. Vacío = solo se usa para comprar (hoy no hay ninguna así, pero el caso existe). |

### `codigos_barras`
| Columna | Qué es |
|---|---|
| `codigo` | El código (único en todo el negocio). |
| `producto_id` | Producto al que pertenece. |
| `presentacion_id` | Vacío = el producto suelto. Con valor = esa presentación (el tubo de Mirado). |

Un producto puede tener varios códigos (el proveedor cambió el empaque). El código generado del NID no se guarda: la etiqueta imprime el NID y al escanearlo se busca por NID.

### `precios_historial`
`articulo_id`, `presentacion_id` (opcional), `precio_venta`, `desde`, `updated_by`. Se escribe cada vez que cambia un precio. Sirve para reportes; la venta guarda de todas formas el precio con que se cobró.

### `movimientos_inventario` (el kárdex)
| Columna | Qué es |
|---|---|
| `id`, `producto_id`, `fecha` | |
| `kind` | `compra`, `venta`, `cancelacion` (regresa lo de una venta cancelada), `devolucion`, `merma`, `ingreso_sin_compra`. |
| `cantidad` | En **unidad base**, con signo: + entra, − sale. Una compra de 2 cajas de 5,000 hojas escribe +10,000. |
| `precio_unitario` | Solo en compras: lo que costó cada unidad base. De aquí sale el costo promedio. |
| `recepcion_partida_id`, `venta_partida_id`, `devolucion_partida_id`, `ajuste_partida_id` | El documento que lo originó; exactamente uno tiene valor. Las ventas y cancelaciones apuntan a la partida vendida (la de un servicio con receta, para sus insumos). Los ajustes (merma, ingreso sin compra, conteo físico) están en el diseño 08. |

- **Solo productos escriben al kárdex.** Un servicio no; al venderlo, cada uno de sus insumos escribe su salida (con referencia al servicio vendido; ver diseño 03).
- **Los renglones no se editan ni se borran:** una corrección es otro movimiento. Es la historia del inventario. (Única excepción controlada: fusionar productos duplicados, ver diseño 02.)
- **Stock = suma de `cantidad` por producto.**
- **El costo por promedio móvil** lo sigue calculando la aplicación y lo guarda en tablas derivadas regenerables, como en la versión uno (`/home/ro/code/inventario_papeleria/docs/decisiones/2026-09-27-costo-promedio-movil.md`). El kárdex es el dato original; el costo, el calculado.

## Cuándo un producto entra al inventario
Un producto nuevo pasa por dos etapas, que salen de los datos (no hay un campo de estado):
- **Por llegar:** se dio de alta al capturar una compra, pero todavía no llega o no tiene precio de venta final. Solo se ve en esa compra; no aparece en caja, ni en inventario, ni en reportes.
- **En catálogo:** ya tuvo su primera recepción (hay movimientos en el kárdex) **y** tiene precio de venta final. Aparece en la página de inventario (con o sin stock) y en caja mientras tenga stock.

Al guardar la primera recepción de un producto nuevo, el precio de venta es obligatorio (ver diseño 02).

## Fuera de este diseño
- **Escalas de precio por cantidad** (p. ej. 1–99 hojas a $0.50, 100+ a $0.40): el mayoreo se resuelve con presentaciones; las escalas solo se agregan si un caso real no cabe en un paquete. El precio de una partida se calcula en una sola función pura, así que agregarlas después es un cambio en un solo lugar.
- **El margen del mayoreo:** como el costo siempre va por unidad base, el margen de un paquete es su precio menos `factor × costo unitario`. Los reportes lo mostrarán por presentación.
- **Fotos, videos y destacados:** van con el catálogo público (aplicación pública), no con el inventario.
- **El nombre del producto según el proveedor:** está en el diseño 02 (compras).
- **Comisión de tarjeta:** se diseña con ventas.
- **Servicios** (recetas, costo externo, precio libre): diseño 03.
- **Kits** (artículos que se venden como varias partidas juntas: "Pago de servicio", "CURP impreso"): diseño 03.

## Respuestas del dueño (2026-09-29)
1. **Descontinuar sí hace falta,** con el motivo: hay productos sin stock que nunca se van a resurtir (el proveedor ya no los vende).
2. **Categorías: se limpian** tomando la categoría original de cada producto.
3. **Presentaciones:** hoy casi todo se vende por pieza (la zona es de bajos recursos), pero algunos clientes ya preguntan por mayoreo. Las presentaciones son para ir armando paquetes más grandes con descuento.
4. **Artículo + dos tablas:** lo común de productos y servicios en `articulos`; lo propio, en `productos` y `servicios`.
