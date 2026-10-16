# La ruta al mínimo viable, después del paso 7

**Salió de:** conversación con el dueño, 2026-10-14; rehecha el 2026-10-16 con [API primero](../decisiones/2026-10-16-api-primero.md) · **Cuándo:** al terminar cada paso; se borra cuando la papelería venda en la v2

## Qué pasa
El mínimo viable ([decisión](../decisiones/2026-10-14-minimo-viable.md)) es catálogo, compras, pedidos, ventas, monedero y corte. Con [API primero](../decisiones/2026-10-16-api-primero.md), cada área nace como API con permisos y MCP delgado. Solo ventas, corte y aprobar compras llevan pantalla. Cada área es más grande que un paso de unas 500 líneas.

## Por qué importa
Para no perder el orden entre sesiones, y para que cada paso empiece por su diseño y por sus operaciones, no por la pantalla.

## Propuesta
Cada punto se parte como el paso 7, con su plan aprobado:

1. **Terminar el paso 7** ([plan](paso-7-productos.md)):
   - **7d2b:** escribir los productos y el historial de precios, adelantar la secuencia del NID y cuadrar.
   - **7e:** presentaciones y códigos de barras, con su migración, **sin pantalla nueva**. Su captura se decide con compras o ventas.
2. **Piloto de API y MCP con unidades de medida:** listar y crear, de punta a punta con Claude Code contra la base de desarrollo, sin tocar la v1. Fija lo que usarán las demás áreas, cada pieza con su nota:
   - autenticación;
   - permisos de lectura y escritura;
   - forma de los errores;
   - registro «por el asistente»;
   - librería de MCP.

   Seguramente son dos pasos: la base de la API con unidades, y después el MCP. Al cerrarlo se revisa esta ruta con lo que costó.
3. **Categorías y productos por API y MCP:**
   - buscar con paginación y existencia;
   - consultar un producto con sus presentaciones y precios;
   - dar de alta;
   - cambiar el precio.
4. **Compras** ([diseño 02](../diseno/02-compras.md)):
   - proveedores;
   - la lista de lo que hay que pedir, armada platicando por el MCP interno;
   - la pantalla para aprobar la compra;
   - las recepciones, el kárdex y el costo;
   - el alta de productos nuevos desde la compra;
   - las facturas y notas del proveedor como documentos adjuntos (antes, decidir dónde se guardan).
5. **Pedidos por API y MCP:**
   - cliente por teléfono;
   - estados (`preparando → listo → avisado → cobrado`);
   - edición;
   - cotización con el total del servidor;
   - documentos adjuntos.

   Revisar la lista de requisitos de [la visión](../ia-como-interfaz.md#lo-que-pide-el-mcp-de-la-v2).
6. **Ventas y cobro en pantalla** ([diseños 03 y 04](../diseno/04-ventas.md)), en escritorio y táctil. Antes, la decisión de la interactividad, con prototipo (`VISION.md`). El pedido se convierte en venta con un botón.
7. **Monedero y corte de caja** ([diseños 05](../diseno/05-corte-de-caja.md) y [06](../diseno/06-clientes-y-monedero.md)), con la migración de clientes y saldos desde la v1. El corte, en pantalla. El saldo del monedero, también por API.

**Para después del mínimo viable:**
- reportes por API;
- la pantalla del proveedor;
- un tablero (Superset u otro);
- la versión ligera sin cluster.

**El MCP de la v1 sigue siendo el piloto** mientras tanto: el teléfono entra por el relevo del POC a la computadora del mostrador, con las herramientas de la v1. Las de la v2 se prueban aisladas, en desarrollo. Reemplazan a las de la v1 en esa computadora cuando sean estables, con su propio paso y el visto bueno del dueño.

**Por decidir con el dueño, cuando llegue:**
1. **El corte de caja:** ¿lleva también herramientas MCP («¿cómo va la caja?»), o es solo pantalla?
