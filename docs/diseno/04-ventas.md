# Diseño 04 — Ventas

**Estado:** propuesta para revisión del dueño. Depende de [01 — Productos e inventario](01-productos-e-inventario.md) y [03 — Servicios](03-servicios.md).

## Lo que ya se decidió (2026-09-29)
- **La venta es su propio documento** (folio, partidas, pagos), separado de los ajustes de inventario. En la v1 era un "ajuste" con columnas de pago.
- **Los pagos van en renglones, uno por forma de pago** (catálogo `c_FormaPago` del SAT). Una venta mixta tiene dos pagos; agregar una forma nueva (CoDi) es agregar un valor, no una columna.
- **Dólares desde el inicio:** Playa del Carmen es destino turístico. Un pago en dólares es un pago en **efectivo con moneda USD** y su tipo de cambio; **el cambio siempre se da en pesos**.
- **Tipo de cambio = FIX de Banxico del día − spread fijo en pesos**, como en la v1 (`SPREAD_TIPO_CAMBIO_DOLARES`). El spread lo ajusta el dueño en pantalla (ajustes de la BD) cuando cambia lo que pagan los bancos de la zona. La venta guarda el tipo de cambio que se aplicó.
- **La forma del reembolso es configurable** (`negocio.toml`): `efectivo` (hoy: el negocio no está dado de alta en Hacienda y conviene minimizar movimientos con tarjeta y transferencia) o `misma_forma` (cuando se registre: ventaja contable, como las tiendas grandes). La pantalla de devolución preselecciona según esa opción.
- **Comisión por pago con tarjeta = cargo ligado al pago**, no un artículo. Se calcula como hoy (iterativa, con IVA) y aparece en el ticket como cargo. Adiós al truco de $1 × cantidad.
- **Cada partida guarda lo de ese momento:** precio cobrado, costo externo (con su ganancia) y de qué kit vino. El costo de la mercancía se sigue calculando del kárdex.
- **Una venta cobrada no se edita: se cancela (con motivo) y se recaptura.** (Un pedido o cotización sin cobrar sí se edita.) La cancelación regresa el inventario sola, en un paso. Hoy se hace a mano como devolución y es tedioso.
- **La devolución queda para cuando el cliente regresa mercancía.**
- **Fecha editable** para capturar ventas atrasadas; queda registro de cuándo se capturó.
- **El monedero va en el corte mínimo:** ya se anunció y los clientes cuentan con él (diseño 06).
- **Pedidos y cotizaciones usan el mismo mecanismo, con propósitos distintos** (ver "Pedidos y cotizaciones"): el pedido se va a recoger; la cotización son solo precios y tal vez nunca se compre. Los dos son ventas sin cobrar que se pueden editar hasta cobrarse. Es un flujo diario (WhatsApp), así que va en el corte mínimo. Ninguno se borra: al cobrarse se vuelven una venta; una cotización que no se compra vence y se archiva.

## Datos de la versión uno que guiaron el diseño (últimos 12 meses)
- ~17 tickets al día, 1.9 partidas por ticket, **ticket mediano de $18** (promedio $37). Horas pico: 11–13 h y 17 h. La caja tiene que ser rápida para tickets chicos.
- Tickets por forma de pago: efectivo 5,824; transferencia 195; tarjeta 166; dólares 3 (históricamente se convertían a pesos a mano y se guardaban como pesos, así que el número real es mayor); monedero 2. Pagos mixtos: 15 tickets.
- 28 % de los tickets llevan cliente. El monedero se ha generado 1,919 veces y canjeado 26 en toda la historia. **Ojo: el número engaña.** Se lanzó con cautela y se anunció hace poco; ya hay clientes siguiendo su acumulado por el enlace de xplaya.com, y una de ellas regresó a comprar después de consultarlo.
- 122 partidas devueltas en 50 días distintos.
- Pedidos: la tabla tiene 13 porque **en la v1 un pedido se borra al entregarse**; solo quedan los pendientes. El flujo es diario: el cliente escribe por WhatsApp ("imprímeme estos documentos"), se captura la venta y se guarda sin cobrar; cuando llega con su número de pedido o su teléfono, se carga, se cobra y se entrega. Si no se conoce al cliente, se usaba un cliente anónimo fijo (`ID_CLIENTE_GENERICO`).

## Tablas propuestas

