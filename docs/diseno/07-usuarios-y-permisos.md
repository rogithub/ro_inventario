# Diseño 07 — Usuarios y permisos

**Estado:** propuesta para revisión del dueño.

## Cómo está hoy (v1)
- Dos personas usan el sistema: el dueño y su esposa, los dos con todo el acceso. Lo único que se separa es quién crea usuarios.
- Hay cuatro roles definidos (Admin, Gerente, Vendedor y "Anonymous"), pero **solo se revisa "Admin"**, en 8 lugares (administrar usuarios, recalcular costos). Todo lo demás lo puede hacer cualquiera que entre.
- "Socio" es una marca aparte en el usuario, que usa finanzas (aportaciones y retiros).
- Las contraseñas usan un esquema propio (HMAC-SHA512 con sal).
- Los roles nacieron pensando en otros negocios, pero nunca se usaron y quedaron mal manejados (lección en `VISION.md`).

## Lo que se propone
- **Permisos por acción, definidos en el código.** Cada acción que importa tiene su permiso; el código revisa el permiso, nunca el nombre de un rol.
- **Roles = conjuntos de permisos, como datos de cada negocio.** Cada negocio puede tener los suyos; se entregan tres roles de arranque.
- **Un usuario tiene un rol.** Suficiente para negocios chicos y medianos; varios roles por usuario, si algún caso real lo pide.
- **La papelería tiene sus propios roles** (son datos del negocio): **Dueño** (todos los permisos) y **Socia** (todos menos `administrar_usuarios`), para la esposa del dueño. El diseño completo existe para negocios con empleados; las pantallas para editar roles pueden esperar a que alguien las necesite.

## Permisos
| Permiso | Qué deja hacer |
|---|---|
| `vender` | Cobrar, guardar pedidos y cotizaciones. |
| `cancelar_venta` | Cancelar una venta cobrada. |
| `devolver` | Registrar devoluciones y reembolsos. |
| `ver_costos` | Ver costos, márgenes y ganancia (en pantallas y reportes). |
| `editar_catalogo` | Dar de alta y editar artículos, precios, recetas, kits, categorías; descontinuar; fusionar duplicados. |
| `comprar` | Capturar compras, recepciones y pagos a proveedores. |
| `ajustar_inventario` | Mermas e ingresos sin compra, conteos físicos y ubicaciones de los productos (diseño 08). |
| `operar_caja` | Hacer cortes y registrar movimientos de caja ("Saqué dinero"). |
| `gestionar_clientes` | Dar de alta y editar clientes. |
| `vetar_clientes` | Poner y levantar vetos. |
| `ver_reportes` | Ver reportes de ventas e inventario (sin costos, salvo que tenga `ver_costos`). |
| `configurar_negocio` | Cambiar ajustes (porcentaje y vigencia del monedero, spread del dólar, tasa de comisión, vigencia de cotizaciones, plazo de un veto). |
| `administrar_usuarios` | Crear usuarios, asignar roles, restablecer contraseñas. |

## Roles de arranque
| Rol | Permisos |
|---|---|
| **Dueño** | Todos. |
| **Encargado** | Todos menos `configurar_negocio` y `administrar_usuarios`. |
| **Cajero** | `vender`, `gestionar_clientes`, `operar_caja` (anotar lo que sale; el corte puede quedar para el encargado si el negocio quiere quitarle este permiso). |

## Tablas propuestas
- `usuarios`: `id`, `email` (único; con él se entra), `nombre`, `password_hash`, `rol_id`, `desactivado_at` (un usuario que se va se desactiva, no se borra: sus ventas conservan quién las hizo), `created_at`.
- `roles`: `id`, `nombre`.
- `roles_permisos`: `rol_id`, `permiso` (uno de la lista de arriba; la lista vive en el código como `enum`).
- Sesiones en Postgres (con cookie), como en el intento anterior.

## Usuarios que no son personas (y no deben existir)
En la v1 se crearon usuarios que no son una persona, igual que el "cliente anónimo" (diseño 06):
- **Un usuario para los pedidos que llegan de xplaya.com** (`ID_XPLAYA.COM_ANONYMOUS_USER` en los ajustes de la v1). En la v2 la venta anota su punto de venta, y **"sitio público" es un punto de venta más**; quien la creó queda vacío, porque no fue una persona. No hay un usuario con el que alguien pudiera entrar.
- **Usuarios para demostraciones a posibles compradores.** En la v2 una demo es **otra instancia** en el cluster (decisión "una instancia por negocio"): su propia base con datos de ejemplo, su `negocio.toml` ("Papelería de demostración") y su dominio (p. ej. `demo.xplaya.com`). Se restablece cuando se quiera, sin tocar la real, y quien la prueba no ve ventas reales.
- **El usuario de la IA** solo existe en desarrollo (semilla), como ya se decidió.

## Reglas
- **La aplicación revisa el permiso al atender cada acción** (en la capa web, antes de llamar al negocio); la pantalla además oculta lo que el usuario no puede hacer. Ocultar no basta: el servidor siempre revisa.
- **Contraseñas con Argon2** (el estándar actual). Los usuarios de la v1 **no se migran con su contraseña**: son dos personas, y al corte se ponen contraseñas nuevas.
- **Contraseña olvidada:** un Dueño (quien tenga `administrar_usuarios`) la restablece. Sin recuperación por correo ni WhatsApp hasta que haga falta.
- **El primer usuario de un negocio** se crea con un comando que pide la contraseña sin mostrarla (decisión de base de datos). No hay usuario administrador por omisión.
- **El usuario de pruebas de la IA** solo existe en desarrollo (semilla).
- **Auditoría:** cada documento guarda quién lo creó o cambió (`created_by`, `updated_by`), como ya se definió en cada diseño.

## Fuera de este diseño
- **Socios** (aportaciones y retiros de dinero): con finanzas. No es un permiso, es un dato del negocio.
- **Cambio rápido de usuario en un punto de venta compartido** (PIN en el iPad o la Elo, para negocios con varios cajeros): cuando haya un caso real.
- **Clientes y proveedores como usuarios** (portales): a futuro.
- **Pantallas para crear o editar roles:** cuando un negocio las necesite.

## Respuestas del dueño (2026-09-29)
1. **Roles:** su esposa puede hacer todo menos dar de alta usuarios (es la única otra usuaria y no le preocupa). Él sí da de alta usuarios: a veces a la IA; una vez uno para que se guarden los pedidos que vienen de xplaya.com; y otros para hacer demos a posibles compradores (quizá eso debería ser otro ambiente, accesible desde k3s).
2. **Contraseña olvidada:** por ahora basta con que un Dueño la restablezca. Evitar mecanismos complejos antes de que sean necesarios.
3. **Contraseñas nuevas** en el corte: de acuerdo.
