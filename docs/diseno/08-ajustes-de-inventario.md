# Diseño 08 — Ajustes de inventario y conteo físico

**Estado:** propuesta para revisión del dueño. Depende de [01 — Productos e inventario](01-productos-e-inventario.md) (kárdex) y [03 — Servicios](03-servicios.md) (recetas).

## Cómo está hoy (v1)
- Hay una página de ajuste manual por producto (`/Productos/Stock/?id=…`): merma o ingreso sin compra, con una nota. 86 mermas y 63 ingresos en toda la historia.
- **No se cuenta el inventario.** Las cosas se pierden y no se sabe cuándo: el sistema dice que hay un kilo de barritas de silicón y nadie las encuentra.
- **Mermas frecuentes:** trabajos mal hechos (se imprimió a color lo que era en blanco y negro). A veces el dueño y su esposa las pagan de su dinero para aprender la lección (se hace la venta y entra a la caja); a veces solo se pierden.
- **No hay ubicación física** de los productos. En la tienda hay tres lugares: **mostrador**, **bodeguita** junto al mostrador (lo que se resurte rápido) y **bodega** (lo que se tiene en cantidad y resurte la bodeguita).

## Lo que se propone
- **Ajuste manual con motivo de una lista corta** + nota libre. La lista permite ver cuánto dinero se va por cada motivo.
- **Merma de un servicio:** se registra el servicio mal hecho ("Copia color × 1") y salen los insumos de su receta. No hay que adivinar qué hoja se gastó.
- **Si alguien paga un trabajo mal hecho de su bolsa, no es merma: es una venta** (le compra a la tienda la copia mal hecha, con su cobro normal). No hace falta nada especial.
- **Conteo físico por partes** (un anaquel o una categoría a la vez): las diferencias contra el kárdex se vuelven ajustes. Sin cerrar la tienda.
- **Ubicaciones = etiquetas de estante**, opcionales: cada negocio etiqueta sus estantes y los divide en secciones como quiera ("Mostrador", "Estante 1", "A-3"). El sistema no impone niveles ni estructura. Sirven para encontrar un producto y para contar por estante. **El stock sigue siendo uno solo por producto** (ver "Stock por lugar").

## Tablas propuestas

### `ajustes`
| Columna | Qué es |
|---|---|
| `id`, `folio`, `fecha` | |
| `kind` | `merma` (sale) o `ingreso_sin_compra` (entra). |
| `motivo` | `danado`, `error_de_trabajo`, `pedido_no_recogido` (lo gastado en un pedido que nadie recogió, diseño 04), `uso_interno`, `extravio`, `encontrado`, `otro` (lista fija, en el código: sirve a cualquier negocio). |
| `nota` | Libre: "se imprimió a color, era BN", "no se encontraron en el anaquel". |
| `conteo_id` | Si salió de un conteo físico. |
| `created_by`, `created_at` | |

### `ajustes_partidas`
| Columna | Qué es |
|---|---|
| `id`, `ajuste_id` | |
| `articulo_id` | Un producto, o un servicio (entonces salen los insumos de su receta). |
| `cantidad` | En la unidad del artículo. |

Cada partida de producto (o cada insumo de un servicio) escribe su movimiento al kárdex (`merma` o `ingreso_sin_compra`) con referencia a la partida del ajuste. El costo de la merma es el costo promedio de ese momento: así se sabe cuánto dinero se perdió.

### `ubicaciones` y `productos_ubicaciones`
- `ubicaciones`: `id`, `etiqueta` (lo que está pegado en el estante: "Mostrador", "Estante 1", "A-3"). Texto libre, datos de cada negocio; sin jerarquía.
- `productos_ubicaciones`: `producto_id`, `ubicacion_id`. Un producto puede estar en varios estantes (parte en el mostrador, el resto en la bodega).

La papelería, por ejemplo: "Mostrador"; "Estante 1" a "Estante 3" en la bodeguita; "Estante A" a "Estante E" en la bodega grande, cada uno dividido en secciones.

**Stock por lugar (decisión):** el stock es uno solo por producto; las ubicaciones dicen dónde buscarlo, no cuánto hay en cada lugar. Llevar stock por lugar obligaría a registrar cada traspaso (bajar cosas de la bodega a la bodeguita); si no se registra, el dato se descuadra en días, y hoy ni los gastos de caja se capturan siempre. Se agrega cuando haya disciplina o un negocio que la necesite (ver "Fuera de este diseño").

### `conteos` y `conteos_partidas`
- `conteos`: `id`, `iniciado_at`, `cerrado_at`, `alcance` (los productos de una ubicación, una categoría o una lista), `created_by`.
- **Se cuenta cada producto completo:** todo lo que hay de él en todos sus lugares (como el stock es uno solo). Contar "la bodeguita" significa contar los productos que viven ahí, en todos sus lugares.
- `conteos_partidas`: `conteo_id`, `producto_id`, `esperado` (lo que decía el kárdex al contar), `contado`.
- **Al cerrar el conteo**, cada diferencia genera un ajuste: faltante = merma con motivo `extravio` ("no se encontró en el conteo"); sobrante = ingreso con motivo `encontrado`.
- Lo vendido mientras se cuenta: el esperado se toma al capturar cada producto, no al iniciar el conteo.

## Reportes que habilita
- **Pérdidas por motivo y por mes:** cuánto dinero se fue por trabajos mal hechos, extravíos, daños.
- **Productos que más se pierden:** candidatos a guardarse en otro lugar.
- **Hace cuánto no se cuenta cada ubicación:** para repartir los conteos cíclicos.

## Permisos
- `ajustar_inventario` (diseño 07): ajustes manuales y conteos.

## Fuera de este diseño
- **Stock separado por lugar** (cuánto hay en mostrador, bodeguita y bodega; traspasos; avisos de "resurte el mostrador"): cuando haya disciplina para registrar cada traspaso, o un negocio que lo necesite. También varias sucursales.
- **Tinta y tóner** de un trabajo mal hecho: con el módulo de impresoras.
- **Devoluciones al proveedor:** con compras, cuando se necesiten.

## Respuestas del dueño (2026-09-29)
1. **Ubicaciones:** hay tres niveles: mostrador; bodeguita cercana al mostrador (lo que se resurte rápido); bodega (lo que se tiene en cantidad y resurte la bodeguita). Mejor que describir la tienda: **etiquetar cada estante y dividirlo en secciones** (mostrador; estantes 1, 2 y 3 en la bodeguita; estantes A a E en la bodega). No hay que forzar a todos los negocios a una estructura: basta con que etiqueten sus estantes.
2. **Mermas pagadas:** el dinero entra a la caja y se hace su venta (y se ponen orejas de burro por una hora, pero eso no hace falta reportarlo al sistema).
