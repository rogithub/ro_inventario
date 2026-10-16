# El mínimo viable: catálogo, compras, pedidos, ventas, monedero y corte; con MCP donde tenga sentido

**Fecha:** 2026-10-14 · **Estado:** reemplazada en parte por [API primero](2026-10-16-api-primero.md): el contrato interno es la API, el teléfono entra por la computadora del mostrador (no por un MCP externo de la v2), el orden cambia y solo algunas áreas llevan pantalla. Lo que entra al mínimo viable sigue igual.

## Contexto
La papelería ya trabaja con un asistente (Claude Code en la computadora del mostrador) y dos MCP armados sin planear, uno encima de la v1 y otro para imprimir y acomodar fotos. Funciona, y mostró que la persona prefiere pedir a llenar formularios ([visión](../ia-como-interfaz.md)). El dueño quiere enfocar la v2 en un producto mínimo viable: seguir atendiendo a proveedores (dar de alta lo que llega) y seguir vendiendo, y delinear lo demás después. `VISION.md` ya dice que la papelería pasa a la v2 cuando pueda capturar ventas y compras; faltaba el orden y el lugar del asistente.

## Decisión
- **El mínimo viable reemplaza a la v1 en la caja.** Por eso entran el monedero (los clientes cuentan con él) y el corte de caja (vender sin corte repite los faltantes de la v1).
- **El orden:**
  1. Terminar el catálogo: los productos migrados (7d2b) y las presentaciones con códigos de barras (7e). Sin códigos no se vende con escáner.
  2. **Compras** ([diseño 02](../diseno/02-compras.md)): el documento único que nace como pedido al proveedor y recibe entregas. Con el kárdex. Las facturas y notas del proveedor se adjuntan a la compra.
  3. **Pedidos**: el centro del día, porque coordinar los de WhatsApp es lo difícil. Con cliente, estados (`preparando → listo → avisado → cobrado`), edición y documentos adjuntos. Convertir un pedido en venta es un botón.
  4. **Ventas y cobro** ([diseño 04](../diseno/04-ventas.md)), con los servicios que necesita ([diseño 03](../diseno/03-servicios.md)): las copias son servicios.
  5. **Monedero** ([diseño 06](../diseno/06-clientes-y-monedero.md)) y **corte de caja** ([diseño 05](../diseno/05-corte-de-caja.md)).
- **Cobrar se queda en pantalla**, en sus dos variantes: escritorio y táctil.
- **MCP donde tenga sentido**, dentro de cada área y no al final:
  - consultar productos, precio y existencia;
  - en compras, registrar lo que llega desde la factura o la nota;
  - en pedidos, crear, editar, cambiar estado y cotizar con el total del servidor.
- **El MCP externo se expone a internet** con autenticación y permisos por rol, para poder hablarle al asistente desde el teléfono. Es un adaptador ligero: no tiene base ni reglas, y delega en un contrato interno de la privada.
- **El hardware del mostrador se maneja con una cola de órdenes** que el agente de esa computadora recoge; la computadora no abre puertos.
- **Se pospone:** gráficas, el módulo de impresiones, el tóner leído por SNMP y el catálogo público completo.

## Descartado
- **Que el MCP externo llame directo a los crates y a la base:** se volvería un segundo núcleo expuesto a internet. Lo que se expone debe ser lo mínimo y poder borrarse sin tocar lo demás.
- **Un MCP interno como contrato entre el núcleo y la computadora del mostrador:** MCP es un idioma para que lo lea una IA. Entre programas propios, un contrato con tipos que revisa el compilador, y una cola que no exige abrir puertos. Los MCP locales de esa computadora siguen existiendo para Claude Code ahí mismo.
- **Que el asistente cobre:** cobrar se queda en pantalla, donde la persona ve el total y el cambio.
- **Venta de mostrador antes que pedidos:** el pedido que se convierte en venta cubre las dos; lo difícil es coordinar WhatsApp.
- **Convivir con la v1 para vender y usar la v2 solo para compras y catálogo:** dos sistemas de venta duplican el trabajo y las diferencias.

## Consecuencias
- El plan de pasos después del 7 está en [`docs/pendientes/ruta-minimo-viable.md`](../pendientes/ruta-minimo-viable.md).
- Cada área lleva, además de sus pantallas, su parte del contrato interno y sus herramientas MCP. El MCP externo llega cuando haya algo que valga la pena pedirle.
- Piezas nuevas que llevan su propia decisión antes de construirse: librería de MCP, OAuth, eventos en vivo a la pantalla, dónde se guardan los documentos y la cola de órdenes del mostrador.
- [Las pantallas las arma el servidor](2026-09-30-pantallas-del-servidor.md) sigue vigente: las pantallas se adelgazan a lo que de verdad necesita el mostrador, no desaparecen.
