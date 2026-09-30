//! Entrar, salir y reconocer quién está en la sesión (diseño 07).
//!
//! La cookie lleva un token aleatorio; la base guarda solo su huella SHA-256: con una copia de la
//! base no se puede entrar.

use std::fmt;
use std::future::Future;
use std::sync::LazyLock;
use std::time::Duration;

use kernel::RepoError;
use sha2::{Digest, Sha256};

use crate::passwords::{PasswordHash, hash_password, verify_password};
use crate::usuarios::{Email, Usuario, UsuariosRepo};

/// Una sesión se cierra sola si no se usa en este tiempo; cada uso la renueva.
pub const SESSION_DURATION: Duration = Duration::from_secs(7 * 24 * 60 * 60);
/// Contraseñas equivocadas seguidas para un email antes de bloquearlo…
pub const MAX_FAILED_ATTEMPTS: u32 = 5;
/// …durante este tiempo.
pub const LOCKOUT: Duration = Duration::from_secs(15 * 60);

/// Lo que va en la cookie: 32 bytes aleatorios en hexadecimal.
#[derive(Clone, PartialEq, Eq)]
pub struct SessionToken(String);

impl SessionToken {
    pub fn generate() -> Result<Self, RepoError> {
        let mut bytes = [0u8; 32];
        getrandom::fill(&mut bytes).map_err(|e| RepoError(format!("sin azar del sistema: {e}")))?;
        Ok(Self(bytes.iter().map(|b| format!("{b:02x}")).collect()))
    }

    /// Solo acepta lo que pudo haber salido de `generate`.
    pub fn from_cookie(texto: &str) -> Option<Self> {
        let valido = texto.len() == 64 && texto.chars().all(|c| c.is_ascii_hexdigit());
        valido.then(|| Self(texto.to_ascii_lowercase()))
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }

    pub fn token_hash(&self) -> TokenHash {
        TokenHash(Sha256::digest(self.0.as_bytes()).into())
    }
}

// Sin el token en los logs.
impl fmt::Debug for SessionToken {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("SessionToken(…)")
    }
}

/// SHA-256 del token: lo único que se guarda.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TokenHash(pub [u8; 32]);

/// Dónde viven las sesiones y los intentos fallidos. Lo implementa `db`; para pruebas, `en_memoria`.
pub trait SessionsRepo {
    fn create(
        &self,
        token_hash: &TokenHash,
        email: &Email,
    ) -> impl Future<Output = Result<(), RepoError>> + Send;

    /// El email de una sesión usada dentro de `duracion`, y renueva su último uso.
    fn find_email(
        &self,
        token_hash: &TokenHash,
        duration: Duration,
    ) -> impl Future<Output = Result<Option<Email>, RepoError>> + Send;

    fn delete(&self, token_hash: &TokenHash) -> impl Future<Output = Result<(), RepoError>> + Send;

    fn record_failure(&self, email: &Email) -> impl Future<Output = Result<(), RepoError>> + Send;

    /// Intentos fallidos de un email dentro de `ventana`.
    fn count_recent_failures(
        &self,
        email: &Email,
        window: Duration,
    ) -> impl Future<Output = Result<u32, RepoError>> + Send;

    fn clear_failures(&self, email: &Email) -> impl Future<Output = Result<(), RepoError>> + Send;

    /// Borra sesiones sin usar en `duracion_sesion` e intentos más viejos que `ventana_fallos`.
    fn purge_expired(
        &self,
        session_duration: Duration,
        failures_window: Duration,
    ) -> impl Future<Output = Result<(), RepoError>> + Send;
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum LoginError {
    /// El mismo para email desconocido, contraseña equivocada o usuario desactivado: no delata
    /// qué emails existen.
    Invalid,
    LockedOut,
    Repo(RepoError),
}

impl fmt::Display for LoginError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Invalid => write!(f, "Email o contraseña incorrectos."),
            Self::LockedOut => write!(
                f,
                "Demasiados intentos. Espera {} minutos e inténtalo de nuevo.",
                LOCKOUT.as_secs() / 60
            ),
            Self::Repo(_) => write!(f, "No se pudo entrar."),
        }
    }
}

