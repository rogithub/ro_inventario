# API primero: la API con permisos es el contrato, los MCP son capas delgadas encima y las pantallas solo donde hacen falta

**Fecha:** 2026-10-16 · **Estado:** vigente

## Contexto
La v2 no tiene API: fuera de `/health`, todo lo que hace la privada sale como pantalla. La [decisión del mínimo viable](2026-10-14-minimo-viable.md) ya pedía un MCP ligero que delegara en un «contrato interno», pero no decía qué forma tenía ese contrato ni cuándo llegaba, y cada área seguía planeada pantalla por pantalla.

Mientras tanto, la papelería ya trabaja pidiéndole al asistente ([visión](../ia-como-interfaz.md)). La prueba del control remoto por voz (repo `MCP_papeleria_poc`) mostró que Cloudflare Access sirve de login para un conector MCP, también en el modo de voz del teléfono. El dueño propuso (2026-10-16) diseñar la v2 para usarse con IA a través de MCP delgados encima de una API con permisos bien delimitados, con menos pantallas. Ofreció empezar de cero si hacía falta.

## Decisión
- **La API HTTP con JSON de la privada es el contrato.** Cada operación es del negocio y tiene nombre (`crear unidad`, `cambiar precio`), no es un CRUD por tabla. Cada una:
  - pide un `Permiso` de la lista que ya existe (`crates/usuarios`);
  - responde errores que dicen qué dato falla y por qué, en español;
  - calcula el dinero en el servidor.

  El handler solo arma y conecta: las reglas siguen en los crates.
- **Quién llama viaja hasta adentro.** Llaman personas con su rol: administrador, vendedor, etc. Lo hecho por el asistente queda a nombre de esa persona, con sus permisos, y el registro dice que fue «por el asistente».
- **Los MCP de la v2 son capas delgadas:** una herramienta equivale más o menos a una llamada a la API. No tienen base de datos ni reglas. Las descripciones de las herramientas son parte del diseño y se revisan como las pruebas. Los usa el asistente de la computadora del mostrador, en la red de la tienda. Ahí se arma, platicando, la lista de lo que hay que pedir a los proveedores.
- **El MCP de la v1 y el de la v2 se separan:**
  - **El de la v1 es lo que hay hoy, y es el piloto.** Del teléfono a la computadora del mostrador, la capa pública (el relevo del POC, con Cloudflare Access) es delgada: solo deja entrar a la sesión de Claude Code de esa computadora. Ahí, las herramientas locales trabajan encima de la v1. Casi lo único que pueden hacer es crear pedidos: la v1 no se diseñó para MCP y no tiene roles.
  - **El de la v2 se prueba aislado,** en desarrollo, sin tocar la v1. Cuando sea estable, reemplaza a las herramientas de la v1 en la computadora del mostrador. El teléfono sigue entrando por la misma capa pública. Hasta entonces, la v1 no se toca.
- **Pantallas solo donde hacen falta:**
  - **Ventas y cobro**, en escritorio y táctil. Vender siempre es desde una pantalla, con una persona supervisando, por ahora.
  - **Corte de caja:** se cuenta dinero con las manos.
  - **Aprobar una compra**, a mano.
  - **Después:** que el proveedor vea lo que le pedimos.

  Lo demás nace sin pantalla: API y MCP.
- **Las pantallas que ya existen** (categorías, unidades de medida, productos) **se quedan como están, congeladas.** Siguen con sus E2E y no reciben funciones nuevas. Sirven de respaldo y para ver una lista con calma. Si alguna estorba, se retira con su propio cambio.
- **Las pantallas no consumen la API por HTTP.** Viven en el mismo binario y llaman a las mismas funciones de los crates, así que no se pueden contradecir.
- **Reportes como operaciones de la API**, que devuelven datos para que la IA los grafique. Un tablero aparte (p. ej. Superset) lleva su propia decisión cuando haga falta.
- **Se supone internet.** El laboratorio es la papelería. Una versión ligera, sin cluster, queda para después.
- **No se empieza de cero.** Se conservan los crates, la migración de la v1 y las pantallas hechas.

## Descartado
- **Empezar de cero:** las reglas, los tipos y la migración (unas 6,900 líneas en los crates, más la migración de la v1) son justo lo que la API necesita. Solo cambia la capa de afuera.
- **Un MCP gordo que llama a los crates y a la base:** sería un segundo núcleo, y expuesto. Ya se había descartado en la decisión del 2026-10-14.
- **Una API genérica (CRUD por tabla, GraphQL):** la IA necesita operaciones con nombre que validen y expliquen por qué rechazan, no tablas. Con tablas, la regla vuelve a vivir en el texto de una skill, que es lo que falló con la v1.
- **Rehacer las pantallas como una aplicación de una sola página encima de la API:** [las pantallas las arma el servidor](2026-09-30-pantallas-del-servidor.md) sigue valiendo para las que quedan.
- **Borrar las pantallas del catálogo:** funcionan, tienen pruebas y sirven de respaldo mientras el MCP se prueba en el mostrador.
- **Que el asistente cobre:** igual que el 2026-10-14, cobrar se queda en pantalla.

## Consecuencias
- **Reemplaza en parte a la [decisión del mínimo viable](2026-10-14-minimo-viable.md).** Siguen valiendo:
  - qué entra al mínimo viable;
  - el monedero y el corte;
  - la cola de órdenes del mostrador.

  Cambian:
  - el contrato interno, que ahora es esta API;
  - el orden, porque la API y su primer MCP van antes que compras;
  - qué áreas llevan pantalla.

  La ruta nueva está en [`docs/pendientes/ruta-minimo-viable.md`](../pendientes/ruta-minimo-viable.md).
- **El primer paso después del 7 es un piloto con unidades de medida:** API y MCP de punta a punta, con el Claude Code del mostrador. Ahí se deciden, cada una con su nota:
  - cómo se autentica la API (sesión, token por persona o el JWT de Access);
  - la forma de los errores;
  - la librería de MCP (`rmcp`, ya probada en el POC).

  El piloto también dice cuánto cuesta de verdad cada área con este rumbo.
- **Cada área lleva su API con pruebas** (por HTTP, con quién llama y sin permiso) **y sus herramientas MCP.** Solo las áreas de la lista llevan pantalla.
- **La regla de la visión «el catálogo y los precios no se cambian por el asistente remoto» sigue en pie** para lo que llega desde el teléfono. En el mostrador, escribir en el catálogo depende del permiso `EditarCatalogo` de quien llama.
- **Un MCP de la v2 expuesto directo a internet, sin pasar por la computadora del mostrador, no se planea.** Si un día hace falta, lleva su propia decisión.
