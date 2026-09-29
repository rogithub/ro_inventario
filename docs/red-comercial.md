# Red Comercial — visión

> Visión a largo plazo, no un plan. Surgió de un sueño del dueño (2026-09-28); aquí queda ordenada con lo que ya existe, lo débil y las oportunidades. Nada de esto se construye sin platicarlo; cada paso tiene que ser útil por sí solo.
> Traída de la versión uno (`/home/ro/code/inventario_papeleria/ventas/red-comercial.md`), donde se escribió. Relacionado, en la versión uno: `/home/ro/code/inventario_papeleria/ventas/modulos.md`, `/home/ro/code/inventario_papeleria/ventas/propuesta_seguridad_industrial.md`, `/home/ro/code/inventario_papeleria/docs/decisiones/2026-09-27-personalizacion-por-cliente-sin-ifs.md`.

## La idea

Una red donde el protagonista no es una persona sino un **negocio**.

- **Lo que se publica es lo que se ofrece:** productos y servicios, no opiniones ni fotos de viajes.
- **La publicación no es solo texto:** puede ser un feed RSS, una API, un servidor MCP. Formas que otras personas, sistemas y asistentes de IA pueden leer y usar.
- **La célula es el inventario:** hoy la versión uno con su catálogo público (xplaya.com); mañana este sistema.
- **Dos tipos de conexión:**
  - **Lazos fuertes:** entre negocios que usan el mismo sistema. Mi venta es tu compra; así se forman cadenas de suministro y, con el tiempo, redes que se entrelazan.
  - **Lazos débiles:** hacia afuera (WhatsApp, TikTok, buscadores, otras redes comerciales). No se compite con ellos: se aprovechan para ganar alcance y capacidades.
- **La diferencia con las plataformas grandes:** WhatsApp nació para comunicarse y TikTok para videos; el comercio llegó después, como una capa encima. Aquí es al revés: el comercio es el centro y lo social sale de él. No se pretende competir con ellas, sino atacar el problema desde adentro del negocio.

## Lo que ya existe

| Pieza | Dónde está | Qué aporta a la red |
|---|---|---|
| Inventario, ventas, compras, costos | versión uno | La célula: cada nodo vale por sí solo aunque nunca se conecte |
| Catálogo público | xplaya.com (galería, fichas, sitemap, marcado `schema.org`) | Ya publica lo que se ofrece en un formato que los buscadores entienden |
| `OrdenCompra` y `Pedido` | versión uno | **Los dos extremos de una misma transacción**: mi orden de compra a un proveedor sería su pedido de un cliente. Existen, pero no se hablan entre negocios |
| Pedidos en línea (`OrigenPedido = EnLinea`) | versión uno + xplaya | Un canal de entrada que no pasa por el mostrador |
| Varios negocios con el mismo código | la escalera de configuración, módulos | La base para que haya más de un nodo |
| EKA (seguridad industrial) | propuesta en `/home/ro/code/inventario_papeleria/ventas/` | El posible segundo nodo, y con él la primera arista real |
| WhatsApp | uso diario, fuera del código | Canal con clientes y transporte de archivos (impresiones, actas y trámites) |
| Wifi gratis en tienda | la tienda | Suma contactos de WhatsApp de forma natural |
| Estados de WhatsApp | uso diario | Escaparate: los contactos ven los videos de productos |
| TikTok | videos ligados a cada producto en la galería | CDN gratuito, herramientas de edición y videos descargables para los estados |

## Puntos débiles

- **Los servicios se comunican mal.** En el sistema un servicio es un renglón ("SERVICIO MICA TERMICA A4"): no dice qué es, qué hay que traer o mandar por WhatsApp, cuánto tarda ni cómo se cobra. Para una red donde lo que se publica es lo que se ofrece, es la carencia principal.
- **La cadena de suministro es incipiente.** Las compras se capturan a mano desde la factura del proveedor; ningún sistema le habla a otro.
- **La red, como tal, todavía es incierta.** Un nodo solo no es red. Las redes viven de que otros se unan (el problema del arranque en frío), y hoy hay un nodo y un prospecto.
- **Los lazos débiles dependen de plataformas ajenas.** WhatsApp y TikTok pueden cambiar sus reglas, cobrar o cerrar funciones en cualquier momento; lo que se construya encima no es nuestro.
- **La integración con WhatsApp es manual.** Los archivos y pedidos llegan por chat y se pasan a mano al sistema.
- **Riesgo de sobreingeniería.** Es el tipo de proyecto que puede volverse un sistema grande e ininteligible. Va contra las reglas de trabajo ([`VISION.md`](../VISION.md); en la versión uno, `/home/ro/code/inventario_papeleria/CLAUDE.md`) (refactor progresivo, nada nuevo sin platicarlo).
- **Recursos mínimos.** Un laboratorio chico: dos personas en la tienda y un programador.

## Oportunidades

- **Publicar en formatos que otros ya leen, sin inventar protocolos:**
  - `schema.org` (ya se usa)
  - RSS
  - el catálogo de WhatsApp Business
  - feeds de comercio de los buscadores
  - un servidor MCP para asistentes de IA

  Conecta sin pedirle permiso a nadie. Un formato propio obligaría a convencer a todos de usarlo.
- **Asistentes de IA como clientes.** Un servidor MCP de solo lectura del catálogo permitiría que cualquier asistente responda "¿tienen mica tamaño carta y cuánto cuesta?". Es la "publicación que no es texto" en su forma más chica.
- **El primer lazo fuerte con EKA.** Que una orden de compra en un sistema llegue como pedido al otro. La papelería puede surtirle a EKA o comprarle.
- **Los servicios como ofertas completas.** Una ficha por servicio en xplaya (qué es, qué traer o mandar, tiempo, precio) sirve desde el primer día para compartir por WhatsApp. También es la base de cualquier "publicación" futura.
- **Negocios de la zona como nodos naturales:** proveedores locales y negocios que ya le compran a la papelería.
- **Cada nodo vale solo.** Si el negocio funciona mejor con el sistema aunque nunca se conecte con nadie, la red no tiene que existir para que valga la pena, y crece solo si hay razones reales.

## Posibles primeros pasos

Cada uno es útil aunque la red nunca llegue:

1. **Fichas de servicio en xplaya:** qué es, qué traer o mandar, tiempos, precio.
2. **Servidor MCP de solo lectura del catálogo:** productos, precio y si hay existencia.
3. **Orden de compra → pedido entre dos sistemas**, cuando EKA sea cliente.

## Principios

- Cada nodo vale por sí solo; la red es un premio, no un requisito.
- Estándares abiertos antes que protocolos propios.
- Aprovechar las plataformas grandes sin depender de ellas: lo esencial (catálogo, clientes, pedidos) vive en nuestro sistema.
- Pasos chicos, entendibles y reversibles, con las mismas reglas que el resto del código.
