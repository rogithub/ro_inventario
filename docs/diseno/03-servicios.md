# Diseño 03 — Servicios

**Estado:** propuesta para revisión del dueño. Depende de [01 — Productos e inventario](01-productos-e-inventario.md) (un servicio es un artículo).

## Lo que ya se decidió (2026-09-29)
- **Un servicio es lo que se hace, no lo que se compra.** Se da de alta en el catálogo, nunca en compras, y no lleva stock. Las compras son solo para lo físico que entra al inventario (lo que se revende y lo que se consume al dar un servicio).
- **Artículo + dos tablas:** lo común (NID, nombre, categoría, precio) en `articulos`; lo propio del servicio en `servicios`.
- **Las familias de servicios son solo categorías:** organizan menús y reportes, sin lógica propia. Todo lo que cambia entre un servicio y otro cabe en **tres atributos**: receta, costo externo y precio libre.
- **Una sola "Recarga de saldo"** con monto libre, en lugar de doce recargas por monto. Los botones rápidos siguen poniendo el monto de un toque.
- **El engargolado no lleva receta:** se vende en partidas separadas y medibles (las impresiones con su hoja, el servicio de perforar cobrado por hoja y por pasta, las pastas y los arillos como productos). Cada pieza varía según lo que elige el cliente.
- **La comisión por pago con tarjeta no es un servicio:** es un recargo del cobro; se diseña con ventas. La comisión por **pago de servicio** sí es un servicio: es la tarifa por hacer el trámite.
- **Kit = un artículo que se vende como varias partidas juntas** (ver "Kits"). "Pago de servicio" es un kit: comisión de $5 + el pago al tercero de monto libre. Así no se olvida capturar la comisión.
- **El costo externo se edita en el servicio; la venta guarda el de ese momento** (con su ganancia), así un cambio no altera las ventas pasadas.

## Datos de la versión uno que guiaron el diseño
- 109 servicios. En la v1 un servicio necesitaba una "compra" (con cantidad 1,000,000) para existir con costo, y la compra un proveedor: de ahí los proveedores inventados y el stock de 999,993.
- Los 5 "kits" son servicios que consumen material:

  | "Kit" en la v1 | Qué es |
  |---|---|
  | Enmicado carta | Servicio de enmicado que consume 1 mica |
  | Polaroid mate / glossy / adhesivo | Servicio que consume 1 papel fotográfico e incluye impresión color y corte |
  | Impresión en folder | Impresión que consume 1 folder |

- "Pago de servicio" y la comisión de tarjeta usan el mismo truco: precio de $1 y cantidad = monto.
- Las categorías de servicios estaban revueltas: SERVICIOS (77), SERVICIO (13), TRAMITES (7), TRAMITE (1), varias de impresión…

## Las familias de la papelería (categorías de partida)
Salen de las ventas de los últimos 12 meses. **Son datos de la papelería, no del sistema:** llegan con la migración desde la versión uno (`tools/migracion-v1`), no con los datos base. Un negocio nuevo empieza sin categorías y crea las suyas (un negocio de seguridad industrial tendría "calzado de seguridad", "señalización"…). El código nunca nombra una categoría: lo que cambia el comportamiento son los tres atributos del servicio. Si algún día hay varios negocios del mismo giro, se pueden ofrecer plantillas de arranque opcionales.

| Familia | Ejemplos | Receta | Costo externo | Precio |
|---|---|---|---|---|
| **Impresión y copias** (~5,000 ventas) | BN y color; chico, mediano y grande; carta, oficio y tabloide; por impresora (REFK1, REFR2, REFH…) | La hoja | — | Fijo |
| **Enmicado** | Mica carta, oficio, 3R, 4R, credencial… | La mica | — | Fijo |
| **Trámites** | Actas por estado, CURP, NSS, SAT, constancias, antecedentes | — | Fijo cuando se le paga a un tercero (acta de Oaxaca: $136 de $240); ninguno en el CURP | Fijo |
| **Recargas y pagos** | Recarga de saldo, pago de servicio | — | Igual a lo cobrado (ingreso trasladado) | Libre |
| **Trabajo de oficina** | Tarea, CV, flyer, transcripción, edición de imagen, PDF a Word, escaneo, respaldo de fotos, letreros | — | — | Fijo o libre ("concepto editable") |
| **Acabados** | Engargolado (perforar), corte, envoltura de regalo, ampliación | — por ahora | — | Fijo |
| **Renta** | Internet por hora o por día | — | — | Fijo |

