# Plan — Formulario de nueva venta

Página más compleja del POS. Este documento es la referencia de implementación;
marcar cada item con `[x]` al completarlo.

**Archivos clave de referencia en el .NET:**
- `inventario_papeleria/Ro.Inventario.Core/Repos/BusquedaProductosRepo.cs` — lógica de búsqueda de producto
- `inventario_papeleria/Ro.Inventario.Core/Repos/ContactosRepo.cs` — búsqueda de clientes
- `inventario_papeleria/Ro.Inventario.Web/Controllers/AppController.cs` — normalización de teléfono
- `inventario_papeleria/Ro.Inventario.Web/scripts/pages/ventas.ts` — carrito KnockoutJS (referencia de lógica, no de código)

---

## Paso 1 — Endpoints JSON de apoyo ✅

Archivo: `src/modules/ventas/routes.rs` (handlers) + `src/modules/ventas/queries.rs` (SQL)

### 1a. `GET /api/productos/buscar?q=&stock=`

Lógica de decisión (en este orden):

1. Si `q` es entero → buscar por `nid` en `v_inventario`
2. Si `q` es UUID válido → buscar por `id` en `v_inventario` (QR code propio)
3. Si no → buscar por `CodigoBarrasItem` o `CodigoBarrasCaja` exacto
4. Si no → FTS: `search_vector @@ websearch_to_tsquery('spanish', q)` + `unaccent(Nombre) ILIKE unaccent('%q%')`
5. Si no → trigramas: `Nombre % q` (requiere extensión `pg_trgm`)

Parámetro `stock=true` agrega `AND Stock > 0` en todas las variantes.

Respuesta JSON (array):
```json
[{
  "id": "uuid",
  "nid": 123,
  "nombre": "Papel bond carta",
  "unidad_medida": "pza",
  "precio_venta": "12.50",
  "precio_compra_promedio": "8.00",
  "stock": "144.00",
  "es_servicio": false,
  "codigo_barras_item": "...",
  "codigo_barras_caja": "..."
}]
```

`precio_compra_promedio` siempre se incluye (POS interno, usuario autenticado).

Struct Rust: `ProductoBuscado` — derivar `serde::Serialize`.
SQL: sobre `v_inventario`. Campos: `id, nid, nombre, unidadmedida, ultimoprecioventa,
preciocomprapromedio, stock, esservicio, codigobarrasitem, codigobarrascode`.

### 1b. `GET /api/clientes/buscar?q=`

Normalización del patrón antes de la query:
- Si todos los caracteres no-dígitos se eliminan y quedan ≥ 6 dígitos → buscar solo por teléfono
- Lógica de normalización en Rust: `q.chars().filter(|c| c.is_ascii_digit()).collect::<String>()`, luego tomar los últimos 10.
- Si el resultado normalizado tiene ≥ 6 dígitos: `WHERE REGEXP_REPLACE(telefono, '\D', '', 'g') ILIKE '%<normalized>%'`
- Si no: `WHERE unaccent(nombre) ILIKE unaccent('%q%') OR unaccent(telefono) ILIKE '%q%'`

Tabla: `Contactos WHERE tipo = 0` (clientes).

Respuesta JSON:
```json
[{
  "id": "uuid",
  "nombre": "Juan Pérez",
  "telefono": "5512345678",
  "email": "juan@example.com"
}]
```

Struct: `ClienteBuscado` — `serde::Serialize`.

### 1c. `GET /api/tipo-cambio`

1. Llamar a Banxico API con token de `Settings.banxico_api_token` (ya está en `Settings`).
   - URL: `https://www.banxico.org.mx/SieAPIRest/service/v1/series/SF43718/datos/oportuno`
   - Header: `Bmx-Token: <token>`
   - Extraer el valor del campo `dato[0].dato`
2. Si falla (timeout, error) → devolver `Settings.tipo_cambio_dolares` como fallback.
3. Guardar el valor exitoso en `Settings` (actualizar `tipo_cambio_dolares` en memoria y en la tabla).

Respuesta JSON: `{ "tipo_cambio": "17.25", "fuente": "banxico" | "cache" }`

### 1d. `GET /api/monedero/:cliente_id`

Query sobre `MonederoGenerados` y `MonederoRedimidos`:

```sql
SELECT
  COALESCE(SUM(mg.dinerodigital), 0)
    FILTER (WHERE mg.devolucionid IS NULL AND mg.fechaexpiracion > NOW()) AS generado_vigente,
  COALESCE(SUM(mr.dinerodigital), 0) AS redimido
FROM contactos c
LEFT JOIN monederogenerados mg ON mg.contactoid = c.id
LEFT JOIN monederoredimidos mr ON mr.contactoid = c.id
WHERE c.id = $1
```

Balance = `generado_vigente - redimido` (mínimo 0).

Respuesta JSON: `{ "balance": "45.00" }`

---

## Paso 2 — POST /ventas (transacción)

