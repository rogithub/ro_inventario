//! Contraseñas con Argon2id (diseño 07). Nunca se guardan ni se registran en claro.

use std::fmt;

use argon2::Argon2;
use argon2::password_hash::phc::PasswordHash as PhcHash;
use argon2::password_hash::{PasswordHasher, PasswordVerifier};

/// Mínimo razonable para una aplicación en internet, y aún tecleable en el iPad.
pub const MIN_PASSWORD_LENGTH: usize = 10;

/// Lo que se guarda en la base: el formato PHC (`$argon2id$v=19$...`), con su sal y parámetros.
#[derive(Clone, PartialEq, Eq)]
pub struct PasswordHash(String);

impl PasswordHash {
    /// Uno leído de la base.
    pub fn from_stored(text: String) -> Self {
        Self(text)
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

// Sin el hash en los logs ni en los mensajes de error.
impl fmt::Debug for PasswordHash {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("PasswordHash(…)")
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PasswordError {
    TooShort,
    /// Falla de Argon2 o de la fuente de azar; no depende de lo que escribió el usuario.
    Hash(String),
}

impl fmt::Display for PasswordError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::TooShort => write!(
                f,
                "La contraseña debe tener al menos {MIN_PASSWORD_LENGTH} caracteres."
            ),
            Self::Hash(detail) => write!(f, "No se pudo proteger la contraseña: {detail}"),
        }
    }
}

impl std::error::Error for PasswordError {}

/// Revisa la longitud y calcula el hash con una sal nueva.
pub fn hash_password(password: &str) -> Result<PasswordHash, PasswordError> {
    if password.chars().count() < MIN_PASSWORD_LENGTH {
        return Err(PasswordError::TooShort);
    }
    Argon2::default()
        .hash_password(password.as_bytes())
        .map(|hash| PasswordHash(hash.to_string()))
        .map_err(|e| PasswordError::Hash(e.to_string()))
}

/// `false` también si el hash guardado está dañado: nunca deja entrar por error.
pub fn verify_password(password: &str, hash: &PasswordHash) -> bool {
    PhcHash::new(&hash.0).is_ok_and(|hash| {
        Argon2::default()
            .verify_password(password.as_bytes(), &hash)
            .is_ok()
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn la_contrasena_correcta_se_verifica_y_una_distinta_no() {
        let hash = hash_password("caja-de-lapices").unwrap();
        assert!(verify_password("caja-de-lapices", &hash));
        assert!(!verify_password("caja-de-lapiceS", &hash));
    }

    #[test]
    fn se_guarda_como_argon2id_y_nunca_en_claro() {
        let hash = hash_password("caja-de-lapices").unwrap();
        assert!(hash.as_str().starts_with("$argon2id$"));
        assert!(!hash.as_str().contains("caja-de-lapices"));
    }

    #[test]
    fn la_misma_contrasena_da_hashes_distintos_por_la_sal() {
        assert_ne!(
            hash_password("caja-de-lapices").unwrap(),
            hash_password("caja-de-lapices").unwrap()
        );
    }

    #[test]
    fn una_contrasena_corta_no_se_acepta() {
        assert_eq!(hash_password("123456789"), Err(PasswordError::TooShort));
        assert!(hash_password("1234567890").is_ok());
    }

    #[test]
    fn la_longitud_cuenta_letras_no_bytes() {
        // 9 letras con acento ocupan más de 10 bytes, pero siguen siendo 9.
        assert_eq!(hash_password("ñáéíóúñáé"), Err(PasswordError::TooShort));
    }

    #[test]
    fn un_hash_danado_no_deja_entrar() {
        let damaged = PasswordHash::from_stored("no-es-un-hash".into());
        assert!(!verify_password("lo-que-sea", &damaged));
    }

    #[test]
    fn el_hash_no_aparece_en_debug() {
        let hash = hash_password("caja-de-lapices").unwrap();
        assert_eq!(format!("{hash:?}"), "PasswordHash(…)");
    }
}