### `ventas`
| Columna | Qué es |
|---|---|
| `id`, `folio` | Folio consecutivo del ticket. |
| `cliente_id` | Opcional. En la v2 no hay "cliente anónimo": sin cliente es sin cliente (ver diseño 06). |
| `contacto_nombre`, `contacto_telefono` | Opcionales, para pedidos y cotizaciones **sin cliente registrado** (un extraño, o no se tienen sus datos a la mano): sirven para encontrar el pedido ("mi teléfono es tal"). No generan monedero. |
| `kind` | Cómo empezó: `mostrador` (venta normal en caja), `pedido` o `cotizacion`. Al cobrarse, cualquiera es una venta; el tipo solo dice su origen. |
| `cambio` | Lo que se le regresó al cliente, en pesos. |
| `cobrada_at` | Cuándo se cobró: **es la fecha de la venta** (la de los reportes y del kárdex). Se puede poner una anterior al capturar ventas atrasadas; si difiere de `created_at` (cuándo se capturó), se sabe que se capturó después. Vacía = pedido o cotización sin cobrar. |
| `entregada_at` | Cuándo se entregó. Vacío = pedido por entregar. En una venta normal de caja, las dos fechas son el mismo momento. |
| `cancelada_at`, `motivo_cancelacion`, `cancelada_by` | Si se canceló. Una cancelada no cuenta en reportes ni en caja, pero se conserva. |
| `punto_venta` | Desde dónde llegó o se cobró: computadora, iPad, Elo o **sitio público** (pedidos en línea desde xplaya.com). Sirve para investigar diferencias en el corte de caja. En lo que llega del sitio público, `created_by` queda vacío: no lo creó una persona. |
| `notas`, `created_at`, `created_by` | `created_by` = quien vendió. |

### `ventas_partidas`
| Columna | Qué es |
|---|---|
| `id`, `venta_id`, `orden` | |
| `articulo_id` | Lo vendido (producto o servicio). |
| `presentacion_id` | Si se vendió en paquete (mayoreo). |
| `descripcion` | Solo en precio libre: "Pago de CFE", "Concepto editable…". |
| `cantidad`, `precio_unitario` | Lo vendido y a qué precio (el de ese momento). |
| `costo_externo` | Copiado del servicio al vender: lo que se le paga a un tercero. Los trasladados guardan aquí lo cobrado. |
| `kit_articulo_id`, `kit_grupo` | Si la partida vino de un kit: cuál y en qué grupo (dos "Pago de servicio" en el mismo ticket son dos grupos). |

### `ventas_pagos`
| Columna | Qué es |
|---|---|
| `id`, `venta_id` | |
| `forma_pago` | Efectivo, tarjeta, transferencia… (catálogo del SAT). |
| `moneda`, `monto_moneda`, `tipo_cambio` | Para dólares: moneda USD, cuántos dólares entregó y a qué tipo de cambio. En pesos, `monto_moneda` = `monto` y el tipo de cambio es 1. |
| `monto` | Lo que este pago cubre de la venta, en pesos. |
| `comision` | Solo tarjeta: el cargo que se le cobra al cliente por pagar así. |
| `referencia` | Folio de la transferencia, últimos dígitos de la tarjeta… (opcional). |

**El dinero se calcula en un solo lugar: el servidor.** En la v1 la comisión de tarjeta se calcula en JavaScript (punto flotante) y se guarda con decimales exactos: a veces difiere uno o dos centavos. Es el mismo tipo de inconsistencia que llevó a dejar SQLite por Postgres. En la v2:
- Totales, comisión, cambio y máximo del monedero son funciones puras del núcleo del negocio, con `Decimal` y pruebas.
- La pantalla solo muestra lo que el servidor calculó (le pregunta mientras se arma la venta); al guardar, el servidor recalcula todo y nunca toma como verdad lo que manda la pantalla.
- El redondeo es una regla escrita (a centavos, en qué pasos y hacia dónde), no un efecto del lenguaje.

**Reglas** (en código puro, con pruebas; se llevan de la v1):
- **Total de la venta** = suma de partidas + comisiones.
- **Comisión de tarjeta** = iterativa: se calcula sobre el monto con tarjeta, y otra vez sobre ese monto + la primera comisión, con IVA; la tasa sale de los ajustes de la BD (la cambia el dueño).
- **Solo el efectivo (pesos o dólares) da cambio.** Tarjeta y transferencia son por el monto exacto.
- La suma de pagos (en pesos) menos el cambio debe ser igual al total.

## Cómo afecta al inventario
Al **cobrar** la venta (en caja es al guardarla; en un pedido, cuando se cobra), en la misma transacción:
- **Producto:** escribe su salida al kárdex (en unidad base; un paquete de 100 hojas saca 100).
- **Servicio con receta:** cada insumo escribe su salida.
- **Kit:** cada partida que agregó se comporta según su tipo.
- **Servicio sin receta:** nada.