impl From<RepoError> for LoginError {
    fn from(error: RepoError) -> Self {
        Self::Repo(error)
    }
}

/// Para gastar el mismo tiempo de Argon2 cuando el email no existe.
static DUMMY_HASH: LazyLock<PasswordHash> = LazyLock::new(|| {
    hash_password("contraseña-que-nadie-tiene")
        .unwrap_or_else(|_| PasswordHash::from_stored(String::new()))
});

/// Revisa el bloqueo y la contraseña; si todo está bien, abre una sesión y regresa su token.
pub async fn login(
    usuarios: &impl UsuariosRepo,
    sessions: &impl SessionsRepo,
    email: &str,
    password: &str,
) -> Result<(Usuario, SessionToken), LoginError> {
    let Ok(email) = Email::parse(email) else {
        verify_password(password, &DUMMY_HASH);
        return Err(LoginError::Invalid);
    };
    if sessions.count_recent_failures(&email, LOCKOUT).await? >= MAX_FAILED_ATTEMPTS {
        return Err(LoginError::LockedOut);
    }
    let usuario = match usuarios.find_for_login(&email).await? {
        Some((usuario, hash)) if verify_password(password, &hash) && usuario.activo => {
            Some(usuario)
        }
        Some(_) => None,
        None => {
            verify_password(password, &DUMMY_HASH);
            None
        }
    };
    let Some(usuario) = usuario else {
        sessions.record_failure(&email).await?;
        // También al fallar: si no, la tabla crecería con emails inventados mientras nadie entre.
        sessions.purge_expired(SESSION_DURATION, LOCKOUT).await?;
        return Err(LoginError::Invalid);
    };
    sessions.clear_failures(&email).await?;
    sessions.purge_expired(SESSION_DURATION, LOCKOUT).await?;
    let token = SessionToken::generate()?;
    sessions.create(&token.token_hash(), &email).await?;
    Ok((usuario, token))
}

/// El usuario de la cookie, si la sesión sigue vigente y el usuario activo.
pub async fn current_user(
    usuarios: &impl UsuariosRepo,
    sessions: &impl SessionsRepo,
    cookie: &str,
) -> Result<Option<Usuario>, RepoError> {
    let Some(token) = SessionToken::from_cookie(cookie) else {
        return Ok(None);
    };
    let Some(email) = sessions
        .find_email(&token.token_hash(), SESSION_DURATION)
        .await?
    else {
        return Ok(None);
    };
    Ok(usuarios
        .find_for_login(&email)
        .await?
        .map(|(usuario, _)| usuario)
        .filter(|usuario| usuario.activo))
}

pub async fn logout(sessions: &impl SessionsRepo, cookie: &str) -> Result<(), RepoError> {
    match SessionToken::from_cookie(cookie) {
        Some(token) => sessions.delete(&token.token_hash()).await,
        None => Ok(()),
    }
}

#[cfg(any(test, feature = "pruebas"))]
pub mod en_memoria {
    use std::sync::Mutex;
    use std::time::Instant;

    use super::*;

    #[derive(Default)]
    pub struct SessionsEnMemoria {
        sessions: Mutex<Vec<(TokenHash, Email, Instant)>>,
        failures: Mutex<Vec<(Email, Instant)>>,
    }

