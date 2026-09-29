# Diseño 06 — Clientes y monedero

**Estado:** propuesta para revisión del dueño. Depende de [04 — Ventas](04-ventas.md).

## Por qué va en el corte mínimo
El monedero se diseñó con cuidado, se probó varias veces y ya **se anunció a los clientes, que cuentan con él**. Aunque pocos lo han canjeado, hay clientes siguiendo su acumulado: el 29 de septiembre de 2026, una clienta pidió su saldo, se le mandó el enlace de xplaya.com por WhatsApp y regresó a comprar (probablemente la venta más alta de la historia de la tienda). Una promesa a los clientes no se rompe en una migración.

## Cómo funciona hoy (v1) — se conserva
- **Genera:** cada partida de una venta **con cliente** abona `importe × porcentaje` al monedero (hoy 2 %: `TIPO_CAMBIO_MONEDERO = 0.02`), con vigencia de **30 días** (`DIAS_VIGENCIA_MONEDERO`). Cada abono guarda el porcentaje con que se generó.
- **No genera:** los ingresos trasladados (recargas, pagos de servicio) ni las partidas que forman parte de un kit (en la v1, el renglón del kit ya generaba).
- **Se puede generar después:** si se olvidó poner el cliente en la venta, se le asigna y se genera su monedero.
- **Se canjea como forma de pago**, en múltiplos de $0.50, empezando por lo más antiguo (lo que vence primero).
- **Deja de valer** al vencer o si la partida se devuelve.
- **Se consulta en xplaya.com:** por enlace opaco (`/monedero/{id del cliente}`) o por teléfono (`/saldo`), con aviso de lo que está por vencer. Los términos del programa están en `/terminos`.
- 215 clientes; 28 % de los tickets llevan cliente.

## Lo que cambia en la v2
- **Kits:** en la v2 un kit agrega partidas con su propio precio, así que cada partida genera según su tipo (la comisión de un pago de servicio sí genera; el pago trasladado no). Resultado equivalente al de la v1.
- **Cancelar una venta** anula su monedero generado y **regresa al cliente lo que haya canjeado** en ella, en un solo paso (ver diseño 04).
- **El pago con monedero es una forma de pago** en `ventas_pagos` (el SAT lo tiene: "monedero electrónico").
- **El porcentaje y la vigencia** son ajustes de la BD (capa 3: el dueño los cambia en pantalla), como hoy. Para la papelería se quedan en **2 % y 30 días**: es lo que se promete a los clientes.
- **Lo pagado con monedero también genera monedero,** como en la v1: el monedero es una forma de pago más y no complica la lógica.
- **La tarjeta de cliente impresa no se migra:** no se usa.

## Tablas propuestas

### `clientes`
| Columna | Qué es |
|---|---|
| `id` | **Se conserva el de la v1:** es el que va en los enlaces del monedero ya enviados. |
| `telefono` | **Obligatorio y único.** Es la forma más confiable de identificar al cliente: muchos compran por WhatsApp, se les mandan enlaces con tarjetas informativas (Open Graph) y los pedidos se consultan por teléfono. También identifica al cliente en `/saldo`. |
| `nombre` | **Opcional:** a veces solo se sabe su número (escribió por WhatsApp para imprimir algo). |
| `email` | Opcional, único. Casi nadie lo usa en la zona. |
| `notas` | |
| `acepto_programa_at` | Cuándo aceptó participar en el monedero. **El monedero se calcula siempre** (lógica simple); pero **solo se le ofrece, se le muestra y se reporta** a quien aceptó. Hay clientes que no lo quieren ("yo no quiero que me estén fregando"): es un permiso de comunicación, no una regla de cálculo. Si cambian de opinión, su saldo ya está ahí. |
| `vetado_hasta`, `motivo_veto`, `vetado_by` | **Veto para pedidos a distancia** (ver "Clientes que no recogen"). Vacío = sin veto. |
| `created_at`, `updated_by` | |

### `monedero_abonos` (lo que se gana)
| Columna | Qué es |
|---|---|
| `id`, `cliente_id` | |
| `venta_partida_id` | La partida que lo generó. |
| `monto`, `porcentaje` | Cuánto se abonó y con qué porcentaje. |
| `created_at`, `vence_at` | Cuándo se generó y cuándo vence. |
| `anulado_at`, `motivo_anulacion` | Si se anuló (devolución o cancelación de la venta). |

### `monedero_canjes` (lo que se usa)
| Columna | Qué es |
|---|---|
| `id`, `abono_id` | De qué abono salió. Un canje de $10 puede salir de varios abonos (los más antiguos primero). |
| `venta_pago_id` | El pago con monedero de la venta donde se usó. |
| `monto`, `created_at` | |
| `revertido_at` | Si la venta se canceló, el canje se revierte y el saldo regresa. |

**Cálculos** (código puro, con pruebas):
- **Saldo** = suma de (`monto` del abono − lo canjeado de él) de los abonos vigentes (no vencidos, no anulados).
- **Repartir un canje** entre abonos: el que vence primero, primero.
- **Máximo canjeable:** el saldo, redondeado hacia abajo a múltiplos de $0.50 (la regla de la v1, `calcularMaximoDivisor50c`).
- **Por vencer:** lo que vence en los próximos N días, para el aviso en xplaya.com.