## Pedidos y cotizaciones
En la v1 son conceptos distintos que viven en la misma tabla y con el mismo enlace. En la v2 siguen compartiendo mecanismo (una venta sin cobrar), pero cada uno tiene su nombre:
- **Pedido:** el cliente se comprometió a recogerlo (mandó por WhatsApp qué imprimir). Aparece en la lista de **pedidos por entregar**.
- **Cotización:** solo son precios ("vecina, cotíceme esta lista de útiles escolares"); tal vez nunca venga. Aparece en la lista de **cotizaciones**; su página pública dice "Cotización" y ofrece el PDF.
- **Hasta cobrarse es un borrador editable:** al recoger su pedido de un cuaderno, el cliente pide también una copia de su INE; se carga, se agrega y se cobra. La regla "una venta no se edita, se cancela" aplica solo a las **ya cobradas**.
- **Precio:** cada partida guarda el precio con que se capturó (lo que el cliente vio en su enlace). Si al cobrar el precio vigente es distinto, la caja lo avisa y el vendedor decide si respeta el cotizado o cobra el nuevo.
- **Una cotización que no se compra vence sola** a los N días (ajuste configurable) y se archiva con motivo "vencida": no afecta inventario ni reportes, pero no se pierde. Da un dato que hoy no existe: cuántas cotizaciones terminan en venta.

Un pedido o una cotización tiene `cobrada_at` o `entregada_at` vacíos:
| Situación | `cobrada_at` | `entregada_at` |
|---|---|---|
| Venta normal en caja | ahora | ahora |
| Pedido por WhatsApp, pendiente | — | — |
| Pedido pagado por adelantado (transferencia) | cuando pagó | — |
| Pedido recogido | cuando pagó | cuando lo recogió |

- **El número de pedido es el folio de la venta:** el cliente llega con "mi pedido es tal" o con su teléfono. Un pedido se busca por folio, por el teléfono del cliente o por el teléfono de contacto.
- **Se cobra** agregando sus pagos (puede ser en caja, al recoger, o antes por transferencia). Al cobrarse, sale el inventario y se genera su monedero.
- **Al cobrar un pedido no entregado,** la caja pregunta si ya se entrega (como en la v1).
- **Un pedido que nunca se recoge** se cancela con motivo "no lo recogió". Como el inventario sale al cobrar, lo que ya se gastó en prepararlo (las hojas de unas copias) se registra en ese momento como **merma**, para que la pérdida quede en el kárdex y en los reportes. Esos pedidos alimentan la historia del cliente (diseño 06).
- **El pedido también es la cotización** que ve el cliente en el sitio público (`xplaya.com/cotizacion/{id}`, con su tarjeta Open Graph para WhatsApp): antes de pagar sabe qué se le entregará y cuánto costará.
  - El enlace usa el **id interno** de la venta (opaco), nunca el folio: con un número consecutivo cualquiera podría ver el pedido de otro.
  - **El mismo enlace sirve para todo:** `/cotizacion/{id}` muestra la cotización mientras está pendiente y, una vez cobrada, redirige a `/recibo/{id}` (mismo id). En la v1 el enlace muere al cobrarse (el pedido se borra).
  - En la migración, el uid de cada pedido pendiente de la v1 se vuelve el id de su venta, para que los enlaces ya enviados sigan abriendo.
- **La pantalla de caja** muestra los pedidos pendientes para cargarlos rápido (en la v1: el menú de pedidos).

## Cancelar una venta
Un solo paso, con motivo obligatorio:
- **Venta cobrada:** se marca cancelada y se escriben en el kárdex movimientos `cancelacion` que regresan el inventario. En el corte de caja, la cancelación cuenta en el periodo en que se hace; los cortes ya cerrados no cambian (diseño 05). Después se captura la venta correcta.
- **Pedido o cotización sin cobrar:** se marca cancelada; no hay inventario que regresar, porque nunca salió. Solo si se gastó material en prepararlo (pedido no recogido) se registra como merma.

## Devoluciones (el cliente regresa mercancía)
- `devoluciones`: `id`, `venta_id`, `fecha`, `motivo`, `reembolso` (cuánto se le regresó y por qué forma de pago; se preselecciona según la configuración del negocio), `created_by`.
- `devoluciones_partidas`: `venta_partida_id`, `cantidad_buen_estado`, `cantidad_danada`.
- Lo que regresa en buen estado entra al kárdex (`devolucion`); lo dañado entra y sale como merma, para que quede la evidencia.

## Fuera de este diseño
- **Monedero** (generar, canjear, consultar por enlace): diseño 06.
- **Corte de caja:** diseño 05.
- **Comisión que cobra la terminal** (contra la que se le cobra al cliente): con finanzas.
- **Facturación (CFDI) de una venta** y el ticket público con enlace: después.
- **Descuentos** por venta: no existen hoy; el mayoreo va por presentaciones.

## Respuestas del dueño (2026-09-29)
1. **Tipo de cambio:** el de Banxico con spread configurable, como hoy. El spread nació al cobrar en dólares: los clientes decían "Banco Azteca me los paga a tanto", menos que el API. Se aprendió sobre la marcha.
2. **Cambio en dólares:** siempre en pesos.
3. **Reembolso:** depende. Hoy (sin alta en Hacienda, es un experimento) conviene siempre en efectivo para minimizar movimientos con tarjeta y transferencia; ya registrado, conviene por el mismo medio (ventaja contable). Configurable.
