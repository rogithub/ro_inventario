# CLAUDE.md — ro_inventario (versión dos)

Sistema de inventario y punto de venta para negocios chicos en México, en Rust. La papelería "El Gordo" es el primer negocio y el laboratorio. Dos aplicaciones (privada y pública) en un mismo workspace.

**Antes de proponer cualquier cosa, leer:**
- [`VISION.md`](VISION.md): qué es y por qué.
- [`docs/decisiones/`](docs/decisiones/README.md): cada decisión de diseño, con lo descartado. Si una propuesta contradice una decisión vigente, se platica antes; no se ignora.
- [`docs/glosario.md`](docs/glosario.md): las palabras del negocio.

---

## Cómo trabajamos

El dueño diseña; la IA implementa. El dueño revisa a velocidad humana, así que la IA trabaja a ese ritmo.

1. **Plan antes de programar.** Un plan corto en español (qué, en qué archivos, por qué) que el dueño aprueba antes de tocar código.
2. **Un cambio a la vez, revisable en ~15 minutos.** Si crece más, se parte y se avisa **antes**. Al terminar un cambio, pausa: no encadenar el siguiente sin su visto bueno.
3. **Al cerrar un cambio:** qué cambió, cómo probarlo a mano y cómo deshacerlo. Distinguir lo que es lógica (revisar con calma) de lo mecánico.
4. **Nada nuevo sin platicarlo:** librería, patrón, herramienta o servicio. Tecnología aburrida a propósito.
5. **Si una decisión tiene una razón de peso en contra o un costo oculto** (datos irrecuperables, trabajo que se tira), se platica antes de ejecutar.
6. **Prueba primero.** Bug → prueba que falla, luego el arreglo. Refactor → prueba que fija el comportamiento actual.
7. **Refactor y funcionalidad en commits separados.** Los archivos grandes no crecen: lo nuevo va en su propio módulo.
8. **El dueño hace los commits y los deploys**, salvo que pida lo contrario. Commits con Conventional Commits (`feat:`, `fix:`, `refactor:`, `chore:`), descripción en español.

---

## Reglas de arquitectura (resumen; el detalle está en las decisiones)

- **Una instancia y una base por negocio.** El código siempre atiende a un solo negocio: no existe `tenant_id`. Los negocios se hablan por API, nunca con consultas entre bases.
- **Configuración en tres capas, cada dato en una sola:** entorno (infraestructura y secretos), `negocio.toml` (lo que se decide con un commit), ajustes en la BD (lo que el usuario cambia en pantalla). Se valida al arrancar. **La configuración elige piezas al arrancar; después nadie pregunta por ella con `if`.**
- **Solo México:** pesos, IVA (16 %, u 8 % fronterizo), CFDI, SPEI, CoDi, Banxico. Nada de configuración por país.
- **Crates:** las áreas del negocio (`crates/ventas`, `crates/inventario`…) son puras: sin `sqlx`, `axum` ni red. Definen los traits que necesitan; `crates/postgres` y `crates/mexico` los implementan; las apps (`apps/privada`, `apps/publica`) solo arman y conectan. Sin reglas del negocio en handlers ni plantillas.
- **Nombres:** vocabulario de programación en inglés, sustantivos del glosario en español (`get_ventas_by_cliente`, `struct Venta`, `is_servicio`). Base de datos en `snake_case`. Términos del SAT cuando existen (`FormaPago`). Sin acentos en identificadores. Comentarios, pantallas y documentación en español.
- **Base de datos:** migraciones numeradas en `migrations/` (una migración en `main` no se edita; se corrige con otra), vistas regenerables en `db/vistas/`, semilla solo de desarrollo en `db/semilla-dev.sql`. La aplicación aplica migraciones y vistas al arrancar.
- **Logs:** `tracing` en JSON en producción; ids del negocio como campos; nunca secretos.
- **Formato y linters:** `rustfmt` de fábrica; `clippy` sin advertencias; sin `unsafe`; sin `unwrap`/`expect` fuera de pruebas (los errores se manejan con `Result`); dinero y cantidades con `Decimal`, nunca `f64`; `cargo audit` en CI.
- **Fines de línea LF** en todo el repo (`.gitattributes`, `.editorconfig`).

---

## Bases de datos y ambientes

