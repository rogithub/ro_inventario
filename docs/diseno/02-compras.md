# Diseño 02 — Compras

**Estado:** propuesta para revisión del dueño. Depende de [01 — Productos e inventario](01-productos-e-inventario.md).

## Lo que ya se decidió (2026-09-29)
- **Un solo documento, la compra, que contiene una o varias recepciones.** Nace como pedido al proveedor; cada entrega que llega es una recepción. Así se distingue lo pedido de lo recibido (sustituciones, cantidades distintas) y se aceptan pedidos que llegan en varias entregas (común en Mercado Libre), sin tener "orden de compra" y "compra" como documentos sueltos.
- **Los pagos al proveedor viven dentro de la compra:** varios pagos o ninguno; de ahí sale cuánto se le debe a cada proveedor.
- **Paquetería a mano por partida**, con una sugerencia calculada (repartir por pieza). Cada quien decide a qué le carga más; no se fuerza una estrategia.
- **Solo se guarda el precio cobrado,** no el acordado.
- **Un producto nuevo existe desde que se captura el pedido, pero entra al inventario hasta que llega y tiene precio de venta final** (ver "Cuándo un producto entra al inventario" en el diseño 01). Al guardar su primera recepción, el precio de venta es obligatorio.
- **Proveedores y clientes en tablas separadas:** a futuro cada uno podría tener su propio portal (clientes: el sitio público; proveedores: otro aparte).
- **Fecha del kárdex = fecha de llegada.**
- **Fusionar duplicados reescribiendo la historia (opción A):** sobrevive el producto con más historia; el sistema lo sugiere y el dueño confirma.
- **Todo con IVA incluido,** también la paquetería: hoy la papelería no factura, así que ese IVA es costo. Separar el IVA acreditable (de la mercancía y del envío) llega con la facturación.

## Datos de la versión uno que guiaron el diseño
- 279 compras desde abril de 2022, 46 proveedores. Desde mayo de 2024, el 100 % pasa por una orden de compra (161 de 161), siempre un pedido = una compra.
- 594 partidas de orden **crearon el producto ahí mismo**: dar de alta al comprar es el flujo normal.
- 891 partidas usan "unidades por paquete" (el proveedor vende paquetes de 12, cajas de 100…).
- La paquetería se sumaba al costo de cada pieza (costo real puesto en tienda). 69 compras la tienen.
- La forma de pago de la compra nunca se llenó (las 279 dicen "efectivo"); los pagos se capturaban en Finanzas.
- El estado de la orden se guardaba **y** se recalculaba de sus fechas: dos fuentes para lo mismo.
- Los servicios se "compraban" con cantidad 1,000,000 para simular stock infinito (por eso aparecían con stock 999,993). En la versión dos un servicio no lleva stock.
- Hay proveedores duplicados ("Dayana Cartagena" con y sin espacio final): se limpian al migrar.
- Hay **proveedores inventados** para poder dar de alta servicios: en la v1 un servicio necesitaba una compra (y la compra un proveedor) para existir con costo. En la v2 un servicio no necesita compra ni proveedor; esos proveedores no se migran.
- A veces el proveedor manda unas cosas por otras o en cantidades distintas a las pedidas.

## Tablas propuestas

### `proveedores`
`id`, `nombre`, `empresa`, `telefono`, `email`, `notas`, `created_at`, `updated_by`. Incluye tiendas en línea (Mercado Libre, TEMU). El RFC y la conexión con otros nodos de la red llegan después.

### `compras` (el pedido)
| Columna | Qué es |
|---|---|
| `id`, `folio` | El folio es un número consecutivo para hablar de la compra ("la compra 312"). |
| `proveedor_id` | |
| `pedida_at`, `enviada_at`, `cerrada_at`, `cancelada_at` | Fechas de cada paso. `cerrada_at`: lo que no llegó ya no va a llegar. |
| `notas`, `created_at`, `updated_by` | |

