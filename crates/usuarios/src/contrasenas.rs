//! Contraseñas con Argon2id (diseño 07). Nunca se guardan ni se registran en claro.

use std::fmt;

use argon2::Argon2;
use argon2::password_hash::phc::PasswordHash;
use argon2::password_hash::{PasswordHasher, PasswordVerifier};

/// Mínimo razonable para una aplicación en internet, y aún tecleable en el iPad.
pub const LONGITUD_MINIMA: usize = 10;

/// Lo que se guarda en la base: el formato PHC (`$argon2id$v=19$...`), con su sal y parámetros.
#[derive(Clone, PartialEq, Eq)]
pub struct HashContrasena(String);

impl HashContrasena {
    /// Uno leído de la base.
    pub fn from_stored(texto: String) -> Self {
        Self(texto)
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

// Sin el hash en los logs ni en los mensajes de error.
impl fmt::Debug for HashContrasena {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("HashContrasena(…)")
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ContrasenaError {
    MuyCorta,
    /// Falla de Argon2 o de la fuente de azar; no depende de lo que escribió el usuario.
    Hash(String),
}

impl fmt::Display for ContrasenaError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::MuyCorta => write!(
                f,
                "La contraseña debe tener al menos {LONGITUD_MINIMA} caracteres."
            ),
            Self::Hash(detalle) => write!(f, "No se pudo proteger la contraseña: {detalle}"),
        }
    }
}

impl std::error::Error for ContrasenaError {}

/// Revisa la longitud y calcula el hash con una sal nueva.
pub fn hash_password(contrasena: &str) -> Result<HashContrasena, ContrasenaError> {
    if contrasena.chars().count() < LONGITUD_MINIMA {
        return Err(ContrasenaError::MuyCorta);
    }
    Argon2::default()
        .hash_password(contrasena.as_bytes())
        .map(|hash| HashContrasena(hash.to_string()))
        .map_err(|e| ContrasenaError::Hash(e.to_string()))
}

/// `false` también si el hash guardado está dañado: nunca deja entrar por error.
pub fn verify_password(contrasena: &str, hash: &HashContrasena) -> bool {
    PasswordHash::new(&hash.0).is_ok_and(|hash| {
        Argon2::default()
            .verify_password(contrasena.as_bytes(), &hash)
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
        assert_eq!(hash_password("123456789"), Err(ContrasenaError::MuyCorta));
        assert!(hash_password("1234567890").is_ok());
    }

    #[test]
    fn la_longitud_cuenta_letras_no_bytes() {
        // 9 letras con acento ocupan más de 10 bytes, pero siguen siendo 9.
        assert_eq!(hash_password("ñáéíóúñáé"), Err(ContrasenaError::MuyCorta));
    }

    #[test]
    fn un_hash_danado_no_deja_entrar() {
        let danado = HashContrasena::from_stored("no-es-un-hash".into());
        assert!(!verify_password("lo-que-sea", &danado));
    }

    #[test]
    fn el_hash_no_aparece_en_debug() {
        let hash = hash_password("caja-de-lapices").unwrap();
        assert_eq!(format!("{hash:?}"), "HashContrasena(…)");
    }
}
