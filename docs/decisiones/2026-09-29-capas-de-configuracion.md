# La configuración vive en tres capas, y cada dato en una sola

**Fecha:** 2026-09-29 · **Estado:** vigente

## Contexto
Cada negocio tiene su propia instancia ([decisión](2026-09-29-una-instancia-por-negocio.md)), así que la configuración es lo que le dice a una instancia qué negocio es: nombre, módulos, formas de pago, tasa de IVA. En la versión uno casi todo vivía en una tabla `Settings` de texto libre (clave → valor): sin tipos, sin revisión, un valor mal escrito se descubría al fallar en caja, y no se sabía quién cambió qué.

## Decisión
Tres capas; **cada dato vive en exactamente una**, nunca en dos con reglas de "cuál gana".

| Capa | Qué va ahí | Quién la cambia | Cuándo aplica |
|---|---|---|---|
| **1. Variables de entorno** | Infraestructura y secretos: conexión a la BD, puerto, llaves, credenciales (PAC, pagos) | El deployment (SealedSecrets) | Al arrancar |
| **2. Archivo del negocio** (`negocio.toml`) | Lo que se decide una vez y rara vez cambia: identidad (nombre, dominio, zona horaria), tasa de IVA, módulos prendidos, formas de pago aceptadas, forma del reembolso (efectivo o la misma del pago), qué implementación usar para cada pieza (tipo de cambio, facturación), nodos amigos | Un commit en el repo de manifests (revisado, con historial) | Al arrancar |
| **3. Ajustes en la BD** | Lo que el dueño cambia desde la aplicación: porcentaje y vigencia del monedero, spread del dólar, comisión de su terminal, días de vigencia de una cotización, plazo de un veto, textos del ticket | El usuario, en pantalla (queda registro de quién y cuándo) | En caliente |

**La regla para decidir la capa:** secreto o infraestructura → entorno; se cambia con un commit → archivo; lo cambia el usuario en pantalla → BD.

**Cómo se usa:**
- El archivo se lee en un `struct` tipado y se valida al arrancar: una clave desconocida, un valor faltante o una combinación inválida (p. ej. facturación prendida sin credenciales del PAC) **detiene el arranque** con un mensaje claro. No hay valores de negocio por omisión escondidos: la tasa de IVA y la zona horaria se escriben siempre.
- **La configuración elige piezas, no ramas.** Al arrancar se arma la aplicación: módulos apagados no registran sus rutas ni aparecen en el menú; cada pieza intercambiable (trait) recibe la implementación que dice el archivo. Después, ningún código vuelve a preguntar por la configuración para decidir qué hacer.
- **Solo México** (ver la visión): no hay configuración de país ni de moneda. Lo que varía entre negocios mexicanos sí es configuración, como la tasa de IVA (16 %, u 8 % en la región fronteriza) y la zona horaria.
- Los nombres de las claves siguen la regla de nombres del proyecto.

Ejemplo ilustrativo (el formato exacto se define al implementarlo):

```toml
[negocio]
nombre = "Papelería El Gordo"
time_zone = "America/Cancun"

[impuestos]
iva = 0.16             # 0.08 en la región fronteriza

[modules]
monedero = true
impresoras = true
facturacion = false

[pagos]
formas = ["efectivo", "tarjeta", "transferencia", "dolares"]
```

## Descartado
- **Todo en la tabla de ajustes de la BD** (como la versión uno): sin tipos ni validación al arrancar, sin revisión ni historial en git, y mezcla lo que decide el operador con lo que decide el usuario.
- **Todo en variables de entorno:** planas, sin estructura para listas ni secciones, fáciles de escribir mal, y revueltas con los secretos.
- **Capas que se sobrescriben entre sí** (archivo con valores por omisión, entorno que los reemplaza, BD que reemplaza al entorno): para saber el valor real hay que revisar tres lugares.
- **Un servicio de feature flags:** otra pieza que operar, para dos o tres negocios.
- **Configuración por país** (moneda, impuestos, facturación y pagos intercambiables por país): complejidad sin un caso real. Si llega un negocio de otro país, se agrega como extensión.
- **YAML:** TOML es lo común en Rust, más simple y sin sorpresas de sangría o tipos implícitos.

## Consecuencias
- Cambiar algo de la capa 2 es un commit y un reinicio de la instancia (ArgoCD lo aplica); lo urgente del día a día va en la capa 3.
- El repo incluye un `negocio.ejemplo.toml` que **una prueba carga y valida**, para que nunca quede desactualizado. CI puede validar también los archivos reales de cada negocio.
- Los archivos reales viven con los manifests de cada instancia (`/home/ro/code/k3s-manifests`), no en este repo: el código no sabe qué negocios existen.
- La pantalla de ajustes (capa 3) solo muestra lo que el usuario puede cambiar; nunca secretos ni decisiones de operación.