Handler en `src/modules/ventas/routes.rs`, lógica en `src/modules/ventas/queries.rs`.

### Body JSON recibido

```json
{
  "fecha": "2026-05-20T14:30:00",
  "fecha_editada": false,
  "cliente_id": "uuid | null",
  "pago": "200.00",
  "pago_monedero": "0.00",
  "pago_tarjeta": "0.00",
  "pago_transferencia": "0.00",
  "pago_dolares": "0.00",
  "tipo_cambio_dolares": "0.00",
  "cambio": "50.00",
  "lineas": [
    { "producto_id": "uuid", "cantidad": "2.00", "precio_unitario": "75.00" }
  ]
}
```

Struct Rust: `NuevaVentaPayload` con `serde::Deserialize`.

### Validaciones (server-side, aunque Alpine también las haga)

- `lineas` no vacío
- Cada `cantidad > 0` y `precio_unitario >= 0`
- Suma de pagos = total de líneas (tolerancia de 1 centavo para float/rounding)
- `cambio >= 0`
- Si `pago_monedero > 0` → `cliente_id` no puede ser null
- Solo efectivo y dólares pueden dar cambio; si `pago_tarjeta > 0 || pago_transferencia > 0` → `cambio` debe ser 0

### Transacción

```
BEGIN
  INSERT INTO Ajustes → obtener id
  INSERT INTO AjustesProductos (una fila por línea)
  INSERT INTO MonederoGenerados (una fila por línea de producto real)
    → solo si TipoAjuste=0 y cliente_id IS NOT NULL
    → monto = cantidad * precio_unitario * settings.tipo_cambio_monedero
    → fecha_expiracion = NOW() + settings.dias_vigencia_monedero días
COMMIT
```

Respuesta: `201 Created` con `{ "id": "uuid" }`.
En caso de error de validación: `422 Unprocessable Entity` con mensaje.

---

## Paso 3 — Esqueleto del template `GET /ventas/nueva`

Archivo: `templates/ventas/nueva.html`

Layout dos columnas en `md+`:
- **Izquierda (col-md-8):** búsqueda de producto + tabla del carrito
- **Derecha (col-md-4):** cliente, métodos de pago, total, botón de guardar

`x-data="carrito()"` envuelve toda la página. La función `carrito()` vive en un
`<script>` al final del template (no en un archivo separado — sin build step).

El template debe compilar y renderizarse vacío antes de agregar lógica.

---

## Paso 4 — Control de búsqueda de producto (reutilizable)

Implementado como componente Alpine dentro del `x-data` del carrito.

### Comportamiento

- Input de texto con placeholder "NID, nombre, código de barras o QR"
- `@keyup.enter` → llama `/api/productos/buscar?q=<valor>&stock=<config>`
- Lista desplegable con resultados (máx. 10); clic o Enter selecciona
- Al seleccionar: agrega línea al carrito, limpia el input, devuelve el foco al input
- `@click.outside` cierra la lista sin seleccionar

### Detección de lector de barcode/QR

El lector inyecta caracteres muy rápido (< 50ms entre teclas) y termina con Enter.
Mecanismo:
```js
lastKeyTime: 0,
onKeydown(e) {
  const now = Date.now();
  const delta = now - this.lastKeyTime;
  this.lastKeyTime = now;
  if (delta < 50) this.isScannerInput = true;
  if (e.key === 'Enter' && this.isScannerInput) {
    this.buscarProducto();
    this.isScannerInput = false;
  }
}
```
Foco automático al cargar la página (`x-init="$el.focus()"`).

### Configuración

Prop `soloConStock: true/false` — inyectada desde el template Rust via `x-data`.
Para la página de nueva venta: `soloConStock: true`.

---

## Paso 5 — Carrito con cálculos en tiempo real (Alpine.js)

### Estado

```js
carrito() {
  return {
    lineas: [],          // { id, productoId, nombre, cantidad, precioUnitario, esServicio }
    clienteId: null,
    clienteNombre: '',
    balanceMonedero: '0.00',
    pagoEfectivo: '',
    pagoMonedero: '',
    pagoTarjeta: '',
    pagoTransferencia: '',
    pagoUsd: '',
    tipoCambioUsd: '0.00',
    cambio: '0.00',
    // ...
  }
}
```

### Getters (computed via `get` de JS)

