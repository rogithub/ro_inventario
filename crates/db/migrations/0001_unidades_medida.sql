-- Unidades en que se cuenta el stock y se cobra (diseño 01).
-- Datos base: solo unidades que sirven a cualquier negocio. Las propias de un negocio
-- (p. ej. "Hoja", "Cuartilla" de la papelería) llegan con sus datos, no aquí.

CREATE TABLE unidades_medida (
    id              uuid    PRIMARY KEY DEFAULT gen_random_uuid(),
    nombre          text    NOT NULL UNIQUE CHECK (btrim(nombre) <> ''),
    allows_fraction boolean NOT NULL
);

COMMENT ON TABLE unidades_medida IS
    'Unidad en que se cuenta el stock de un producto o se cobra un servicio.';
COMMENT ON COLUMN unidades_medida.nombre IS
    'Pieza, Metro, Gramo, Hora…';
COMMENT ON COLUMN unidades_medida.allows_fraction IS
    'Si se puede vender en fracciones (1.5 metros) o solo en enteros (piezas). La caja lo valida.';

INSERT INTO unidades_medida (nombre, allows_fraction) VALUES
    ('Pieza', false),
    ('Metro', true),
    ('Gramo', true),
    ('Hora',  true);
