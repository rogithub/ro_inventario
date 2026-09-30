-- Sesiones e intentos fallidos al entrar (diseño 07). Lo vencido se borra en cada intento de entrar.

CREATE TABLE sessions (
    token_hash   bytea       PRIMARY KEY CHECK (length(token_hash) = 32),
    usuario_id   uuid        NOT NULL REFERENCES usuarios ON DELETE CASCADE,
    created_at   timestamptz NOT NULL DEFAULT now(),
    last_used_at timestamptz NOT NULL DEFAULT now()
);
CREATE INDEX sessions_usuario ON sessions (usuario_id);
COMMENT ON TABLE sessions IS
    'Sesiones abiertas. Vencen si no se usan en 7 días (SESSION_DURATION en crates/usuarios); cada uso las renueva.';
COMMENT ON COLUMN sessions.token_hash IS
    'SHA-256 del token que va en la cookie. El token nunca se guarda: con una copia de la base no se puede entrar.';
COMMENT ON COLUMN sessions.usuario_id IS 'Quién entró. Si el usuario se desactiva, su sesión deja de servir.';
COMMENT ON COLUMN sessions.created_at IS 'Cuándo entró.';
COMMENT ON COLUMN sessions.last_used_at IS 'Último uso; de aquí se cuentan los 7 días para que venza.';

CREATE TABLE failed_logins (
    email text        NOT NULL,
    at    timestamptz NOT NULL DEFAULT now()
);
CREATE INDEX failed_logins_email_at ON failed_logins (email, at);
COMMENT ON TABLE failed_logins IS
    'Contraseñas equivocadas por email (exista o no). 5 en 15 minutos bloquean ese email (crates/usuarios).';
COMMENT ON COLUMN failed_logins.email IS 'El email con que se intentó, exista o no (no delata cuáles existen).';
COMMENT ON COLUMN failed_logins.at IS 'Cuándo. Los de hace más de 15 minutos ya no cuentan y se borran.';
