-- Categorías del catálogo (diseño 01): planas, sin jerarquía; productos y servicios las comparten.
-- Sin datos base: las categorías son de cada negocio (las de la papelería llegan con la migración de la v1).

CREATE TABLE categorias (
    id     uuid PRIMARY KEY DEFAULT gen_random_uuid(),
    nombre text NOT NULL CHECK (btrim(nombre) <> '')
);

-- El nombre no se repite sin importar mayúsculas: "cuadernos" choca con "Cuadernos".
CREATE UNIQUE INDEX categorias_nombre_unique ON categorias (lower(nombre));

COMMENT ON TABLE categorias IS
    'Agrupación del catálogo. Cada artículo lleva exactamente una.';
COMMENT ON COLUMN categorias.nombre IS
    'Cuadernos, Plumas, Impresiones…';