## Tablas propuestas

### `servicios` (lo propio de un servicio)
| Columna | Qué es |
|---|---|
| `articulo_id` | El artículo que es este servicio. |
| `is_precio_libre` | El precio (y la descripción) se capturan en cada venta: pago de un recibo, recarga, concepto editable. |
| `costo_externo` | Lo que se le paga a un tercero por dar el servicio (los derechos de un acta). Vacío = ninguno. |
| `is_trasladado` | Todo lo cobrado es para un tercero (recargas, pagos de servicio): el costo es igual a lo cobrado y no cuenta en el margen. No se combina con `costo_externo`. |

### `recetas`
`servicio_id`, `componente_id` (un artículo), `cantidad`. Lo que un servicio consume o incluye:
- **Insumos:** productos que se gastan al hacerlo (copia carta BN → 1 hoja carta; enmicado carta → 1 mica carta). Al vender el servicio, cada insumo escribe su salida al kárdex. El ticket solo muestra el servicio.
- **Servicios incluidos:** otros servicios que forman parte (polaroid → impresión color + corte). Sirven para el costo y para contar cuántas veces se hizo cada uno. Una receta no puede incluirse a sí misma, ni directa ni indirectamente.

### Kits (artículos que se venden juntos)
Un **kit** es un artículo (`kind = kit`) que al venderse **agrega varias partidas a la venta**, cada una con su precio. No tiene precio propio: vale lo que sumen sus partidas.

`kit_componentes`: `kit_id`, `articulo_id` (producto o servicio), `cantidad`, `orden`.

| Kit | Partidas que agrega |
|---|---|
| Pago de servicio | Comisión por pago de servicio ($5) + Pago a tercero (trasladado, monto y concepto libres: "Pago de CFE", $350) |
| CURP impreso | Trámite CURP ($10) + Copia BN (con su hoja como insumo) |

- Si un componente es de precio libre, la caja lo pide al agregar el kit.
- Cada partida conserva de qué kit vino, para los reportes.
- Disponibilidad: la de sus componentes.
- **Kit no es receta:** la receta es interna (la copia gasta una hoja y el ticket dice solo "Copia"); el kit se ve en el ticket como varias partidas.
- **Kit no es botón rápido:** el botón es un atajo de pantalla; el kit es un artículo con NID que se busca y se vende como cualquier otro.

## Cómo se comporta un servicio
- **Disponibilidad en caja:** sin insumos (el CURP), siempre disponible; con insumos, mientras alcancen (el menor de stock ÷ cantidad entre sus insumos).
- **Costo de una venta** = costo de sus insumos (promedio móvil, al momento de la venta) + costo de sus servicios incluidos + `costo_externo`. Si `is_trasladado`, el costo es lo cobrado.
- **Margen:** precio menos costo. Los trasladados no cuentan (entran y salen de caja). Un servicio sin receta ni costo externo es todo margen (el valor de tu tiempo).
- **Servicios que se venden juntos** ("CURP impreso", "Pago de servicio") son kits: varias partidas de un toque.
- **Descontinuar** funciona igual que en un producto (en `articulos`).

## Fuera de este diseño
- **Impresoras:** tóner, contadores (SNMP), desgaste del equipo, costo real por copia y cuánto deja cada máquina. Es un módulo aparte que conecta servicios, hojas, colores y equipos; se revisará a detalle. Mientras tanto, las copias se migran como hoy (un servicio por impresora, color y tamaño) con la hoja como insumo.
- **La impresora como equipo** (inversión, no mercancía): con impresoras y finanzas.
- **Comisión por pago con tarjeta:** con ventas.
- **Guardar en la venta el costo externo y la ganancia de cada partida:** con ventas.
- **Tiempo del personal** como costo del servicio: no se mide.

## Respuestas del dueño (2026-09-29)
1. **Costo externo:** basta con editarlo, siempre que la venta guarde el valor de la ganancia de ese momento.
2. **Pago de servicio:** la forma actual (pago + comisión en dos partidas) funciona, pero lo importante es que no se olvide capturar la comisión. Propuesta del dueño: un kit con dos componentes, los $5 fijos de la comisión + una entrada libre del ingreso que se traslada (la vecina con su recibo de luz de $350: $5 de comisión + "1 × 350 Pago de CFE").
3. **Categorías:** las siete familias (impresión y copias, enmicado, trámites, recargas y pagos, trabajo de oficina, acabados, renta) están bien para la papelería; otros negocios tendrán las suyas.
