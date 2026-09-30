-- Usuarios, roles y permisos (diseño 07). El código revisa permisos, nunca el nombre de un rol.

CREATE TABLE roles (
    id     uuid PRIMARY KEY DEFAULT gen_random_uuid(),
    nombre text NOT NULL UNIQUE CHECK (btrim(nombre) <> '')
);
COMMENT ON TABLE roles IS
    'Conjunto de permisos con nombre. Es un dato de cada negocio: cada uno puede tener los suyos.';

CREATE TABLE roles_permisos (
    rol_id  uuid NOT NULL REFERENCES roles ON DELETE CASCADE,
    permiso text NOT NULL,
    PRIMARY KEY (rol_id, permiso)
);
COMMENT ON TABLE roles_permisos IS 'Los permisos de cada rol.';
COMMENT ON COLUMN roles_permisos.permiso IS
    'Uno de la lista del código (enum Permiso en crates/usuarios): vender, ver_costos, administrar_usuarios…';

CREATE TABLE usuarios (
    id             uuid        PRIMARY KEY DEFAULT gen_random_uuid(),
    email          text        NOT NULL UNIQUE CHECK (email = lower(btrim(email)) AND email LIKE '_%@_%'),
    nombre         text        NOT NULL CHECK (btrim(nombre) <> ''),
    password_hash  text        NOT NULL,
    rol_id         uuid        NOT NULL REFERENCES roles,
    desactivado_at timestamptz,
    created_at     timestamptz NOT NULL DEFAULT now()
);
COMMENT ON TABLE usuarios IS
    'Personas que entran al sistema. Un usuario que se va se desactiva, no se borra: sus ventas conservan quién las hizo.';
COMMENT ON COLUMN usuarios.email IS 'Con él se entra. Se guarda en minúsculas y sin espacios.';
COMMENT ON COLUMN usuarios.password_hash IS 'Argon2id en formato PHC ($argon2id$v=19$...). Nunca la contraseña.';
COMMENT ON COLUMN usuarios.desactivado_at IS 'Desde cuándo ya no puede entrar. NULL = activo.';

-- Datos base: los roles de arranque que sirven a cualquier negocio. Los propios de un negocio
-- (p. ej. "Socia" de la papelería) llegan con sus datos. Deben coincidir con
-- roles_de_arranque() en crates/usuarios; una prueba en crates/db lo verifica.
INSERT INTO roles (nombre) VALUES ('Dueño'), ('Encargado'), ('Cajero');

INSERT INTO roles_permisos (rol_id, permiso)
SELECT r.id, p.permiso
FROM roles r
CROSS JOIN (VALUES
    ('vender'), ('cancelar_venta'), ('devolver'), ('ver_costos'), ('editar_catalogo'),
    ('comprar'), ('ajustar_inventario'), ('operar_caja'), ('gestionar_clientes'),
    ('vetar_clientes'), ('ver_reportes'), ('configurar_negocio'), ('administrar_usuarios')
) AS p (permiso)
WHERE r.nombre = 'Dueño'
   OR (r.nombre = 'Encargado' AND p.permiso NOT IN ('configurar_negocio', 'administrar_usuarios'))
   OR (r.nombre = 'Cajero' AND p.permiso IN ('vender', 'gestionar_clientes', 'operar_caja'));
