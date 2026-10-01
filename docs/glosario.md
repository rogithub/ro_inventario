# Glosario del negocio

Las palabras del negocio que van **en español** en el código y en la base de datos. Todo lo que no esté aquí va en inglés (verbos, conceptos técnicos). Regla completa: [decisión de nombres](decisiones/2026-09-29-nombres.md).

- Salen de cómo se habla en la tienda y, cuando existen, de los términos oficiales (SAT). Sin acentos ni ñ en el código (`comision`, `devolucion`).
- Una palabra por concepto: si ya hay una, se usa esa (`Stock`, no `Existencia`).
- ¿Falta una palabra del negocio? Se agrega aquí en el mismo commit que la usa por primera vez.
- Parte del glosario de la versión uno (`/home/ro/code/inventario_papeleria/docs/glosario.md`), sin sus detalles de tablas.

## El negocio y sus usuarios
| Término | Qué es |
|---|---|
| `Negocio` | La empresa que usa el sistema. Cada una tiene su propia instancia y base de datos. |
| `Usuario` | Persona que entra al sistema. |
| `Permiso` | Una acción que un usuario puede hacer (vender, cancelar una venta, ver costos…). La lista vive en el código y sus nombres son del glosario (ver casos de frontera). |
| `Rol` | Conjunto de permisos con nombre (Dueño, Encargado, Cajero). Cada negocio puede tener los suyos. |

## Productos
| Término | Qué es |
|---|---|
| `Catalogo` | Todo lo que el negocio ofrece y sus datos de apoyo: artículos, categorías, unidades, presentaciones. El permiso `editar_catalogo` lo cambia; el catálogo público lo muestra. |
| `Articulo` | Lo que se vende en caja: un producto o un servicio. Tiene NID, nombre, categoría y precio. |
| `Producto` | Artículo físico: se compra, lleva stock y se revende o se consume en un servicio. |
| `NID` | Número corto del producto: se teclea en caja y se imprime en la etiqueta como código de barras. |
| `CodigoBarras` | Código impreso en un producto o en su empaque (presentación). |
| `Nombre`, `Marca`, `Modelo`, `Color`, `Descripcion` | Datos del catálogo de un producto. `Nombre` también es el de un cliente, un proveedor, un usuario, un rol o una unidad. |
| `Kardex` | Registro de todos los movimientos de inventario (entradas y salidas) de cada producto. |
| `Descontinuado` | Producto que ya no se resurte ni se ofrece; conserva su historia. |
| `Motivo` | Por qué se hizo algo: una merma, un ajuste, una descontinuación. |
| `Mayoreo` | Venta por volumen, con precio de descuento; se ofrece con presentaciones. |
| `Servicio` | Artículo que es algo que se hace (copias, enmicado, recargas, trámites). No se compra ni lleva stock; puede tener receta. |
| `CostoExterno` | Lo que se le paga a un tercero por dar un servicio (los derechos de un acta). |
| `Receta` | Lo que un servicio consume o incluye para hacerse: insumos y otros servicios, con su cantidad. |
| `Insumo` | Producto físico que un servicio gasta al hacerse y que no se vende por separado en ese momento (la hoja de una copia, la mica de un enmicado). |
| `Componente` | Cada elemento de una receta (un insumo o un servicio incluido). |
| `Kit` | Artículo que se vende como varias partidas juntas ("Pago de servicio" = comisión + pago libre). No es una receta. En la v1 se llamaba kit a lo que hoy es un servicio con receta. |
| `Presentacion` | Forma de venta con su factor: pieza, paquete, caja. |
| `Categoria` | Agrupación del catálogo. |
| `UnidadMedida` | Pieza, metro, hoja… |
| `Precio`, `PrecioVenta`, `PrecioCompra` | Precios del producto. |
| `PrecioLibre` | Servicio cuyo precio y descripción se capturan en cada venta. |
| `Stock` | Cuánto hay. Se calcula a partir de los movimientos; no se guarda a mano. |
| `Foto`, `Video`, `Galeria`, `Destacado` | Imágenes, videos y catálogo público. |

## Ventas y cobro
| Término | Qué es |
|---|---|
| `Venta` | Documento con partidas y pagos. Puede estar cobrada, o sin cobrar si empezó como pedido o cotización. |
| `Ticket` | Comprobante de una venta para el cliente. |
| `FormaPago` | Cómo paga el cliente: efectivo, tarjeta, transferencia, dólares, monedero… (catálogo `c_FormaPago` del SAT). |
| `MetodoPago` | Para el SAT: en una exhibición (PUE) o en parcialidades (PPD). **No** es efectivo/tarjeta. |
| `Pago` | Monto recibido con una forma de pago. Una venta puede tener varios. |
| `Cambio` | Lo que se le regresa al cliente. |
| `Comision` | Cargo por cobrar con cierta forma de pago (p. ej. tarjeta). |
| `Moneda`, `TipoCambio` | Moneda de un pago y su conversión a la moneda del negocio. |
| `Impuesto`, `IVA` | Impuestos de la venta. IVA de 16 %, u 8 % en la región fronteriza. |
| `Devolucion` | El cliente regresa mercancía de una venta. |
| `Cancelacion` | Anular una venta mal capturada; el inventario regresa y la venta se conserva marcada, con su motivo. |
| `Reembolso` | Lo que se le regresa al cliente en una devolución. |
| `Pedido` | Venta que el cliente se comprometió a recoger y todavía no se cobra o no se entrega (p. ej. mandó por WhatsApp qué imprimir). Se puede editar hasta cobrarse. |
| `Cotizacion` | Venta sin cobrar que solo informa precios (una lista de útiles); tal vez nunca se compre. Vence a los N días y se archiva. |
| `IngresoTrasladado` | Servicio cobrado a su costo: entra y sale de caja sin ganancia. |

