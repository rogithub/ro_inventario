# Pruebas E2E por historia: la IA las escribe y el dueño las aprueba viendo el video

**Fecha:** 2026-10-08 · **Estado:** vigente

## Contexto
Los E2E (`e2e/tests/`) los escribe la IA junto con cada cambio, y el dueño los revisa por sus nombres y por las capturas. No hay un registro de qué historia cubre cada prueba ni de que el dueño la aprobó, y cuando una falla el reporte de Playwright no dice qué paso del negocio se rompió. La idea viene de IAe2eQaTool (una herramienta de un equipo de QA, en C# con Azure DevOps), adaptada a un solo desarrollador en el kit `e2e_testing` (`/home/ro/code/e2e_testing`, público, GPL v3), que se comparte con la versión uno.

## Decisión
- **Historias de usuario como issues** de GitHub de este repo, con las etiquetas `historia` y `área:<área>`. Las redacta la IA a partir de lo que se platica; la IA crea el issue (con `gh`) solo después de que el dueño aprueba el texto. Es la única escritura en GitHub que hace la IA: el código sigue la regla de ramas (commits locales en `develop`, el dueño hace los `push`).
- **Casos de prueba como `caso.md`** con pasos numerados, en `e2e/historias/<área>/<issue>-<slug>/cp-<n>-<slug>/`, junto a su `prueba.spec.ts` y su `revision.json`. Cada parte de la prueba va dentro de `paso('<n>', …)`, así el reporte dice «FALLA en el paso 3».
- **El kit `@ro/e2e`** (dependencia de desarrollo de `e2e/`, desde un tag de GitHub por `git+https`): `test` (el de Playwright, más la marca de cuándo empieza el video), `paso()`, los dos dispositivos, `evidencia()` (con `E2E_EVIDENCIA=1`: video de cada prueba y 400 ms entre acciones, así que la corrida tarda más; no activa el trace) y un reporter que escribe cada escenario en `e2e/corridas/<id>/` como texto, JSON y video, con el segundo del video en que empieza cada paso. Ese segundo es exacto solo si la prueba importa `test` de `@ro/e2e`. Tacha en lo que escribe los valores de `E2E_PASS`, `E2E_USER` y `DATABASE_URL`.
- **El dueño aprueba cada caso en un tablero** (un artifact de claude.ai) viendo el video: Sí, Cambia esto o Es un bug. La firma se copia a `revision.json`.
- Las pruebas de `e2e/tests/` siguen igual; las que un caso cubre se quitan al aprobarse el caso.
- Las convenciones (páginas sin aserciones, acciones con todas, nunca debilitar una aserción, romper la prueba a propósito antes de aprobarla) están en `e2e_testing/docs/convenciones.md`.

## Descartado
- **Jira o Azure DevOps para historias y casos:** otro sistema que mantener para una sola persona; GitHub ya está.
- **Los casos como issues:** dos fuentes de verdad; como archivos cambian en el mismo commit que la prueba.
- **Todas las pruebas en `e2e_testing`:** las pruebas cambian con las pantallas y deben ir en el mismo commit.
- **Gherkin:** una capa más; la prueba cita los pasos numerados del caso.
- **Rust o C# para las pruebas:** Playwright y su servidor MCP son nativos de Node, y los E2E ya están en TypeScript.

## Consecuencias
- Una dependencia más en `e2e/` que viene de GitHub: `npm ci` necesita red hacia GitHub (CI la tiene). Para actualizar el kit se cambia el tag en `e2e/package.json`.
- `e2e/corridas/` (ignorada por git) crece con cada corrida, con videos si hay evidencia; se borra a mano mientras no estorbe (pendiente en el kit).
- Los traces siguen como estaban (`retain-on-failure`) y guardan lo tecleado, contraseña incluida: no salen de kukulkan.
- `testDir` de Playwright pasa a ser `e2e/` completo, para encontrar `tests/` y `historias/`.
- Para revertir: quitar `@ro/e2e` y el reporter de `e2e/`; las pruebas de casos son Playwright normal y se pueden mover a `tests/` cambiando `paso()` por `test.step()`.
