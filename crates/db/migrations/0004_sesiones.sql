-- Sesiones e intentos fallidos al entrar (diseño 07). Lo vencido se borra al entrar alguien.

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

CREATE TABLE intentos_fallidos (
    email text        NOT NULL,
    at    timestamptz NOT NULL DEFAULT now()
);
CREATE INDEX intentos_fallidos_email_at ON intentos_fallidos (email, at);
COMMENT ON TABLE intentos_fallidos IS
    'Contraseñas equivocadas por email (exista o no). 5 en 15 minutos bloquean ese email (crates/usuarios).';
