-- Largos máximos de los textos de un artículo y de un producto (decididos por el dueño el 2026-10-01).
-- El área lo valida antes con su mensaje (MAX_DESCRIPCION, MAX_MARCA… en crates/inventario);
-- aquí queda como red por si algo escribe sin pasar por ella.

ALTER TABLE articulos
    ADD CONSTRAINT articulos_descripcion_largo CHECK (char_length(descripcion) <= 1000);

ALTER TABLE productos
    ADD CONSTRAINT productos_marca_largo  CHECK (char_length(marca)  <= 150),
    ADD CONSTRAINT productos_modelo_largo CHECK (char_length(modelo) <= 200),
    ADD CONSTRAINT productos_color_largo  CHECK (char_length(color)  <= 150);
