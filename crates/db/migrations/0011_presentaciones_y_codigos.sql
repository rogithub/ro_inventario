-- Presentaciones y códigos de barras (diseño 01, paso 7e). El área valida antes con sus mensajes
-- (Factor, CodigoBarras, NewPresentacion en crates/inventario); aquí queda como red.

CREATE TABLE presentaciones (
    id           uuid          PRIMARY KEY DEFAULT uuidv7(),
    producto_id  uuid          NOT NULL REFERENCES productos,
    nombre       text          NOT NULL CHECK (btrim(nombre) <> '' AND char_length(nombre) <= 150),
    -- Más de una unidad base: una presentación de 1 sería la pieza suelta (dueño, 2026-10-17).
    factor       numeric(12,3) NOT NULL CHECK (factor > 1),
    precio_venta numeric(12,2) CHECK (precio_venta > 0),
    created_at   timestamptz   NOT NULL DEFAULT now(),
    updated_at   timestamptz   NOT NULL DEFAULT now(),
    updated_by   uuid          REFERENCES usuarios,
    -- Para que un código o un precio solo apunten a una presentación de su mismo producto.
    CONSTRAINT presentaciones_id_producto_unique UNIQUE (id, producto_id)
);

-- El nombre no se repite en un mismo producto sin importar mayúsculas, como en categorías:
-- "Paquete de 100" choca con "PAQUETE DE 100" (dueño, 2026-10-17).
CREATE UNIQUE INDEX presentaciones_producto_nombre_unique ON presentaciones (producto_id, lower(nombre));

COMMENT ON TABLE presentaciones IS
    'Formas de vender (o comprar) un producto en grupo: "Paquete de 100 hojas". Sobre todo para el mayoreo.';
COMMENT ON COLUMN presentaciones.factor IS
    'Cuántas unidades base trae (100, 12, 5000). Mayor que 1.';
COMMENT ON COLUMN presentaciones.precio_venta IS
    'Precio de la presentación completa. NULL = solo se usa para comprar.';
COMMENT ON COLUMN presentaciones.updated_by IS
    'Quién la cambió por última vez. NULL = migrada de la v1.';

CREATE TABLE codigos_barras (
    -- Exacto, con mayúsculas y minúsculas, como lo manda el escáner (dueño, 2026-10-17).
    codigo          text        PRIMARY KEY CHECK (codigo ~ '^\S+$' AND char_length(codigo) <= 50),
    producto_id     uuid        NOT NULL REFERENCES productos,
    presentacion_id uuid,
    created_at      timestamptz NOT NULL DEFAULT now(),
    created_by      uuid        REFERENCES usuarios,
    FOREIGN KEY (presentacion_id, producto_id) REFERENCES presentaciones (id, producto_id)
);
CREATE INDEX codigos_barras_producto ON codigos_barras (producto_id);

COMMENT ON TABLE codigos_barras IS
    'Códigos impresos en un producto o en su empaque. Un producto puede tener varios. El generado del NID no se guarda.';
COMMENT ON COLUMN codigos_barras.presentacion_id IS
    'NULL = el producto suelto. Con valor = esa presentación.';
COMMENT ON COLUMN codigos_barras.created_by IS
    'Quién lo agregó. NULL = migrado de la v1.';

ALTER TABLE precios_historial
    ADD COLUMN presentacion_id uuid,
    ADD FOREIGN KEY (presentacion_id, articulo_id) REFERENCES presentaciones (id, producto_id);

COMMENT ON COLUMN precios_historial.presentacion_id IS
    'NULL = el precio de la unidad del artículo. Con valor = el de esa presentación.';
