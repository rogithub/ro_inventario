# Dos ramas: la IA trabaja en `develop`, el dueño prueba en su laptop y lleva a `main`

**Fecha:** 2026-10-06 · **Estado:** vigente

## Contexto
Hasta el paso 7b, la IA hacía los commits directo en `main` (locales, en kukulkan) y el dueño hacía el `push`, que dispara CI y publica la imagen. No había un lugar para probar a mano antes de que un cambio llegara a `main`. Al intentar probar la aplicación de kukulkan desde la laptop, la cookie de sesión `Secure` no viajaba por http (se agregó `COOKIE_SECURE`). El dueño es el único desarrollador y ya trabajaba así en la v1: una rama `develop` que baja a su laptop y prueba con su base de desarrollo, y `main` para lo aprobado.

## Decisión
- **Dos ramas, nada más:** `develop` y `main`.
- **La IA hace sus commits en `develop`**, con las mismas reglas de siempre (un paso a la vez, `revisar.sh` y el revisor antes de reportar). No trabaja en `main`.
- **El dueño hace los `push`:** sube `develop`, la baja en su laptop y la prueba con su instancia y su base de desarrollo (`dev_ro_inventario`, el mismo nombre que la de kukulkan, a propósito, para no confundirse). Con su visto bueno hace el merge a `main` y el `push`, que dispara CI y la imagen.
- **CI corre solo en `main`.** El dueño corre las pruebas a mano en su laptop cuando quiere.
- **Cuando el dueño quiere que la IA vea lo mismo que él,** la IA arranca la aplicación en kukulkan con `COOKIE_SECURE=false` y el dueño la abre en `http://192.168.0.2:5100`.

## Descartado
- **Probar en kukulkan, sin bajar `develop` a la laptop:** no pide nada en la laptop, pero el dueño ya tiene su entorno configurado allá y prefiere probar ahí.
- **Una base aparte para el dueño en kukulkan (`dev_humano`):** su laptop ya tiene la suya.
- **CI también en `develop`:** agrega complejidad; las pruebas se corren a mano.
- **Una instancia de pruebas (staging) en el cluster:** se platica cuando haga falta probar con https y en condiciones de producción, probablemente cerca de la caja o del primer deploy.

## Consecuencias
- `main` solo recibe lo que el dueño probó. Si algo falla en `develop`, no llega a la imagen.
- Los E2E de kukulkan siguen dejando datos en la base de kukulkan, no en la del dueño (ver `docs/pendientes/datos-e2e-en-desarrollo.md`).
- Para revertir: volver a trabajar en `main` y actualizar la regla 8 de CLAUDE.md.
