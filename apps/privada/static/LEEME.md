# Archivos de terceros

Se sirven desde la aplicación (van dentro del binario), no de un CDN: la caja no depende de un servicio
externo y las versiones quedan fijas aquí. Decisión: `docs/decisiones/2026-09-30-pantallas-del-servidor.md`.

| Archivo | Paquete npm | Versión |
|---|---|---|
| `bootstrap.min.css`, `bootstrap.bundle.min.js` | `bootstrap` | 5.3.8 |
| `bootstrap-icons.min.css`, `fonts/bootstrap-icons.woff2` | `bootstrap-icons` | 1.13.1 |
| `htmx.min.js` | `htmx.org` | 2.0.11 |

**Para actualizar:** `npm pack <paquete>@<versión>` (verifica la integridad contra el registro), copiar
los mismos archivos de su `dist/` (o `font/`), quitar la línea `sourceMappingURL` (los mapas no se
incluyen) y actualizar esta tabla. Solo `woff2`: todos los navegadores de las cajas lo leen.
