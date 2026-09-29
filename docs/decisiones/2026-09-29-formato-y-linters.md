# Formato y linters: los de fábrica, más unas reglas que importan en una caja

**Fecha:** 2026-09-29 · **Estado:** vigente

## Contexto
En la versión uno el formato dependía de cada editor (llegó a haber archivos con fines de línea mezclados), no había linter en CI y las dependencias con vulnerabilidades se acumularon (17 avisos de `npm audit` sin atender). Rust trae herramientas estándar para todo esto; lo que falta es decidir cuáles se usan y qué reglas extra valen la pena para un punto de venta.

## Decisión
- **Formato: `rustfmt` con su configuración de fábrica.** Sin archivo `rustfmt.toml`: el formato no se discute. El editor formatea al guardar; CI corre `cargo fmt --check` y rechaza código sin formato.
- **Linter: `clippy` sin advertencias** (`cargo clippy --all-targets -- -D warnings` en CI), con las reglas por omisión más estas, declaradas una sola vez para todo el workspace en `[workspace.lints]` del `Cargo.toml` raíz (cada crate las hereda con `lints.workspace = true`):

  | Regla | Por qué |
  |---|---|
  | `unsafe_code = "forbid"` | La seguridad de memoria es parte de la razón para usar Rust; nada de `unsafe` en código propio. |
  | `clippy::unwrap_used`, `clippy::expect_used` = error (permitidos en pruebas vía `clippy.toml`) | Un `unwrap` que falla tumba la petición. En caja, un error se muestra y se registra; no revienta. |
  | `clippy::float_arithmetic` = error en los crates de áreas del negocio | El dinero y las cantidades son `Decimal`, nunca `f64`. En la versión uno la comisión necesitó redondeos a mano para no perder centavos. |

- **Dependencias: `cargo audit` en CI.** Revisa las dependencias contra la base pública de vulnerabilidades de Rust; un aviso nuevo falla el build hasta que se actualice o se documente por qué se acepta.
- **Pruebas E2E (TypeScript de Playwright):** solo revisión de tipos (`tsc --noEmit`) en CI. Sin linters adicionales.
- **Orden en CI:** formato, clippy, audit, pruebas unitarias e integración, E2E. Si uno falla, no se despliega.

## Descartado
- **`clippy::pedantic` completo:** cientos de avisos de estilo; ruido para una persona. Si una regla de ese grupo resulta útil, se agrega sola a la tabla.
- **Formato personalizado (`rustfmt.toml`):** cada ajuste es una discusión, y el formato estándar es el que conoce cualquier programador de Rust (y la IA).
- **Hooks de pre-commit:** el dueño hace los commits a mano y CI ya vigila lo mismo; un hook sería otra pieza que instalar en cada máquina.
- **`cargo deny`:** revisa además licencias y dependencias duplicadas; más completo, pero más configuración. `cargo audit` cubre lo urgente (vulnerabilidades).

## Consecuencias
- Un `unwrap` en código de producción no compila en CI: los errores se manejan con `Result` y llegan a la pantalla y al log con su id.
- Un aviso de seguridad en una dependencia detiene los deploys hasta atenderlo; no se acumulan.
- Emacs necesita `rustfmt` al guardar (`rustic-mode` o `rust-mode` lo hacen) para no descubrir el formato hasta CI.