## Clientes que no recogen
Pasa poco, pero duele: clientes (incluso conocidos) piden copias o impresiones desde su casa y nunca pasan por ellas; se pierden hojas y tinta.
- **Historia automática, sin estrellas a mano:** los pedidos cancelados con motivo "no lo recogió" se cuentan solos. Al buscar al cliente o su teléfono para un pedido se ve, p. ej., "recogió 12 de 14 pedidos; el último que no recogió: 3 de agosto".
- **Veto para pedidos a distancia:** el dueño puede vetar a un cliente: no se le reciben pedidos por WhatsApp, solo se le vende en la tienda. Lleva motivo y fecha de vencimiento (por omisión, un plazo configurable); **al vencer se levanta solo** (no es para siempre: "un banco no te veta para siempre"), y se puede levantar antes.
- **Aviso en caja:** al guardar un pedido para ese teléfono (del cliente o de contacto) aparece el veto con su motivo y vencimiento.

## Enlaces públicos que no se pueden romper
Los clientes ya tienen estos enlaces en WhatsApp. La aplicación pública de la v2 mantiene las mismas rutas, y la migración conserva los identificadores:

| Ruta | Identificador que se conserva |
|---|---|
| `/monedero/{id}` | id del cliente |
| `/saldo` | teléfono del cliente |
| `/recibo/{id}` (y `/pdf`, `/print`) | id de la venta (en la v1, el id del "ajuste" de tipo venta) |
| `/cotizacion/{uid}` (y `/pdf`, `/print`) | uid del pedido |
| `/terminos` | los términos del programa |

Los enlaces que se comparten por WhatsApp llevan metadatos Open Graph (tarjeta con imagen y texto al pegarlos); la aplicación pública de la v2 los conserva.

## Cómo se prueba la migración del monedero
- **Saldo por cliente:** para cada cliente (excepto el cliente anónimo de la v1), el saldo en la v2 recién migrada es igual al de la v1, al centavo, a la misma fecha.
- **Lo que está por vencer** coincide, cliente por cliente.
- **Cada enlace de la tabla anterior** responde en la v2 para una muestra de ids reales de la v1.

## Fuera de este diseño
- Clientes como usuarios con acceso propio (portal de clientes): a futuro.
- Clientes que son negocios (mayoreo, facturación a su RFC): con facturación.
- **Sin "cliente anónimo".** En la v1 existe un cliente fijo (`ID_CLIENTE_GENERICO`): "Cliente Papeleria", teléfono "GENERICO CLIENTE", sin aceptar el programa; 477 ventas, $31.58 de monedero que nadie puede cobrar y 1 pedido pendiente. En el código solo aparece en el botón "asignar cliente genérico" de las dos cajas; nada más lo trata distinto, por eso genera monedero sin dueño. Cubría dos necesidades distintas, que en la v2 tienen su lugar:

  | Situación | En la v1 | En la v2 |
  |---|---|---|
  | Cliente que acepta el monedero | Cliente | Cliente (aceptó) |
  | Cliente que no quiere el monedero | Cliente sin aceptar, o el anónimo | Cliente sin aceptar: se calcula, no se ofrece |
  | Extraño, o sin datos a la mano (pedido o cotización) | El cliente anónimo | Venta **sin cliente**, con nombre y teléfono de contacto opcionales para encontrar el pedido |

  - **Solo genera monedero una venta con cliente registrado:** ya no hay monedero sin dueño.
  - **Migración:** las ventas del anónimo pasan sin cliente; sus $31.58 no se migran (nunca fueron cobrables); la prueba de saldos lo excluye; su pedido pendiente pasa como pedido sin cliente.
- **Pedidos:** son ventas por cobrar o por entregar (diseño 04). El enlace `/cotizacion/{uid}` de los pendientes se conserva.

## Respuestas del dueño (2026-09-29)
1. **2 % y 30 días:** son parámetros configurables, pero para la papelería es lo que se promete y se queda igual.
2. **Lo pagado con monedero genera monedero:** sí, como en la v1, para no complicar; el monedero es una forma de pago más.
3. **Tarjeta impresa:** no se usa.
4. **Teléfono obligatorio, email opcional:** muchos clientes compran por WhatsApp; se les mandan enlaces con Open Graph desde el sitio público. El teléfono es la forma más confiable de verificar identidad; el correo casi nadie lo usa en la zona.
5. **Aceptación:** al principio todos tenían monedero a fuerzas, hasta que una clienta dijo "yo no quiero que me estén fregando"; por eso existe la marca. Siempre se calcula por simplicidad, pero solo se ofrece y se reporta a quien aceptó. Aun sin monedero el teléfono importa: los pedidos por WhatsApp (mandan algo para imprimir, se guarda el número de pedido y la venta se asocia al cliente por su teléfono; al recoger dice "mi pedido es tal" o "mi teléfono es tal").
