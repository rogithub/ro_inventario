-- Largo máximo del motivo de descontinuar (decidido por el dueño el 2026-10-07). El área lo valida
-- antes con su mensaje (MAX_MOTIVO_DESCONTINUADO en crates/inventario); aquí queda como red.

ALTER TABLE articulos
    ADD CONSTRAINT articulos_motivo_descontinuado_largo
        CHECK (char_length(motivo_descontinuado) <= 300);