**Estado de entrega** (calculado de las fechas y las recepciones; una sola fuente): pedida → enviada → recibida parcialmente → recibida (todo lo pedido llegó, o se cerró) — o cancelada.
**Estado de pago** (calculado aparte): pendiente, parcial o pagada, comparando los pagos contra lo recibido.

### `compras_partidas` (lo pedido)
La lista que se le manda al proveedor. **Se congela al marcar la compra como enviada.**
| Columna | Qué es |
|---|---|
| `id`, `compra_id`, `orden` | `orden` = posición en la lista. |
| `producto_id` | Siempre apunta a un producto; si es nuevo, se da de alta al capturar el pedido y queda "en pedido" hasta su primera recepción. |
| `nombre_proveedor` | Cómo lo llama el proveedor. |
| `cantidad_proveedor`, `unidades_por_paquete` | "3 paquetes de 12". |

### `recepciones` (cada entrega)
| Columna | Qué es |
|---|---|
| `id`, `compra_id`, `recibida_at` | Cuándo llegó. Es la fecha que usa el kárdex. |
| `factura_date`, `total_factura` | Del ticket o factura de esa entrega; el total se compara contra la suma de sus partidas para detectar errores de captura. |
| `costo_paqueteria`, `nombre_paqueteria`, `numero_rastreo` | El envío de esa entrega. |
| `notas`, `updated_by` | |

### `recepciones_partidas` (lo que llegó)
| Columna | Qué es |
|---|---|
| `id`, `recepcion_id` | |
| `compra_partida_id` | La partida pedida a la que corresponde. Vacío = llegó sin pedirse. |
| `sustituye_partida_id` | Si llegó en lugar de otra partida (cartulina azul en lugar de roja). |
| `producto_id`, `nombre_proveedor` | Lo que de verdad llegó. |
| `cantidad_proveedor`, `unidades_por_paquete`, `precio_proveedor` | Lo que llegó y lo que se cobró: "3 paquetes de 12 a $45". |
| `paqueteria` | Parte del envío que se le carga (a mano, con sugerencia). |
| `is_regalo` | Vino sin costo: entra con costo 0. |
| `motivo_diferencia` | Si llegó distinto a lo pedido o dañado, por qué. |
| `precio_venta_nuevo` | Si con esta compra cambia el precio de venta del producto (queda en su historial). **Obligatorio en la primera recepción de un producto nuevo.** |

**En el caso normal (una sola entrega igual a lo pedido),** la recepción se prellena con las partidas pedidas y solo se corrige lo que cambió: para quien captura se siente como un solo paso.

**Compra de mostrador** (se pide, se paga y se recibe en el mismo momento; p. ej. un paquete de tijeras en oferta en Home Depot que se revende por pieza): una sola pantalla que captura el ticket y, al guardar, crea la compra, sus partidas, una recepción idéntica, el pago y el cierre, todo con la misma fecha y hora. No hay que pasar por pedida → enviada → recibida.

**Paquetería opcional:** vale cero si la compra no tiene envío; el costo por pieza queda en `precio ÷ unidades del paquete`.

**Cálculos** (en código puro, con pruebas):
- Cantidad en unidad base = `cantidad_proveedor × unidades_por_paquete`.
- Costo por unidad base = `(precio_proveedor ÷ unidades_por_paquete) + (paqueteria ÷ cantidad en unidad base)`.
- Sugerencia de paquetería por partida = `costo_paqueteria × (piezas de la partida ÷ piezas de toda la recepción)`.
- Diferencias con lo pedido: por partida, pedido contra la suma de lo recibido, más lo que llegó sin pedirse. Es la evidencia para reclamarle al proveedor.

### `compras_pagos`
`id`, `compra_id`, `fecha`, `forma_pago` (efectivo, transferencia, tarjeta…), `monto`, `referencia`, `notas`, `updated_by`. Muchas compras se pagan antes de que lleguen.