| Base | Dónde | Qué puede hacer la IA |
|---|---|---|
| Producción (v2) | Postgres del cluster | **Nada.** Nunca conectarse. Solo la aplicación la cambia, al hacer deploy |
| Producción (v1) `inventario_papeleria` | `192.168.0.10:30432` | **Nada.** Solo el dueño saca copias con `pg_dump` |
| Desarrollo (v2) `dev_ro_inventario` | Postgres de kukulkan (podman, `localhost:5432`) | Leer y escribir; reconstruirla con el script de desarrollo |
| Copia de la v1 `dev_inventario_papeleria` | mismo Postgres de kukulkan | Leer, como origen de `tools/migracion-v1` |
| Pruebas de integración | bases temporales de `sqlx::test` | Las crean y borran las pruebas |

- **Usuario de pruebas de la IA:** lo crea `db/semilla-dev.sql`; solo existe en desarrollo. Sus credenciales están en las variables `E2E_USER` y `E2E_PASS` del perfil del dueño: se leen corriendo el comando dentro de `zsh -ic '…'`. Nunca pedirlas, imprimirlas ni guardarlas.
- Las copias de producción se usan con datos reales (sin anonimizar), por decisión del dueño.

---

## Logs de producción (solo lectura)

- La IA **consulta los logs antes de pedirle un error al dueño.**
- Herramienta: `herramientas/logs.sh` *(pendiente de crear)*. Mientras tanto, la de la versión uno: `/home/ro/code/inventario_papeleria/herramientas/logs-prod.sh`.
- Token de Grafana con rol Viewer en `~/secrets/grafana.env` (nunca imprimirlo).
- Cada error mostrado al usuario trae un id de petición: buscar por ese campo.

---

## Comandos

*Pendientes: se completan cuando exista el código.* Deben quedar aquí: compilar, correr cada aplicación en desarrollo, reconstruir la base de desarrollo, pruebas unitarias y de integración, E2E y consulta de logs.

**Reglas al correrlos:**
- **Verificar el código de salida**, no solo el resumen impreso: un "passed" puede venir con salida en error. Reportar fallas tal cual, con su salida.
- **Servidores locales:** en un puerto libre (p. ej. 5099), en segundo plano, y detenerlos al terminar por su id de tarea. **Nunca matar procesos por nombre o patrón.**
- **Secretos:** nunca en heredoc, `echo` ni `export` en la terminal (quedan en el historial). Se capturan con el editor del dueño (emacs) o con un prompt sin eco. Comandos de una línea cuando el dueño los tenga que correr.

---

## Infraestructura

- Cluster k3s "purépecha" (x86_64), ArgoCD con manifests en `/home/ro/code/k3s-manifests` (tiene su propio `CLAUDE.md`).
- **La IA no corre `kubectl`:** da los comandos al dueño uno por uno (él los corre en curicaueri con el alias `k`) y revisa la salida antes del siguiente.
- Imágenes propias solo amd64, etiqueta `latest`, publicadas por GitHub Actions a `ghcr.io`.
- Los archivos `negocio.toml` reales viven con los manifests de cada instancia, no en este repo.
- Zona horaria de la papelería: `America/Cancun`.

---

## Entorno de uso

| Punto de venta | Dispositivo | Navegador |
|---|---|---|
| Caja principal | Escritorio Linux Debian | Firefox |
| Caja secundaria | iPad mini (744 px de ancho) | Safari |

- Las pruebas E2E corren en los dos anchos. 744 px es donde más se rompe una pantalla.
- Botones de acción frecuente con ícono y `title`; tablas compactas; controles de tamaño táctil.

---

## Proyectos relacionados

| Proyecto | Ruta | Para qué consultarlo |
|---|---|---|
| Versión uno | `/home/ro/code/inventario_papeleria` | Reglas del negocio ya descubiertas (`CLAUDE.md`, `docs/decisiones/`, pruebas). En producción; se congela cuando la v2 arranque en serio |
| Catálogo actual | `/home/ro/code/xplaya` | Base técnica en Rust (Axum, sqlx, plantillas) |
| Infraestructura | `/home/ro/code/k3s-manifests` | Cluster, despliegues, respaldos |
| Intento anterior | etiqueta `intento-2026-05` de este repo | Port abandonado; referencia, no base |