## Inventario
| Término | Qué es |
|---|---|
| `Merma` | Salida sin venta (dañado, perdido, consumo). |
| `IngresoSinCompra` | Entrada de mercancía sin compra registrada. |
| `Ajuste` | Corrección de inventario (merma o ingreso sin compra), con su motivo. |
| `Conteo` | Contar físicamente una parte del inventario (una ubicación, una categoría) y corregir las diferencias con ajustes. |
| `Ubicacion` | Etiqueta del estante (o sección) donde puede estar un producto: "Mostrador", "Estante 1", "A-3". Libre y opcional; un producto puede estar en varias. |
| `Extravio` | Motivo de merma: se perdió y no se sabe cómo. |
| `Costo` | Costo de la mercancía por promedio móvil ponderado. |
| `Margen` | Precio menos costo (bruto). |

## Clientes, proveedores y compras
| Término | Qué es |
|---|---|
| `Cliente` | Persona registrada, con monedero. |
| `Veto` | Restricción temporal a un cliente: no se le reciben pedidos a distancia, solo se le vende en la tienda. Vence y se levanta sola. |
| `Monedero` | Saldo del cliente: lo que gana al comprar y lo que usa como pago. |
| `Abono` | Lo que una partida de venta suma al monedero de un cliente; vence. |
| `Canje` | Uso del monedero como forma de pago; sale de los abonos que vencen primero. |
| `Proveedor` | A quien se le compra. |
| `Contacto` | Nombre y teléfono de una venta sin cliente registrado (un pedido o una cotización de un extraño), para encontrarla. En la v1 era la agenda de clientes y proveedores. |
| `Compra` | Mercancía recibida de un proveedor, con su factura. |
| `OrdenCompra` | En la v1, el pedido a un proveedor. En la v2 no es un documento aparte: es una compra que todavía no se recibe (diseño 02). |
| `Factura` | Comprobante fiscal (CFDI en México), emitido o recibido. |
| `RFC` | Identificador fiscal en México. |
| `Paqueteria` | Envío de una compra. |
| `Folio` | Número consecutivo de un documento (compra, venta) para referirse a él. |
| `Partida` | Cada renglón de un documento (pedido, recepción, venta): qué producto, cuánto y a qué precio. |
| `Recepcion` | Una entrega de mercancía de un proveedor; una compra puede tener varias. |
| `Regalo` | Mercancía que el proveedor dio sin costo; entra al inventario con costo 0. |

## Finanzas
| Término | Qué es |
|---|---|
| `MovimientoFinanciero` | Entrada o salida de dinero que no es venta ni compra. |
| `Concepto` | Clasificación de un movimiento financiero. |
| `Caja`, `Banco` | Dónde se mueve el dinero. La caja es el cajón físico de efectivo. |
| `PuntoVenta` | Dispositivo desde el que se cobra (computadora, iPad, Elo). Varios pueden cobrar a la misma caja. |
| `Corte` | Contar el efectivo de la caja (por moneda), compararlo con lo esperado y, si se quiere, retirar dinero. |
| `Sobrante`, `Faltante` | Diferencia de un corte: se contó más o menos de lo esperado. |
| `Socio` | Dueño que aporta o retira dinero. |

## Impresión
| Término | Qué es |
|---|---|
| `Impresora` | Equipo de copias. |
| `Toner` | Consumible de una impresora, con contadores de impresión. |
| `Copia` | Servicio de impresión (BN o color, por tamaño). |

## La red
| Término | Qué es |
|---|---|
| `Nodo` | Una instancia del sistema (un negocio) conectada con otras. |

## Casos de frontera
- `Stock`, `Kit` e `IVA` se dicen así en la tienda: son del glosario aunque no sean palabras en español.
- `Aviso`, `Carrito`, `Boton`, `Linea` **no** son del negocio: van `Toast`, `Cart`, `Button`, `Line`.
- Fechas y horas son técnicas: `created_at`, `factura_date`, no `fecha_factura`.
- Los **permisos** van en español aunque sean verbos (`Vender`, `CancelarVenta`, `VerCostos`; en la base `vender`, `cancelar_venta`): son la lista de acciones del negocio, se guardan como texto y así se leen en los roles.
- "Activo", "nuevo", "página", "sección", "entorno" y "datos" **no** son del negocio: `is_active`, `deactivated_at`, `NewUsuario`, `UnidadesPage`, `UnidadesSection`, `Env`. `Descontinuado` sí lo es (un producto que ya no se ofrece).
- `Venta` y `Ajuste` son documentos separados (en la v1 compartían tabla).
