# Diseño 01 — Productos e inventario

**Estado:** propuesta para revisión del dueño. Todavía no es SQL: primero se acuerda el modelo, después se escribe la migración.

## Lo que ya se decidió (2026-09-29)
- **Kárdex único:** todo lo que mueve inventario escribe renglones en una sola tabla de movimientos. El stock y el costo salen de ahí.
- **Una categoría por producto.**
- **Códigos de barras en su propia tabla**, apuntando al producto o a una de sus presentaciones.
- **Un campo de tipo con dos valores:** producto (lo que se vende tal cual; lleva stock) o servicio (lo que se hace). *(Cambio del 2026-09-29: antes había un tercer tipo, kit; ver "Servicios con receta".)*
- **Un servicio puede tener una receta:** los insumos que consume (la hoja de una copia, la mica de un enmicado) y otros servicios que incluye (la impresión y el corte de una polaroid). Los servicios se dan de alta en el catálogo, nunca en compras.
- **Un producto se descontinúa, no se borra**, y queda anotado por qué (el proveedor ya no lo vende, ya no se resurte).
- **Categorías:** cada producto conserva su categoría original; las que queden sin productos no se migran. El intento de recategorizar con IA se abandonó y no se toma en cuenta.
- **Las presentaciones son para vender por mayoreo:** paquetes más grandes con precio de descuento, para los clientes que empiezan a preguntar por volumen (la mayoría compra por pieza).

## Datos de la versión uno que guiaron el diseño
- 2,336 productos: 109 servicios, 5 "kits", 1 de precio libre. **Los 5 kits son en realidad servicios que consumen material** (enmicado = servicio + mica; polaroid = impresión + corte + papel fotográfico; impresión en folder = impresión + folder). 97 % se venden por pieza, 64 por metro, 2 por gramos.
- 2,335 de 2,336 productos tienen una sola categoría (275 categorías; 93 con un solo producto).
- Solo la hoja (NID 40) usa presentaciones: paquetes de 20, 50, 100 y 300.
- Los códigos de barras vivían en tres lugares: `CodigoBarrasItem` (el producto suelto), `CodigoBarrasCaja` (el empaque del proveedor, p. ej. el tubo de lápices Mirado) y en cada presentación. Lo que no traía código se etiquetaba con un código generado del NID.
- Las 244 ventas de "piezas" con decimales son todas la comisión de tarjeta, registrada como producto de $1 con cantidad = monto. En la versión dos la comisión será un concepto propio (se diseña con ventas).

## Tablas propuestas

### `productos`
| Columna | Qué es |
|---|---|
| `id` | Identificador interno (UUID). |
| `nid` | Número corto para buscarlo en caja y para la etiqueta; se asigna solo, consecutivo. Los NID de la versión uno se conservan al migrar. |
| `nombre` | Nombre del producto. |
| `kind` | `producto` o `servicio`. |
| `categoria_id` | Su categoría (obligatoria). |
| `unidad_medida_id` | La **unidad base**: en ella se cuenta el stock y se calcula el costo (pieza, hoja, metro…). |
| `precio_venta` | Precio actual de una unidad base. |
| `is_precio_libre` | Solo servicios: precio y descripción se capturan en cada venta. |
| `marca`, `modelo`, `color`, `descripcion` | Datos del catálogo; opcionales. |
| `descontinuado_at` | Cuándo se descontinuó. Vacío = se sigue manejando. Un descontinuado no se ofrece en caja ni en el catálogo, y no aparece en los reportes de resurtido; su historia se conserva. |
| `motivo_descontinuado` | Por qué ya no se resurte ("el proveedor ya no lo vende"). Obligatorio al descontinuar. |
| `created_at`, `updated_at`, `updated_by` | Auditoría. |

### `unidades_medida`
| Columna | Qué es |
|---|---|
| `id`, `nombre` | Pieza, hoja, metro, gramos… |
| `allows_fraction` | Si se puede vender 1.5 (metro, gramos) o solo enteros (pieza). La caja lo valida. |

### `categorias`
`id`, `nombre` (único). Plana, sin jerarquía.

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

### `recetas` (servicios con receta)
`servicio_id`, `componente_id`, `cantidad`. Lo que un servicio consume o incluye:
- **Insumos:** productos físicos que se gastan al hacerlo (copia carta BN → 1 hoja carta; enmicado carta → 1 mica carta). Al vender el servicio, cada insumo escribe su salida al kárdex. El ticket solo muestra el servicio.
- **Servicios incluidos:** otros servicios que forman parte (polaroid → impresión color + corte). Sirven para el costo y para contar cuántas veces se usó cada servicio.

