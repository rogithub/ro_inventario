# Diseño 05 — Corte de caja

**Estado:** propuesta para revisión del dueño. Depende de [04 — Ventas](04-ventas.md) y [02 — Compras](02-compras.md) (pagos a proveedores en efectivo).

## Cómo se hace hoy (2026-09-29)
- **Una sola caja física** (un cajón) y tres puntos de venta que cobran a ella: la computadora de escritorio, el iPad y la Elo touch.
- **El corte se hace cuando se mueve dinero al banco** y se saca de la caja; se deja lo que haya. No hay fondo fijo ni horario.
- La v1 no tiene corte de caja propiamente dicho: tiene un **resumen del día** por forma de pago (informa, no compara contra lo contado) y un **saldo de caja** en finanzas (desde marzo de 2026) que suma los **dólares convertidos a pesos**, aunque en el cajón son billetes aparte.
- **Esto ha causado muchas inconsistencias:** al retirar no queda registro de cuánto había ni de cuánto se esperaba, así que los faltantes se acumulan sin saber de cuándo son.

## Lo que se decidió
- **El sistema no impone cuándo cortar; cada conteo queda registrado.** Un corte es una sola operación que se puede hacer en cualquier momento: contar, comparar con lo esperado, anotar la diferencia y, si se quiere, retirar dinero. Lo que queda es el inicio del siguiente periodo.
  - La papelería corta al llevar dinero al banco y deja lo que haya (su costumbre de hoy, ahora con registro).
  - Un negocio disciplinado corta cada noche o por turno, con fondo fijo: la misma operación, más seguido.
  - Un conteo sin retirar también es un corte.
- **Pesos y dólares se cuentan por separado.**
- **Caja ≠ punto de venta:** una caja es un cajón de dinero; los puntos de venta son desde donde se cobra. Hoy hay una caja; varias cajas (negocios más grandes) se agregan cuando haya un caso real.
- **Cada venta anota desde qué punto de venta se cobró**, para investigar diferencias.
- **Anotar lo que sale del cajón tiene que tomar segundos**, porque hoy muchos gastos y retiros no se capturan (ir por las tortillas y olvidarlo):
  - **En el momento:** botón "Saqué dinero" en cualquier punto de venta: monto y para qué. Nada más.
  - **Al cortar, lo olvidado:** antes de cerrar, el corte muestra el faltante y permite explicarlo agregando lo que se recuerde ("tortillas $30", "retiro personal $200"); cada cosa se vuelve un movimiento de caja. Lo que nadie sepa explicar queda como faltante.
- **La papelería pasará a corte diario** al cerrar (lo pidió la esposa del dueño). Hoy existe solo una tablita al final de `/ventas/index` de la v1 con lo que debería haber por forma de pago; en la v2 es el corte completo, con conteo y registro. Un faltante de hoy todavía se recuerda; uno de hace tres semanas, no.

## Tablas propuestas

### `movimientos_caja`
El efectivo que entra o sale del cajón sin ser una venta, una devolución ni un pago a proveedor (esos ya se registran en sus propios documentos y el corte los lee de ahí; no se capturan dos veces).
| Columna | Qué es |
|---|---|
| `id`, `fecha` | |
| `concepto` | `gasto`, `deposito_banco`, `retiro_dueno`, `aportacion`, `otro`. |
| `moneda`, `monto` | Con signo: + entra, − sale. |
| `descripcion`, `created_by` | "Garrafón de agua", "Depósito BBVA". |

### `cortes_caja`
| Columna | Qué es |
|---|---|
| `id`, `realizado_at`, `created_by` | Cuándo y quién contó. El periodo va del corte anterior a este. |
| `notas` | Explicación de una diferencia, si se sabe. |

### `cortes_caja_monedas` (un renglón por moneda: pesos y dólares)
| Columna | Qué es |
|---|---|
| `corte_id`, `moneda` | |
| `inicial` | Lo que quedó en el corte anterior (lo que de verdad se dejó, no lo esperado). |
| `esperado` | Calculado al momento del corte y **guardado**: lo que debería haber. |
| `contado` | Lo que se contó. |
| `retirado` | Lo que se sacó en este corte (se registra también como movimiento de caja: depósito o retiro). |

- **Diferencia** = `contado − esperado` (sobrante si es positiva, faltante si es negativa). Se calcula, no se guarda.
- **Queda para el siguiente periodo** = `contado − retirado`.

**Cálculo del esperado** (código puro, con pruebas), por moneda, entre el corte anterior y este:
`inicial` + pagos en efectivo de ventas (en esa moneda) − cambio dado − reembolsos en efectivo − pagos a proveedores en efectivo + movimientos de caja.
- El cambio siempre sale en pesos, aunque el cliente haya pagado en dólares.
- Tarjeta y transferencia no están en el cajón: el corte las muestra como totales del periodo para cotejar contra la terminal y el banco, sin contarlas.
- Una venta cancelada después de un corte afecta al periodo en que se cancela, no al ya cerrado (el esperado de un corte guardado no cambia).

## Cómo se ve en la práctica (papelería)
1. Van a llevar dinero al banco. Abren "Corte de caja".
2. El sistema muestra lo esperado en pesos y en dólares desde el último corte, más los totales de tarjeta y transferencia.
3. Cuentan: capturan cuánto hay en pesos y en dólares.
4. El sistema muestra la diferencia. Si la hay, se anota lo que se sepa ("se pagó el garrafón y no se capturó").
5. Si hay faltante, agregan lo que recuerden haber sacado (tortillas, retiros): se vuelven movimientos de caja y la diferencia se reduce.
6. Capturan cuánto se lleva al banco (en el corte diario puede ser nada). Queda lo demás para el siguiente periodo.

## Fuera de este diseño
- **Varias cajas** (negocios con más de un cajón): cuando haya un caso real. Los puntos de venta ya quedan anotados en cada venta.
- **Banco** (depósitos, pagos por transferencia, conciliación con el estado de cuenta), socios y el resto de finanzas: módulo de finanzas, después del corte.
- **Conteo por denominación** (cuántos billetes de $500, de $200…): ayuda de pantalla opcional; no cambia los datos.

## Respuestas del dueño (2026-09-29)
1. **Gastos de la caja:** a veces ni se capturan (ir por las tortillas y olvidarlo).
2. **Diferencias:** basta con verlas en el corte, sin avisos.
3. **Retiros personales:** igual que los gastos, muchas veces no se anotan.
- Lo mejor sería el **corte diario**: la esposa del dueño lo pidió; por ahora solo existe una tablita al final de `/ventas/index` con lo que debería haber por forma de pago.