    // Si una prueba falló con el candado tomado, los datos siguen sirviendo para las demás.
    fn lock<T>(m: &Mutex<T>) -> std::sync::MutexGuard<'_, T> {
        m.lock().unwrap_or_else(|e| e.into_inner())
    }

    impl SessionsRepo for SessionsEnMemoria {
        async fn create(&self, token_hash: &TokenHash, email: &Email) -> Result<(), RepoError> {
            lock(&self.sessions).push((token_hash.clone(), email.clone(), Instant::now()));
            Ok(())
        }

        async fn find_email(
            &self,
            token_hash: &TokenHash,
            duration: Duration,
        ) -> Result<Option<Email>, RepoError> {
            let mut sessions = lock(&self.sessions);
            let Some(sesion) = sessions
                .iter_mut()
                .find(|(h, _, usada)| h == token_hash && usada.elapsed() < duration)
            else {
                return Ok(None);
            };
            sesion.2 = Instant::now();
            Ok(Some(sesion.1.clone()))
        }

        async fn delete(&self, token_hash: &TokenHash) -> Result<(), RepoError> {
            lock(&self.sessions).retain(|(h, _, _)| h != token_hash);
            Ok(())
        }

        async fn record_failure(&self, email: &Email) -> Result<(), RepoError> {
            lock(&self.failures).push((email.clone(), Instant::now()));
            Ok(())
        }

        async fn count_recent_failures(
            &self,
            email: &Email,
            window: Duration,
        ) -> Result<u32, RepoError> {
            let total = lock(&self.failures)
                .iter()
                .filter(|(e, cuando)| e == email && cuando.elapsed() < window)
                .count();
            Ok(u32::try_from(total).unwrap_or(u32::MAX))
        }

        async fn clear_failures(&self, email: &Email) -> Result<(), RepoError> {
            lock(&self.failures).retain(|(e, _)| e != email);
            Ok(())
        }

        async fn purge_expired(
            &self,
            session_duration: Duration,
            failures_window: Duration,
        ) -> Result<(), RepoError> {
            lock(&self.sessions).retain(|(_, _, usada)| usada.elapsed() < session_duration);
            lock(&self.failures).retain(|(_, cuando)| cuando.elapsed() < failures_window);
            Ok(())
        }
    }
}

/// Lo que toda implementación de `SessionsRepo` debe cumplir, junto con su `UsuariosRepo`
/// (una sesión es de un usuario que existe). Corre contra memoria (aquí) y Postgres (en `db`).
#[cfg(any(test, feature = "pruebas"))]
#[allow(clippy::unwrap_used)] // código de pruebas
pub mod contrato {
    use super::*;
    use crate::usuarios::NuevoUsuario;

    async fn ana(usuarios: &impl UsuariosRepo) -> Email {
        usuarios
            .add(NuevoUsuario::new("ana@x.mx", "Ana", "Cajero", "caja-de-lapices").unwrap())
            .await
            .unwrap();
        Email::parse("ana@x.mx").unwrap()
    }

    pub async fn sesion_creada_se_encuentra_y_borrada_ya_no(
        usuarios: &impl UsuariosRepo,
        sessions: &impl SessionsRepo,
    ) {
        let email = ana(usuarios).await;
        let token_hash = SessionToken::generate().unwrap().token_hash();

        sessions.create(&token_hash, &email).await.unwrap();
        assert_eq!(
            sessions
                .find_email(&token_hash, SESSION_DURATION)
                .await
                .unwrap(),
            Some(email)
        );

        sessions.delete(&token_hash).await.unwrap();
        assert_eq!(
            sessions
                .find_email(&token_hash, SESSION_DURATION)
                .await
                .unwrap(),
            None
        );
    }

    pub async fn huella_desconocida_no_se_encuentra(sessions: &impl SessionsRepo) {
        let token_hash = SessionToken::generate().unwrap().token_hash();
        assert_eq!(
            sessions
                .find_email(&token_hash, SESSION_DURATION)
                .await
                .unwrap(),
            None
        );
    }

    pub async fn los_fallos_se_cuentan_por_email_y_se_borran(sessions: &impl SessionsRepo) {
        let ana = Email::parse("ana@x.mx").unwrap();
        let beto = Email::parse("beto@x.mx").unwrap();
        for _ in 0..3 {
            sessions.record_failure(&ana).await.unwrap();
        }
        sessions.record_failure(&beto).await.unwrap();

        assert_eq!(
            sessions.count_recent_failures(&ana, LOCKOUT).await.unwrap(),
            3
        );
        assert_eq!(
            sessions
                .count_recent_failures(&beto, LOCKOUT)
                .await
                .unwrap(),
            1
        );

        sessions.clear_failures(&ana).await.unwrap();
        assert_eq!(
            sessions.count_recent_failures(&ana, LOCKOUT).await.unwrap(),
            0
        );
        assert_eq!(
            sessions
                .count_recent_failures(&beto, LOCKOUT)
                .await
                .unwrap(),
            1
        );
    }

