-- Sesiones e intentos fallidos al entrar (diseño 07). Lo vencido se borra en cada intento de entrar.

CREATE TABLE sesiones (
    huella       bytea       PRIMARY KEY CHECK (length(huella) = 32),
    usuario_id   uuid        NOT NULL REFERENCES usuarios ON DELETE CASCADE,
    created_at   timestamptz NOT NULL DEFAULT now(),
    last_used_at timestamptz NOT NULL DEFAULT now()
);
CREATE INDEX sesiones_usuario ON sesiones (usuario_id);
COMMENT ON TABLE sesiones IS
    'Sesiones abiertas. Vencen si no se usan en 7 días (DURACION_SESION en crates/usuarios); cada uso las renueva.';
COMMENT ON COLUMN sesiones.huella IS
    'SHA-256 del token que va en la cookie. El token nunca se guarda: con una copia de la base no se puede entrar.';
COMMENT ON COLUMN sesiones.usuario_id IS 'Quién entró. Si el usuario se desactiva, su sesión deja de servir.';
COMMENT ON COLUMN sesiones.created_at IS 'Cuándo entró.';
COMMENT ON COLUMN sesiones.last_used_at IS 'Último uso; de aquí se cuentan los 7 días para que venza.';

CREATE TABLE intentos_fallidos (
    email text        NOT NULL,
    at    timestamptz NOT NULL DEFAULT now()
);
CREATE INDEX intentos_fallidos_email_at ON intentos_fallidos (email, at);
COMMENT ON TABLE intentos_fallidos IS
    'Contraseñas equivocadas por email (exista o no). 5 en 15 minutos bloquean ese email (crates/usuarios).';
COMMENT ON COLUMN intentos_fallidos.email IS 'El email con que se intentó, exista o no (no delata cuáles existen).';
COMMENT ON COLUMN intentos_fallidos.at IS 'Cuándo. Los de hace más de 15 minutos ya no cuentan y se borran.';
