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
| `Rol`, `Permiso` | Qué puede hacer cada usuario (vender, comprar, ajustar inventario, ver costos, administrar usuarios). |
| `Pais` | Define moneda, impuestos, facturación y formas de pago locales. |

## Productos
| Término | Qué es |
|---|---|
| `Producto` | Artículo del catálogo. Tiene un número corto para buscarlo en caja. |
| `Servicio` | Producto que no lleva stock (copias, enmicado, recargas, trámites). |
| `Kit` | Producto que se arma con otros; su stock es cuántos alcanzan a armarse. |
| `Componente` | Producto que forma parte de un kit, con su cantidad en la receta. |
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
| `Venta` | Ticket cobrado. |
| `Ticket` | Comprobante de una venta para el cliente. |
| `FormaPago` | Cómo paga el cliente: efectivo, tarjeta, transferencia, dólares, monedero… (catálogo `c_FormaPago` del SAT). |
| `MetodoPago` | Para el SAT: en una exhibición (PUE) o en parcialidades (PPD). **No** es efectivo/tarjeta. |
| `Pago` | Monto recibido con una forma de pago. Una venta puede tener varios. |
| `Cambio` | Lo que se le regresa al cliente. |
| `Comision` | Cargo por cobrar con cierta forma de pago (p. ej. tarjeta). |
| `Moneda`, `TipoCambio` | Moneda de un pago y su conversión a la moneda del negocio. |
| `Impuesto`, `IVA` | Impuestos de la venta; los define el país. |
| `Devolucion` | Regreso de productos de una venta. |
| `Pedido` | Encargo o cotización previa a la venta; puede llegar en línea. |
| `IngresoTrasladado` | Servicio cobrado a su costo: entra y sale de caja sin ganancia. |

## Inventario
| Término | Qué es |
|---|---|
| `Merma` | Salida sin venta (dañado, perdido, consumo). |
| `IngresoSinCompra` | Entrada de mercancía sin compra registrada. |
| `Ajuste` | Corrección de inventario (merma o ingreso sin compra). |
| `Costo` | Costo de la mercancía por promedio móvil ponderado. |
| `Margen` | Precio menos costo (bruto). |

## Clientes, proveedores y compras
| Término | Qué es |
|---|---|
| `Cliente` | Persona registrada, con monedero. |
| `Monedero` | Saldo del cliente: lo que gana al comprar y lo que usa como pago. |
| `Proveedor` | A quien se le compra. |
| `Contacto` | Cliente o proveedor en la agenda. |
| `Compra` | Mercancía recibida de un proveedor, con su factura. |
| `OrdenCompra` | Pedido a un proveedor antes de recibirlo. |
| `Factura` | Comprobante fiscal (CFDI en México), emitido o recibido. |
| `RFC` | Identificador fiscal en México. |
| `Paqueteria` | Envío de una compra. |

## Finanzas
| Término | Qué es |
|---|---|
| `MovimientoFinanciero` | Entrada o salida de dinero que no es venta ni compra. |
| `Concepto` | Clasificación de un movimiento financiero. |
| `Caja`, `Banco` | Dónde se mueve el dinero. |
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
- Palabras pendientes de decidir con el diseño de la base: si `Venta` y `Ajuste` comparten tabla como en la versión uno o se separan.