    pub async fn purgar_no_borra_lo_vigente(
        usuarios: &impl UsuariosRepo,
        sessions: &impl SessionsRepo,
    ) {
        let email = ana(usuarios).await;
        let token_hash = SessionToken::generate().unwrap().token_hash();
        sessions.create(&token_hash, &email).await.unwrap();
        sessions.record_failure(&email).await.unwrap();

        sessions
            .purge_expired(SESSION_DURATION, LOCKOUT)
            .await
            .unwrap();

        assert!(
            sessions
                .find_email(&token_hash, SESSION_DURATION)
                .await
                .unwrap()
                .is_some()
        );
        assert_eq!(
            sessions
                .count_recent_failures(&email, LOCKOUT)
                .await
                .unwrap(),
            1
        );
    }
}

#[cfg(test)]
mod tests {
    use super::en_memoria::SessionsEnMemoria;
    use super::*;
    use crate::usuarios::NuevoUsuario;
    use crate::usuarios::en_memoria::UsuariosEnMemoria;

    // --- el token ---

    #[test]
    fn cada_token_es_distinto_y_de_64_caracteres_hexadecimales() {
        let a = SessionToken::generate().unwrap();
        let b = SessionToken::generate().unwrap();
        assert_ne!(a, b);
        assert_eq!(a.as_str().len(), 64);
        assert!(a.as_str().chars().all(|c| c.is_ascii_hexdigit()));
    }

    #[test]
    fn la_cookie_solo_acepta_tokens_bien_formados() {
        let token = SessionToken::generate().unwrap();
        assert_eq!(SessionToken::from_cookie(token.as_str()), Some(token));
        assert_eq!(SessionToken::from_cookie(""), None);
        assert_eq!(SessionToken::from_cookie("abc"), None);
        assert_eq!(SessionToken::from_cookie(&"z".repeat(64)), None);
    }

    #[test]
    fn la_huella_es_el_sha256_del_token_y_no_el_token() {
        let token = SessionToken::from_cookie(&"a".repeat(64)).unwrap();
        let esperado: [u8; 32] = Sha256::digest("a".repeat(64).as_bytes()).into();
        assert_eq!(token.token_hash(), TokenHash(esperado));
    }

    #[test]
    fn el_token_no_aparece_en_debug() {
        let token = SessionToken::generate().unwrap();
        assert_eq!(format!("{token:?}"), "SessionToken(…)");
    }

    // --- el contrato, en memoria ---

    #[tokio::test]
    async fn en_memoria_cumple_el_contrato_sesion_creada() {
        contrato::sesion_creada_se_encuentra_y_borrada_ya_no(
            &UsuariosEnMemoria::default(),
            &SessionsEnMemoria::default(),
        )
        .await;
    }

    #[tokio::test]
    async fn en_memoria_cumple_el_contrato_huella_desconocida() {
        contrato::huella_desconocida_no_se_encuentra(&SessionsEnMemoria::default()).await;
    }

    #[tokio::test]
    async fn en_memoria_cumple_el_contrato_fallos() {
        contrato::los_fallos_se_cuentan_por_email_y_se_borran(&SessionsEnMemoria::default()).await;
    }

    #[tokio::test]
    async fn en_memoria_cumple_el_contrato_purgar() {
        contrato::purgar_no_borra_lo_vigente(
            &UsuariosEnMemoria::default(),
            &SessionsEnMemoria::default(),
        )
        .await;
    }

    // --- entrar, reconocer y salir ---

    async fn con_ana() -> (UsuariosEnMemoria, SessionsEnMemoria) {
        let usuarios = UsuariosEnMemoria::default();
        usuarios
            .add(NuevoUsuario::new("ana@x.mx", "Ana", "Cajero", "caja-de-lapices").unwrap())
            .await
            .unwrap();
        (usuarios, SessionsEnMemoria::default())
    }

    #[tokio::test]
    async fn con_la_contrasena_correcta_entra_y_la_cookie_la_reconoce() {
        let (usuarios, sessions) = con_ana().await;

        let (usuario, token) = login(&usuarios, &sessions, " ANA@x.mx ", "caja-de-lapices")
            .await
            .unwrap();
        assert_eq!(usuario.nombre, "Ana");

        let actual = current_user(&usuarios, &sessions, token.as_str())
            .await
            .unwrap();
        assert_eq!(actual, Some(usuario));
    }

