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
- **Una venta guardada no se edita: se cancela (con motivo) y se recaptura.** La cancelación regresa el inventario sola, en un paso. Hoy se hace a mano como devolución y es tedioso.
- **La devolución queda para cuando el cliente regresa mercancía.**
- **Fecha editable** para capturar ventas atrasadas; queda registro de cuándo se capturó.
- **El monedero va en el corte mínimo:** ya se anunció y los clientes cuentan con él (se diseña con clientes). **Los pedidos** llegan después del corte.

## Datos de la versión uno que guiaron el diseño (últimos 12 meses)
- ~17 tickets al día, 1.9 partidas por ticket, **ticket mediano de $18** (promedio $37). Horas pico: 11–13 h y 17 h. La caja tiene que ser rápida para tickets chicos.
- Tickets por forma de pago: efectivo 5,824; transferencia 195; tarjeta 166; dólares 3 (históricamente se convertían a pesos a mano y se guardaban como pesos, así que el número real es mayor); monedero 2. Pagos mixtos: 15 tickets.
- 28 % de los tickets llevan cliente. El monedero se ha generado 1,919 veces y canjeado 26 en toda la historia. **Ojo: el número engaña.** Se lanzó con cautela y se anunció hace poco; ya hay clientes siguiendo su acumulado por el enlace de xplaya.com, y una de ellas regresó a comprar después de consultarlo.
- 122 partidas devueltas en 50 días distintos. 13 pedidos en total, ninguno en línea.

## Tablas propuestas

### `ventas`
| Columna | Qué es |
|---|---|
| `id`, `folio` | Folio consecutivo del ticket. |
| `fecha` | Cuándo ocurrió la venta. Normalmente = `created_at`; se puede poner una anterior al capturar ventas atrasadas. Si difieren, se sabe que se capturó después. |
| `cliente_id` | Opcional. |
| `cambio` | Lo que se le regresó al cliente, en pesos. |
| `cancelada_at`, `motivo_cancelacion`, `cancelada_by` | Si se canceló. Una cancelada no cuenta en reportes ni en caja, pero se conserva. |
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

**Reglas** (en código puro, con pruebas; se llevan de la v1):
- **Total de la venta** = suma de partidas + comisiones.
- **Comisión de tarjeta** = iterativa: se calcula sobre el monto con tarjeta, y otra vez sobre ese monto + la primera comisión, con IVA; la tasa sale de los ajustes de la BD (la cambia el dueño).
- **Solo el efectivo (pesos o dólares) da cambio.** Tarjeta y transferencia son por el monto exacto.
- La suma de pagos (en pesos) menos el cambio debe ser igual al total.

## Cómo afecta al inventario
Al guardar la venta, en la misma transacción:
- **Producto:** escribe su salida al kárdex (en unidad base; un paquete de 100 hojas saca 100).
- **Servicio con receta:** cada insumo escribe su salida.
- **Kit:** cada partida que agregó se comporta según su tipo.
- **Servicio sin receta:** nada.

## Cancelar una venta
Un solo paso, con motivo obligatorio: marca la venta como cancelada y escribe en el kárdex las entradas que la anulan (el inventario regresa). Los pagos de una cancelada dejan de contar en el corte de caja. Después se captura la venta correcta.

## Devoluciones (el cliente regresa mercancía)
- `devoluciones`: `id`, `venta_id`, `fecha`, `motivo`, `reembolso` (cuánto se le regresó y por qué forma de pago; se preselecciona según la configuración del negocio), `created_by`.
- `devoluciones_partidas`: `venta_partida_id`, `cantidad_buen_estado`, `cantidad_danada`.
- Lo que regresa en buen estado entra al kárdex (`devolucion`); lo dañado entra y sale como merma, para que quede la evidencia.

## Fuera de este diseño
- **Monedero** (generar, canjear, consultar por enlace): se diseña con clientes, en el corte mínimo. **Pedidos:** después del corte.
- **Corte de caja** (efectivo esperado contra contado, por forma de pago y moneda): se diseña aparte, es corto y va en el corte mínimo.
- **Comisión que cobra la terminal** (contra la que se le cobra al cliente): con finanzas.
- **Facturación (CFDI) de una venta** y el ticket público con enlace: después.
- **Descuentos** por venta: no existen hoy; el mayoreo va por presentaciones.

## Respuestas del dueño (2026-09-29)
1. **Tipo de cambio:** el de Banxico con spread configurable, como hoy. El spread nació al cobrar en dólares: los clientes decían "Banco Azteca me los paga a tanto", menos que el API. Se aprendió sobre la marcha.
2. **Cambio en dólares:** siempre en pesos.
3. **Reembolso:** depende. Hoy (sin alta en Hacienda, es un experimento) conviene siempre en efectivo para minimizar movimientos con tarjeta y transferencia; ya registrado, conviene por el mismo medio (ventaja contable). Configurable.
