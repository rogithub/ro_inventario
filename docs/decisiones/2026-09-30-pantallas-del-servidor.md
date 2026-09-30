# Las pantallas las arma el servidor: askama + htmx, y el dinero se calcula solo ahí

**Fecha:** 2026-09-30 · **Estado:** vigente

## Contexto
Antes de la primera pantalla hay que decidir cómo se hacen. En la versión uno las pantallas son TypeScript con Knockout: el navegador calcula totales, comisión de tarjeta y cambio con punto flotante, y el servidor guarda con decimales; a veces difieren uno o dos centavos (diseño 04). La preocupación con que todo lo calcule el servidor es la latencia, sobre todo para negocios lejanos del cluster. Revisando la v1: **cada escaneo ya va al servidor** (`busquedas/GetByQr`), igual que la búsqueda por nombre; lo único instantáneo es editar el carrito y teclear los pagos.

## Decisión
- **El servidor arma el HTML con plantillas askama**, revisadas al compilar: una variable mal escrita en la pantalla de cobro no compila.
- **htmx** reemplaza solo el pedazo que cambia: cada escaneo o cambio del carrito va al servidor y regresa el carrito ya calculado. El escaneo entra en cola (`hx-sync`) para que el lector nunca se trabe esperando respuesta.
- **El dinero se calcula solo en el servidor** (diseño 04). La pantalla muestra lo que el servidor calculó.
- **Bootstrap 5** para el diseño y los componentes (modales, avisos, menús, con su propio JavaScript): trae lo que necesita una caja (tablas compactas, botones táctiles, formularios), sin paso de compilación, el dueño lee sus clases y la v1 ya lo probó en el iPad y en Firefox. Se personaliza con sus variables de color y tipografía.
- **Sin Alpine** mientras Bootstrap y htmx alcancen; entra cuando haya un caso real que ninguno resuelva, y solo para comportamiento visual, nunca reglas del negocio.
- **htmx y Bootstrap se sirven desde la aplicación** (`static/`, versiones fijas en el repo), no de un CDN: la caja no depende de un servicio externo.
- **Sin TypeScript ni paso de compilación del frontend** en las aplicaciones. TypeScript queda solo en las pruebas E2E.

## Descartado
- **Calcular en el navegador con TypeScript:** todo instantáneo, pero son dos cálculos del dinero que con el tiempo se separan. Lo que ve el cajero y lo que teclea en la terminal de Mercado Pago dejaría de coincidir con lo guardado, y la conciliación y el corte no cuadran. Es el problema de la v1.
- **Rust compilado a WebAssembly en el navegador** (el mismo `kernel`): instantáneo y sin diferencias porque es la misma función, pero con un build más complejo y más piezas. No hace falta hoy; queda como salida.
- **Aplicación de una sola página (React, Leptos) con API JSON:** para equipos grandes o para trabajar sin conexión; ninguno es el caso.
- **Tailwind CSS:** la más popular y la IA la escribe bien, pero necesita paso de compilación, no trae modales ni avisos y llena las plantillas de clases difíciles de revisar para el dueño.
- **CSS propio (CSS moderno, `<dialog>` nativo):** ligero y a la medida, pero el dueño no podría revisarlo y la consistencia dependería de disciplina.
- **CSS sin clases (Pico y similares):** se queda corto para tablas densas y botones táctiles.
- **minijinja** (el de xplaya): plantillas que se editan sin recompilar, pero los errores aparecen al abrir la pantalla, no en CI.

## Consecuencias
- **Cuatro revisiones sobre cada pantalla:** el compilador (tipos y plantillas), las pruebas unitarias (las reglas, en Rust), clippy y Playwright (la pantalla completa, en Firefox y en el ancho del iPad).
- **La latencia es una decisión de despliegue, no de código:** un negocio lejano puede tener su instancia más cerca (otra nube, una mini PC en la tienda) sin cambiar nada (ver una instancia por negocio).
- **La prueba E2E de la caja simula latencia** (~300 ms) y verifica que escaneos rápidos terminen completos, en orden y con el total correcto.
- **Si algún día se necesita cálculo instantáneo en el navegador**, el camino es `kernel` a WebAssembly, no reescribirlo en JavaScript. Para mantener esa puerta abierta, `kernel` nunca hace entrada/salida (sin base, sin red).
- **Sin internet, la caja se detiene**, igual que en la v1.
- Cambiar una plantilla requiere recompilar; solo afecta a quien desarrolla, no a los puntos de venta.
