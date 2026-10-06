# Un 403 o un 404 al navegar sale como un pedazo de HTML, sin la página

**Salió de:** revisor de la parte 7c1b, 2026-10-06 · **Cuándo:** antes de que lo usen otros que no sean el dueño, o con la siguiente pantalla que reciba un id en la dirección

## Qué pasa
`forbidden()` y `not_found()` de `apps/privada/src/productos/editar.rs` (y `forbidden()` de `nuevo.rs`, e `internal_error` de `lib.rs`) responden solo un `<div class="alert">`. Con htmx está bien: se pinta en su lugar. Pero al llegar navegando (un enlace viejo a `/productos/999`, escribir la dirección, regresar después de un cambio de rol) se ve una alerta suelta, sin menú ni estilos ni forma de regresar.

## Por qué importa
Con el NID en la dirección, un 404 es más probable que antes. Un cajero que se topa con eso no sabe qué hacer.

## Propuesta
Un ayudante común en `lib.rs`: con htmx, solo la alerta; sin htmx, la alerta dentro de `base.html` con el menú. Se aplica a los tres de una vez.
