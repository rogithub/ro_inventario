# Los E2E dejan datos en la base de desarrollo, y crecen sin tope

**Salió de:** revisor de la parte 7b2b, 2026-10-02 · **Cuándo:** cuando las listas de desarrollo estorben, o antes de compras (que va a crear más)

## Qué pasa
Los E2E corren contra `dev_ro_inventario` y no borran lo que crean: cada dato lleva un sello («E2E <proyecto> <milisegundos>», `e2e/tests/support.ts`) para no chocar. Cada corrida completa, en los dos navegadores, deja unas 26 categorías, algunas unidades y 14 productos más, y nadie los limpia.

## Por qué importa
Las listas de desarrollo (`/categorias`, `/unidades`, el select de categorías del alta) crecen con cada corrida: estorban al probar a mano y vuelven lentas las capturas. En CI no pasa, porque la base es desechable.

## Propuesta
Para que decida el dueño:
1. Un script de desarrollo que borre lo que empieza con «E2E » (ojo: los artículos no se borran en la aplicación; aquí es solo limpieza de desarrollo).
2. Reconstruir la base de desarrollo de vez en cuando (`herramientas/dev-db.sh` + semilla).
3. Que Playwright use su propia base desechable también en desarrollo, como en CI.
