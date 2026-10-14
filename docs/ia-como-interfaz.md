# La IA como interfaz — visión

> Visión, no un plan. Salió de la conversación del dueño del 2026-10-14, después de ver funcionar a Claude Code en la computadora del mostrador. Nada de esto se construye sin platicarlo; las piezas técnicas nuevas llevan su propia decisión. El alcance y el orden del mínimo viable están en [la decisión](decisiones/2026-10-14-minimo-viable.md).

## De dónde sale

En octubre de 2026, la dueña de la papelería empezó a trabajar con Claude Code en la computadora del mostrador, con dos MCP armados sin planear:
- **`pedidos`**, encima de la v1 (`/home/ro/code/inventario_papeleria/Ro.Inventario.Agente`): buscar productos y clientes, accesos rápidos, crear pedidos y consultarlos.
- **`imagina`**, local (`/home/ro/code/pdfeitor/mcp`): arma PDFs con fotos, atiende el kiosko e imprime con CUPS.

Con ellos y con las herramientas de la computadora, el asistente atiende WhatsApp, cotiza, crea pedidos, imprime, acomoda fotos, copia INE, arregla PDFs y escanea. Ella le habla desde la terminal o desde el iPad con `/remote-control`. El detalle, con lo que falla, está en el documento que el asistente escribió sobre su propio trabajo: [Lo que hace el asistente en la papelería](https://claude.ai/code/artifact/b5a9e507-8c35-47f0-9e66-374575a4100c).

Ha sido un éxito. Llenar formularios ya no es la única forma de usar un punto de venta: la persona pide y el sistema hace.

## Lo que enseñó

Casi todo lo que falla tiene una causa común: **la regla vive en el texto de una skill, no en el servidor.**
- «El pedido lleva cliente» es una instrucción. El agente la olvidó y el pedido #1906 salió sin cliente.
- El precio de la impresión a color depende de la cobertura y se decide a ojo, con reglas escritas en texto. Las cotizaciones salen disparejas.
- El estado del pedido (listo, avisado, cobrado) se lleva en un `pendientes.md` aparte del POS. Hay dos fuentes de verdad.
- Los errores dicen «An error occurred» y el agente no sabe corregirse.

Es la misma tesis de la v2: **las reglas y el dinero, en el servidor.** La IA pide; el servidor decide, calcula y explica por qué rechaza.

## La forma

```
teléfono (voz) → Claude (nube) → MCP externo         ← ligero, quizá temporal
                                   autenticación + traducir herramientas a llamadas;
                                   sin base de datos, sin reglas
                                        │
                                        ▼  contrato interno (API de la privada)
                     ┌──────────── núcleo interno (cluster) ────────────┐
                     │ reglas, transacciones, personas, pedidos,         │
                     │ cola de órdenes, estado en tiempo real            │
                     └──────┬──────────────────────────┬────────────────┘
                            ▼                          ▼
              pantallas (caja, iPad):        computadora del mostrador: un agente
              ven en vivo «se creó el        toma órdenes de la cola, y sus MCP
              pedido», «la impresora         locales (imagina y los que vengan)
              arrancó»                       sirven a Claude Code ahí mismo
```

- **El MCP externo existe para poder hablar.** Literalmente: hablarle al asistente desde el teléfono y oír su respuesta, en tiempo real. Para eso Claude tiene que llegar al sistema desde la nube, y lo expuesto tiene que ser lo mínimo: autenticación (OAuth) y la traducción de cada herramienta a una llamada al contrato interno. Sin credenciales de la base y sin reglas. Si un día ya no hace falta dar esa vuelta, se borra esa app y su salida a internet, y no cambia nada más.
- **Lo estable es el contrato interno.** Las pantallas, el MCP externo y lo que venga después (una voz local, otro negocio de la [Red Comercial](red-comercial.md)) son clientes del mismo contrato y cumplen las mismas reglas.
- **Quién lo pidió viaja hasta adentro.** Lo hecho por el asistente queda a nombre de la persona autenticada, con los permisos de su rol, y el registro dice que fue por el asistente.
- **La pantalla acompaña a la voz.** El núcleo publica lo que pasa (pedido creado, orden tomada, impresión terminada) y la pantalla lo muestra al momento.
- **La computadora del mostrador no abre puertos.** El hardware (impresoras, escáner, archivos descargados) se maneja con una **cola de órdenes**. El núcleo deja la orden («imprime este archivo en R1, 2 copias, color») y el agente de la computadora la recoge cuando sale a preguntar. Si está apagada, la orden espera. La lista de órdenes posibles es cerrada: el asistente no puede pedir nada fuera de ella. Por dentro, la computadora se enriquece con sus propios MCP locales, como procesos de la máquina, sin web.
- **Los documentos personales no salen de la tienda.** El asistente puede decir «imprime el INE que llegó hoy» viendo solo el nombre del archivo; el contenido no pasa por la nube salvo que ella lo pida.

## Reglas que se conservan

Vienen del documento del asistente y valen para todo lo que se construya:
- **El catálogo y los precios no se cambian por el asistente remoto.** Los maneja la dueña.
- **Nada sale al cliente sin su «sí»:** mensajes, precios, totales y avisos de «ya está listo».
- **Imprimir siempre con confirmación:** archivo, impresora, copias, color, tamaño y caras.
- **Sin nombres de clientes en registros y telemetría:** se correlaciona por número de pedido.
- **Documentos personales** (INE, CURP, actas) solo en la computadora o en la red local.
- **Cobrar se queda en pantalla**, de escritorio y táctil ([decisión](decisiones/2026-10-14-minimo-viable.md)).

## Lo que pide el MCP de la v2

La lista del asistente sirve como requisitos:
- errores que digan qué parámetro falta;
- el enlace de la cotización en cada respuesta;
- cliente obligatorio por teléfono, salvo venta de mostrador explícita;
- editar el pedido y su estado;
- alta de cliente;
- cotizar sin crear pedido, con el total calculado en el servidor;
- precio de impresión por parámetros (impresora, tamaño, color, cobertura, caras), con la regla de cobertura en el servidor;
- el trabajo hecho ligado al pedido;
- búsqueda con paginación y existencia;
- permisos separados de lectura y escritura.

## Dudas abiertas

- **¿El modo de voz del teléfono usa los conectores MCP?** Está confirmado que la app del teléfono usa los conectores remotos agregados desde claude.ai, y que Claude se conecta a ellos desde la nube de Anthropic. No hay nada oficial sobre el modo de voz. **Prueba de 5 minutos, antes de diseñar:** agregar en claude.ai un conector público cualquiera, abrir el modo de voz en el teléfono y pedirle algo que lo use.
- **Cómo se empujan los eventos a la pantalla**, dónde se guardan los documentos y qué librería de MCP y de OAuth se usan: cada una, con su decisión, cuando llegue.

## Para después

- **Leer el hardware en lugar de alimentarlo con datos.** Las impresoras de red reportan nivel de tóner y contadores de páginas por SNMP, un estándar que traen casi todas. Cruzar «hojas impresas según la impresora» contra «hojas cobradas» vale más que el tóner.
- Gráficas y el módulo de impresiones.