    #[tokio::test]
    async fn una_contrasena_equivocada_no_entra_y_cuenta_como_fallo() {
        let (usuarios, sessions) = con_ana().await;

        let resultado = login(&usuarios, &sessions, "ana@x.mx", "caja-de-plumas").await;

        assert_eq!(resultado.map(|_| ()), Err(LoginError::Invalid));
        let ana = Email::parse("ana@x.mx").unwrap();
        assert_eq!(
            sessions.count_recent_failures(&ana, LOCKOUT).await.unwrap(),
            1
        );
    }

    #[tokio::test]
    async fn un_email_que_no_existe_da_el_mismo_error() {
        let (usuarios, sessions) = con_ana().await;
        for email in ["nadie@x.mx", "no-es-email"] {
            let resultado = login(&usuarios, &sessions, email, "caja-de-lapices").await;
            assert_eq!(resultado.map(|_| ()), Err(LoginError::Invalid), "{email}");
        }
    }

    #[tokio::test]
    async fn un_usuario_desactivado_no_entra() {
        let (usuarios, sessions) = con_ana().await;
        usuarios.deactivate(&Email::parse("ana@x.mx").unwrap());

        let resultado = login(&usuarios, &sessions, "ana@x.mx", "caja-de-lapices").await;

        assert_eq!(resultado.map(|_| ()), Err(LoginError::Invalid));
    }

    #[tokio::test]
    async fn tras_cinco_fallos_se_bloquea_aunque_la_contrasena_sea_correcta() {
        let (usuarios, sessions) = con_ana().await;
        for _ in 0..MAX_FAILED_ATTEMPTS {
            let _ = login(&usuarios, &sessions, "ana@x.mx", "caja-de-plumas").await;
        }

        let resultado = login(&usuarios, &sessions, "ana@x.mx", "caja-de-lapices").await;

        assert_eq!(resultado.map(|_| ()), Err(LoginError::LockedOut));
    }

    #[tokio::test]
    async fn entrar_borra_los_fallos_anteriores() {
        let (usuarios, sessions) = con_ana().await;
        for _ in 0..MAX_FAILED_ATTEMPTS - 1 {
            let _ = login(&usuarios, &sessions, "ana@x.mx", "caja-de-plumas").await;
        }
        login(&usuarios, &sessions, "ana@x.mx", "caja-de-lapices")
            .await
            .unwrap();

        let ana = Email::parse("ana@x.mx").unwrap();
        assert_eq!(
            sessions.count_recent_failures(&ana, LOCKOUT).await.unwrap(),
            0
        );
    }

    #[tokio::test]
    async fn una_cookie_mal_formada_o_desconocida_no_es_nadie() {
        let (usuarios, sessions) = con_ana().await;
        for cookie in ["", "basura", &"a".repeat(64)] {
            assert_eq!(
                current_user(&usuarios, &sessions, cookie).await.unwrap(),
                None
            );
        }
    }

    #[tokio::test]
    async fn si_desactivan_al_usuario_su_sesion_deja_de_servir() {
        let (usuarios, sessions) = con_ana().await;
        let (_, token) = login(&usuarios, &sessions, "ana@x.mx", "caja-de-lapices")
            .await
            .unwrap();

        usuarios.deactivate(&Email::parse("ana@x.mx").unwrap());

        assert_eq!(
            current_user(&usuarios, &sessions, token.as_str())
                .await
                .unwrap(),
            None
        );
    }

    #[tokio::test]
    async fn al_salir_la_cookie_deja_de_servir() {
        let (usuarios, sessions) = con_ana().await;
        let (_, token) = login(&usuarios, &sessions, "ana@x.mx", "caja-de-lapices")
            .await
            .unwrap();

        logout(&sessions, token.as_str()).await.unwrap();

        assert_eq!(
            current_user(&usuarios, &sessions, token.as_str())
                .await
                .unwrap(),
            None
        );
    }
}