```js
get subtotal() {
  // suma de lineas excluyendo la línea de comisión TC
  return this.lineas
    .filter(l => l.productoId !== COMISION_TC_ID)
    .reduce((s, l) => s + l.cantidad * l.precioUnitario, 0);
},

get comisionTc() {
  // cálculo iterativo: dos pasadas sobre pagoTarjeta
  const t = parseFloat(this.pagoTarjeta) || 0;
  if (t === 0) return 0;
  const tasa = TASA_TC;       // de Settings, inyectado en el template
  const iva  = IVA;           // de Settings
  const c1 = t * tasa * (1 + iva);
  const c2 = (t + c1) * tasa * (1 + iva);
  return c2;
},

get total() {
  return this.subtotal + this.comisionTc;
},

get pagoUsdEnPesos() {
  return (parseFloat(this.pagoUsd) || 0) * (parseFloat(this.tipoCambioUsd) || 0);
},

get cambioCalculado() {
  const pagado = (parseFloat(this.pagoEfectivo) || 0) + this.pagoUsdEnPesos;
  const restante = this.total
    - (parseFloat(this.pagoMonedero) || 0)
    - (parseFloat(this.pagoTarjeta) || 0)
    - (parseFloat(this.pagoTransferencia) || 0)
    - this.pagoUsdEnPesos;
  return Math.max(0, pagado - restante);
}
```

### Mutaciones

- `agregarLinea(producto)` — agrega o incrementa cantidad si ya existe
- `quitarLinea(index)` — elimina línea; si era la última, resetea comisión TC
- `actualizarComisionTc()` — elimina línea de comisión existente y la re-agrega si `pagoTarjeta > 0`
- `seleccionarCliente(cliente)` — setea `clienteId`, carga balance de monedero via fetch
- `limpiarCliente()` — resetea cliente y monedero

### Regla: línea de comisión TC

Se agrega/actualiza automáticamente cuando cambia `pagoTarjeta`.
`productoId = COMISION_TC_ID` (UUID de Settings, inyectado en el template).
`precioUnitario = comisionTc` (recalculado).
`cantidad = 1`.

---

## Paso 6 — Control de búsqueda de cliente

Similar al de producto pero más simple (sin detección de scanner).

- Input de texto con placeholder "Nombre o teléfono"
- `@input.debounce.400ms` → fetch a `/api/clientes/buscar?q=<valor>`
- Lista desplegable; al seleccionar: `seleccionarCliente(c)`, fetch a `/api/monedero/:id`
- Botón "×" para limpiar cliente (venta anónima)
- Muestra: nombre + teléfono + balance de monedero disponible

Normalización de teléfono: en el lado del servidor (Rust), no en el cliente.

---

## Paso 7 — Submit

```js
async guardar() {
  this.guardando = true;
  const payload = { /* armar desde el estado */ };
  const res = await fetch('/ventas', {
    method: 'POST',
    headers: { 'Content-Type': 'application/json' },
    body: JSON.stringify(payload)
  });
  if (res.ok) {
    const { id } = await res.json();
    window.location.href = '/ventas';
  } else {
    this.error = await res.text();
    this.guardando = false;
  }
}
```

Botón de guardar: deshabilitado si `total === 0` o `guardando === true`.

---

## Inyección de Settings en el template

Los valores de Settings que Alpine necesita se pasan desde Rust via atributos
`data-*` en el `div` raíz del `x-data`, y se leen en la función `carrito()`:

```html
<div x-data="carrito()"
     data-comision-tc-id="{{ settings.comision_tc_servicio_id }}"
     data-tasa-tc="{{ settings.tasa_comision_tc }}"
     data-iva="{{ settings.iva }}"
     data-tipo-cambio-monedero="{{ settings.tipo_cambio_monedero }}">
```

```js
carrito() {
  const el = document.currentScript.closest('[x-data]');
  const COMISION_TC_ID = el.dataset.comisionTcId;
  const TASA_TC = parseFloat(el.dataset.tasaTc);
  const IVA = parseFloat(el.dataset.iva);
  // ...
}
```

---

## Notas de dominio importantes

- **Comisión TC:** se calcula en **dos pasadas** sobre el monto en tarjeta.
  Primera: `c1 = pago_tarjeta * tasa * (1 + iva)`.
  Segunda: `c2 = (pago_tarjeta + c1) * tasa * (1 + iva)`.
  Se usa `c2`. Se agrega como **línea de producto** con el UUID de `SERVICIO_COMISION_TARJETRA_CREDITO_ID`.

- **MonederoGenerado:** solo si `TipoAjuste=0` y `ClienteId IS NOT NULL`.
  Una fila por línea de AjustesProductos.
  Excluir la línea de comisión TC (es un servicio, no genera monedero).
  `monto = cantidad * precio_unitario * tipo_cambio_monedero`.
  `fecha_expiracion = NOW() + dias_vigencia_monedero`.

- **Trampa del JOIN:** no hacer JOIN entre `Ajustes` y `AjustesProductos` para calcular totales de pago. Los pagos solo se leen de `Ajustes`.

- **`ID_CLIENTE_GENERICO`** en Settings: UUID del cliente anónimo para ventas sin cliente. Se puede usar si el negocio requiere que toda venta tenga `ClienteId`, pero en este sistema las ventas anónimas dejan `ClienteId = NULL`.

- **Tipos de cambio USD:** el valor de Banxico se guarda en `settings` tabla (columna `value` de la fila `TIPO_CAMBIO_DOLARES`) y en memoria en `AppState::settings.tipo_cambio_dolares` para el fallback.
