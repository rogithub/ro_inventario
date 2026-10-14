# La ruta al mínimo viable, después del paso 7

**Salió de:** conversación con el dueño, 2026-10-14 · **Cuándo:** al terminar cada paso; se borra cuando la papelería venda en la v2

## Qué pasa
El dueño fijó el mínimo viable ([decisión](../decisiones/2026-10-14-minimo-viable.md)): catálogo, compras, pedidos, ventas, monedero y corte, con MCP donde tenga sentido. Cada área es más grande que un paso de ~500 líneas.

## Por qué importa
Para no perder el orden entre sesiones, y para que cada paso nuevo empiece por su diseño y no por la pantalla.

## Propuesta
Cada punto se parte como el paso 7, con su plan aprobado:

1. **Paso 7:** 7d2b (escribir productos y precios, secuencia del NID, cuadre) y 7e (presentaciones y códigos de barras).
2. **Compras** (diseño 02): proveedores, la compra con sus recepciones, el kárdex y el costo. Las facturas y notas como documentos adjuntos (decidir antes dónde se guardan). Alta de productos nuevos desde la compra (el formulario de 7b2a ya se pensó para reusarse).
3. **Pedidos:** cliente por teléfono, estados, edición, cotización con el total del servidor, documentos adjuntos. Revisar la lista de requisitos de [la visión](../ia-como-interfaz.md#lo-que-pide-el-mcp-de-la-v2).
4. **Ventas y cobro** (diseños 03 y 04), en escritorio y táctil. Antes, la decisión de la interactividad, con prototipo (`VISION.md`). El pedido se convierte en venta con un botón.
5. **Monedero y corte de caja** (diseños 05 y 06), con la migración de clientes y saldos desde la v1.
6. **MCP externo:** contrato interno, OAuth y herramientas de las áreas ya hechas. Antes, la prueba de 5 minutos del modo de voz.

Por decidir con el dueño, cuando llegue: si el MCP externo se adelanta en cuanto haya pedidos (para que la dueña lo use ya) o espera a ventas.