### `compras_documentos`
`id`, `compra_id`, `recepcion_id` (opcional), `archivo`: facturas, tickets y fotos de la nota, guardados en MinIO.

### Lo que el proveedor sabe de cada producto (vista, no tabla)
De las recepciones anteriores sale, por proveedor y producto: cómo lo llama, en qué paquete lo vende y a qué precio lo cobró la última vez. Sirve para prellenar el siguiente pedido y es la semilla de la cadena de suministro. Es una vista: no hay datos que mantener a mano.

## Cómo entra al inventario
- **Al guardar una recepción,** cada partida de producto físico escribe un movimiento `compra` al kárdex: la cantidad en unidad base, con su costo por unidad base.
- **Fecha del movimiento = `recibida_at`**, que se puede corregir si se captura tarde. Es la fecha real en que hubo mercancía en la tienda: así el costo promedio no queda "sin costo" por capturar tarde.
- **Partidas de servicio** (p. ej. el costo de un pago de servicios) no escriben al kárdex; dan el costo de referencia del servicio.
- **Una recepción guardada no se edita:** un error se corrige con un ajuste de inventario o una devolución al proveedor (se diseña con ajustes).

## Evitar productos duplicados
En la versión uno pasó seguido: se compra cartulina (NID 52), otro proveedor la llama "cartulina de tantos gramos", y quien captura la da de alta como producto nuevo en lugar de resurtir. Se corrigió con scripts de deduplicación y advertencias al capturar. Aquí la defensa es por capas:

1. **El camino normal es buscar, no crear.** Al capturar una partida se escribe el nombre como viene en la nota, y el sistema propone el producto existente:
   - por la historia del proveedor (ya lo llamó así antes);
   - por código de barras (coincide = es ese producto; el código es único, ahí el duplicado es imposible);
   - por parecido del nombre (búsqueda por similitud de trigramas, como el buscador de la v1; sin IA).

   "Crear producto nuevo" aparece después de ver los parecidos. El nombre del proveedor queda ligado al producto, así que la siguiente vez ya no hay que decidir.
2. **Reporte de posibles duplicados** (nombre parecido, misma categoría, precio parecido) para revisar de vez en cuando. Solo sugiere.
3. **Operación "fusionar productos"** para corregir sin scripts: todo lo del duplicado (ventas, compras, kárdex, códigos de barras, nombres del proveedor) pasa al que sobrevive, como si siempre hubiera sido uno. Sobrevive el de más historia (más ventas y movimientos); si el duplicado se acaba de crear, se sacrifica el más nuevo. El sistema sugiere y el dueño confirma. El NID del sacrificado queda como alias (las etiquetas impresas siguen funcionando). Es la única excepción controlada a "el kárdex no se edita", y queda registrado quién fusionó, cuándo y qué.
4. **Advertencias al capturar**, como en la v1, como apoyo.

## Fuera de este diseño
- Devoluciones al proveedor.
- IVA acreditable (de mercancía y de paquetería) y facturas recibidas del SAT: llegan con facturación.
- Pedidos automáticos a otros nodos de la red.

## Respuestas del dueño (2026-09-29)
1. **Producto nuevo:** comienza a ser stock hasta que se recibe y se conoce su precio de venta final (con la ganancia), su costo de compra y su paquetería. Antes de eso no se reporta en el inventario. Las páginas de venta solo muestran lo que tiene stock; una página aparte de inventario muestra y busca con y sin stock.
2. **Proveedores y clientes separados:** estuvieron juntos y se separaron pensando en que a futuro los clientes tengan acceso a su información (hoy con enlaces opacos en xplaya.com, p. ej. el del monedero) y los proveedores, en otro sitio aparte. Tablas separadas es lo más limpio.
3. **Fecha de llegada** para el kárdex; además, el producto no se usa hasta tener su precio de venta final.
4. **Fusión opción A:** sobrevive el producto con más historia; si el duplicado es reciente, se sacrifica el más nuevo.
