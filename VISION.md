# Visión — ro_inventario (versión dos)

Sistema de inventario y punto de venta para negocios chicos que compran mercancía y la revenden, y que ofrecen servicios. Se rehace desde cero en Rust, tomando lo aprendido en la versión uno (`inventario_papeleria`, ASP.NET) y en `xplaya` (Rust); rutas en [Proyectos relacionados](#proyectos-relacionados). La papelería "El Gordo" es el primer negocio y el laboratorio.

Este documento dice **qué** es el sistema y **por qué**. Cada decisión concreta (cómo, y qué se descartó) va en su propia nota en `docs/decisiones/`.

## Por qué una versión dos

- **La versión uno funciona, pero se enredó al crecer.** Una sola clase de ventas llegó a ~1,460 líneas; reglas del negocio viven en vistas SQL, en TypeScript y en C# a la vez; la misma regla se escribió dos veces en dos repos.
- **Se aprendió mucho del negocio.** Reglas que costó años descubrir hoy están escritas y probadas: costo por promedio móvil, stock de kits, comisión iterativa de tarjeta, monedero, la trampa del JOIN en pagos. Ese conocimiento es el activo; el código se puede rehacer.
- **Lo que viene necesita otra base.** El efectivo se sigue usando y se seguirá usando, pero tiende a reducirse: hay iniciativas para restringirlo en ciertos pagos y crecen los pagos digitales (tarjeta, transferencia; en México SPEI y CoDi). Los negocios chicos van a necesitar cobrar de muchas formas, facturar y reportar, y conectarse con sus proveedores y clientes. La versión uno no se diseñó para eso.
- **Hacer software se volvió barato; tener el entorno, no.** Aquí se controla el negocio, el hardware, el cluster y los datos reales. Es un laboratorio vivo.

## Qué es

**Dos aplicaciones en un mismo repositorio**, sobre un núcleo compartido:

| Aplicación | Para quién | Qué hace |
|---|---|---|
| **Privada** (punto de venta) | quienes atienden el negocio | ventas, compras, inventario, clientes, costos, reportes |
| **Pública** (catálogo) | clientes y otros sistemas | catálogo, fichas de producto y servicio, pedidos en línea, monedero, recibos |

- Las dos comparten el núcleo del negocio: una regla vive en un solo lugar y las dos la usan.
- Son dos programas y dos imágenes separadas. La pública **lee** el catálogo y lo que el cliente consulta (monedero, recibos), y **solo escribe una cosa**: los pedidos en línea, que llegan como bandeja de entrada para que el negocio los revise. Su usuario de BD tiene exactamente esos permisos, y no puede depender de los demás módulos que escriben: el compilador lo impide.
- **Un negocio = una instancia y una base de datos propias.** El código atiende siempre a un solo negocio; dar de alta otro es configuración y despliegue, no código. Ver [la decisión](docs/decisiones/2026-09-29-una-instancia-por-negocio.md).

## Principios

### El negocio es el protagonista
- Cada negocio vale por sí solo con el sistema, aunque nunca se conecte con nadie.
- Lo que un negocio ofrece (productos y servicios) se publica en formatos que personas, sistemas y asistentes de IA pueden leer: HTML, `schema.org`, RSS, API, MCP. Estándares abiertos antes que protocolos propios.
- Los negocios se conectan por la red (API), nunca compartiendo tablas, aunque vivan en el mismo servidor. La cadena de suministro se diseña como mensajes entre nodos: la orden de compra de uno es el pedido de otro.

### Entendible por su dueño
- El dueño diseña (base de datos, pantallas, reglas); la IA implementa **por pasos chicos** que se puedan revisar en unos 15 minutos.
- Antes de programar, un plan corto en español que el dueño aprueba. Después, qué cambió y cómo probarlo.
- **Tecnología aburrida.** Nada nuevo (librería, patrón, herramienta) sin platicarlo antes.

### SOLID, a la manera de Rust
| Principio | Cómo se aplica aquí |
|---|---|
| **Una responsabilidad** | Un crate por área del negocio (ventas, compras, inventario…). Un módulo o función, una razón para cambiar. Si un archivo pasa de ~300 líneas, se revisa si mezcla cosas. |
| **Abierto/cerrado** | Lo que cambia entre negocios se resuelve con configuración que elige piezas al arrancar (valor → módulo prendido/apagado → trait con varias implementaciones). Nunca `if negocio == …`. |
| **Sustitución** | Toda implementación de un trait cumple el mismo contrato; las pruebas del contrato corren contra cada implementación (la real y la de pruebas). |
| **Interfaces pequeñas** | Traits chicos y específicos (`RepoVentas`, `ProveedorTipoCambio`), no uno gigante por capa. |
| **Inversión de dependencias** | El núcleo del negocio define los traits que necesita (base de datos, SAT, pagos, otros nodos) y no conoce su implementación. **La dirección la vigila el compilador:** el núcleo no tiene como dependencia ni la BD ni la web, así que meter SQL o HTTP en una regla del negocio ni siquiera compila. |

### Pruebas desde el primer commit
- **Unitarias** (`cargo test`) para toda regla del negocio, en el núcleo puro, sin base de datos.
- **De integración** contra un Postgres real de pruebas, para repositorios y migraciones.
- **E2E con Playwright** para las dos aplicaciones, en los dos puntos de venta reales: escritorio con Firefox y el ancho del iPad mini (744 px).
- Todo bug corregido deja su prueba: primero la que falla, luego el arreglo.
- Las pruebas describen el comportamiento en español; leer sus nombres dice qué hace el sistema.
- CI corre todo en cada push; no se despliega en rojo.

### Nombres
- Vocabulario de programación en inglés; sustantivos del negocio en español. La frontera es el glosario: si la palabra está ahí, español; si no, inglés (`get_ventas_by_cliente`, `struct Venta`, `enum MetodoPago`).
- Sin acentos en identificadores. Comentarios, pantallas, documentación y commits en español.

### El código se desenreda, no se enreda
- Los archivos grandes no crecen: lo nuevo va en su propio módulo.
- Refactor y funcionalidad van en commits separados.
- Decisiones de diseño en `docs/decisiones/`, una nota por decisión.

## Stack propuesto

Base: lo que ya funciona en `xplaya`. Cada pieza se confirma en su decisión.

- **Rust** (edición 2024), **Axum**, **sqlx** con **PostgreSQL**, plantillas del lado del servidor.
- Despliegue en el cluster k3s propio con ArgoCD, como la versión uno.
- La interactividad del punto de venta (carrito, pagos) es la decisión técnica más delicada: se toma aparte, con prototipo, antes de construir la pantalla de venta.

## Módulos

**Núcleo (el corte mínimo):** usuarios y permisos, productos y servicios, inventario (stock), ventas con sus formas de pago, compras, clientes, proveedores, costo por promedio móvil.

- **Usuarios y permisos:** se diseñan para negocios con empleados (quién puede vender, comprar, ajustar inventario, ver costos, administrar usuarios), aunque la papelería use solo dos perfiles. El diseño es completo desde el principio; las pantallas para configurarlo pueden esperar a que un negocio las necesite.

**Después del corte**, en el orden que pida el negocio: pedidos, monedero, kits, catálogo público completo, finanzas, impresoras y tóners, facturación electrónica (CFDI), pagos con terminal integrados, conexión entre negocios (la red).

## Pagos, y solo México

- **El efectivo es una forma de pago más, no el centro.** Una venta se cobra con una o varias formas de pago; cada forma tiene sus reglas (si da cambio, si lleva comisión, si requiere referencia o confirmación) y agregar una nueva no toca las demás.
- **El sistema es para México:** pesos, IVA (16 %, u 8 % en la región fronteriza), CFDI, SPEI, CoDi, tipo de cambio de Banxico y horarios de México. No se diseña para otros países ni para varias monedas: sería complejidad sin un caso real. Si algún día hay un negocio en otro país, se agrega como extensión.
- **Los servicios externos van detrás de un trait** (Banxico, y después el SAT o el PAC y los pagos). No es para cambiar de país, sino para probar sin llamar al servicio real.

## El corte y la migración

- **La papelería pasa a la versión dos cuando pueda capturar ventas y compras.** Al inicio se captura en paralelo en las dos versiones para detectar diferencias.
- **La migración de datos se escribe desde el primer día**, no al final: un script que convierte la base de la versión uno a la nueva y que se corre seguido contra una copia de producción. Las inconsistencias salen mientras se diseña, cuando es barato corregirlas.
- **La versión uno no se congela formalmente:** el dueño es el único desarrollador y decide. Si por fuerza mayor cambia algo en la uno, lo avisa en la sesión de trabajo, para que la dos lo contemple.
- Producción de la versión uno sigue intocable: solo se lee una copia.

## Qué no es

- No es un ERP para todo tipo de empresa: es para negocios chicos de compra, venta y servicios.
- No compite con WhatsApp, TikTok o los marketplaces: los aprovecha como canales, sin depender de ellos para lo esencial.
- No es internacional: es para negocios en México.
- No es un port de la versión uno: se rediseña desde la base de datos. El intento de port línea por línea sobre la misma base quedó archivado en la etiqueta `intento-2026-05`.

## Hacia dónde va

La visión larga, la **Red Comercial**, está en [`docs/red-comercial.md`](docs/red-comercial.md): negocios como nodos que publican lo que ofrecen y se conectan en cadenas de suministro y redes, con lazos fuertes entre sistemas amigos y lazos débiles hacia el resto del ecosistema. La versión dos se diseña para que ese futuro sea posible, sin construirlo antes de tiempo.

## Lecciones que se traen

- De la versión uno: el costo se calculó mal durante años (promedio simple); reglas escondidas en vistas SQL; una clase de ventas que lo hacía todo; configuración leída "del primer formulario de la página" que se rompió al agregar otro; fines de línea mezclados. Cada una tiene hoy una regla que la evita.
- Del intento `intento-2026-05`: se detuvo porque la versión uno siguió recibiendo funcionalidades nuevas y la de Rust se fue quedando atrás; además no había ningún cliente a la vista y no se veía como algo que le sirviera a alguien más. Por eso la dos tiene un propósito claro desde el inicio, y la uno solo cambia por necesidad. Ese propósito apareció solo: EKA llegó a partir de una demo del sistema a un amigo. Mostrar algo que funciona atrae interés.
- Los roles de usuario: la versión uno nació con roles pensando en otros negocios, pero en la papelería nunca se usaron (dos personas; solo se separa quién crea usuarios) y quedaron mal manejados. Las funcionalidades se agregaban según el uso diario, sin pensar en negocios con más empleados. Aquí los usuarios y permisos se diseñan bien desde el principio, aunque la papelería use lo mínimo.
- De `xplaya`: un sistema hecho casi todo por IA funciona, pero su dueño participó poco y lo entiende menos. Por eso aquí el dueño diseña.

## Proyectos relacionados

| Proyecto | Ruta local | Repositorio | Qué es |
|---|---|---|---|
| Versión uno | `/home/ro/code/inventario_papeleria` | `github.com/rogithub/inventario_papeleria` | Punto de venta en ASP.NET, en producción en `papeleria.xplaya.com`. Sus reglas del negocio están en `CLAUDE.md`, `docs/decisiones/` y `docs/glosario.md` |
| Catálogo público actual | `/home/ro/code/xplaya` | `github.com/rogithub/xplaya` | Catálogo en Rust, en producción en `xplaya.com` |
| Infraestructura | `/home/ro/code/k3s-manifests` | `github.com/rogithub/k3s-manifests` | Manifests del cluster k3s y ArgoCD |
| Intento anterior | este repo, etiqueta `intento-2026-05` | — | Port de la versión uno a Rust, archivado |