Con esto:
- **Disponibilidad:** un servicio sin receta (tramitar un CURP) siempre está disponible; uno con insumos, mientras alcancen (el menor de stock ÷ cantidad entre sus insumos, como los kits de la v1).
- **Costo del servicio** = el de su receta. Sin receta, un costo fijo (0 para el CURP) o trasladado (recargas: cuesta lo que se cobra). El desgaste de la impresora llega con el módulo de impresoras.
- **Un "kit" de verdad** (productos que se venden juntos en paquete, p. ej. cuaderno + lápiz + goma) hoy no existe; entra cuando haya un caso real.

### `precios_historial`
`producto_id`, `presentacion_id` (opcional), `precio_venta`, `desde`, `updated_by`. Se escribe cada vez que cambia un precio. Sirve para reportes; la venta guarda de todas formas el precio con que se cobró.

### `movimientos_inventario` (el kárdex)
| Columna | Qué es |
|---|---|
| `id`, `producto_id`, `fecha` | |
| `kind` | `compra`, `venta`, `devolucion`, `merma`, `ingreso_sin_compra`. |
| `cantidad` | En **unidad base**, con signo: + entra, − sale. Una compra de 2 cajas de 5,000 hojas escribe +10,000. |
| `precio_unitario` | Solo en compras: lo que costó cada unidad base. De aquí sale el costo promedio. |
| referencia al documento | El renglón de venta, compra o ajuste que lo originó. Se define con esos módulos. |

- **Solo productos físicos escriben al kárdex.** Un servicio no; al venderlo, cada uno de sus insumos escribe su salida (con referencia al servicio vendido).
- **Los renglones no se editan ni se borran:** una corrección es otro movimiento. Es la historia del inventario.
- **Stock = suma de `cantidad` por producto.**
- **El costo por promedio móvil** lo sigue calculando la aplicación y lo guarda en tablas derivadas regenerables, como en la versión uno (`/home/ro/code/inventario_papeleria/docs/decisiones/2026-09-27-costo-promedio-movil.md`). El kárdex es el dato original; el costo, el calculado.

## Cuándo un producto entra al inventario
Un producto nuevo pasa por dos etapas, que salen de los datos (no hay un campo de estado):
- **En pedido:** se dio de alta al capturar una compra, pero todavía no llega o no tiene precio de venta final. Solo se ve en esa compra; no aparece en caja, ni en inventario, ni en reportes.
- **En catálogo:** ya tuvo su primera recepción (hay movimientos en el kárdex) **y** tiene precio de venta final. Aparece en la página de inventario (con o sin stock) y en caja mientras tenga stock.

Al guardar la primera recepción de un producto nuevo, el precio de venta es obligatorio (ver diseño 02).

## Fuera de este diseño
- **Escalas de precio por cantidad** (p. ej. 1–99 hojas a $0.50, 100+ a $0.40): el mayoreo se resuelve con presentaciones; las escalas solo se agregan si un caso real no cabe en un paquete. El precio de un renglón se calcula en una sola función pura, así que agregarlas después es un cambio en un solo lugar.
- **El margen del mayoreo:** como el costo siempre va por unidad base, el margen de un paquete es su precio menos `factor × costo unitario`. Los reportes lo mostrarán por presentación.
- **Fotos, videos y destacados:** van con el catálogo público (aplicación pública), no con el inventario.
- **El nombre del producto según el proveedor** (distinto en 1,433 renglones de compra): se diseña con compras; es la semilla del "código del proveedor" para la cadena de suministro.
- **Comisión de tarjeta:** se diseña con ventas.
- **El costo de los servicios** (se diseña con servicios e impresoras). Hay cuatro tipos:
  - **sin costo directo** (tramitar un CURP: costo 0, todo es margen);
  - **ingreso trasladado** (pago de servicios, recargas: costo = lo que se cobra);
  - **consume material** (una copia se lleva una hoja: la hoja es su insumo, en su receta);
  - **usa un equipo** (la impresora es una inversión, no mercancía; el costo por copia sale del tóner y del desgaste medido con los contadores SNMP).

  En la v1 una impresora se dio de alta como "compra" de un servicio para ligar su costo a las copias: resolvía una pregunta real con la herramienta disponible. En la v2 la impresora es equipo.
- **"CURP impreso"** (trámite + copia) son dos servicios que se venden juntos de un toque, en dos partidas: el trámite (sin receta) y la copia (con su hoja como insumo).

## Respuestas del dueño (2026-09-29)
1. **Descontinuar sí hace falta,** con el motivo: hay productos sin stock que nunca se van a resurtir (el proveedor ya no los vende).
2. **Categorías: se limpian** tomando la categoría original de cada producto.
3. **Presentaciones:** hoy casi todo se vende por pieza (la zona es de bajos recursos), pero algunos clientes ya preguntan por mayoreo. Las presentaciones son para ir armando paquetes más grandes con descuento.
