-- Los nombres del catálogo no pasan de 150 caracteres (kernel::nombres::MAX_NOMBRE_CATALOGO).
-- El área lo valida antes con su mensaje; aquí queda como red por si algo escribe sin pasar por ella.

ALTER TABLE categorias
    ADD CONSTRAINT categorias_nombre_largo CHECK (char_length(nombre) <= 150);

ALTER TABLE unidades_medida
    ADD CONSTRAINT unidades_medida_nombre_largo CHECK (char_length(nombre) <= 150);
