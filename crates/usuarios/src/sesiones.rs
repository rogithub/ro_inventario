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

use crate::contrasenas::{HashContrasena, hash_password, verify_password};
use crate::usuarios::{Email, Usuario, UsuariosRepo};

/// Una sesión se cierra sola si no se usa en este tiempo; cada uso la renueva.
pub const DURACION_SESION: Duration = Duration::from_secs(7 * 24 * 60 * 60);
/// Contraseñas equivocadas seguidas para un email antes de bloquearlo…
pub const INTENTOS_MAXIMOS: u32 = 5;
/// …durante este tiempo.
pub const BLOQUEO: Duration = Duration::from_secs(15 * 60);

/// Lo que va en la cookie: 32 bytes aleatorios en hexadecimal.
#[derive(Clone, PartialEq, Eq)]
pub struct TokenSesion(String);

impl TokenSesion {
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

    pub fn huella(&self) -> HuellaToken {
        HuellaToken(Sha256::digest(self.0.as_bytes()).into())
    }
}

// Sin el token en los logs.
impl fmt::Debug for TokenSesion {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("TokenSesion(…)")
    }
}

/// SHA-256 del token: lo único que se guarda.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct HuellaToken(pub [u8; 32]);

/// Dónde viven las sesiones y los intentos fallidos. Lo implementa `db`; para pruebas, `en_memoria`.
pub trait SesionesRepo {
    fn create(
        &self,
        huella: &HuellaToken,
        email: &Email,
    ) -> impl Future<Output = Result<(), RepoError>> + Send;

    /// El email de una sesión usada dentro de `duracion`, y renueva su último uso.
    fn find_email(
        &self,
        huella: &HuellaToken,
        duracion: Duration,
    ) -> impl Future<Output = Result<Option<Email>, RepoError>> + Send;

    fn delete(&self, huella: &HuellaToken) -> impl Future<Output = Result<(), RepoError>> + Send;

    fn record_failure(&self, email: &Email) -> impl Future<Output = Result<(), RepoError>> + Send;

    /// Intentos fallidos de un email dentro de `ventana`.
    fn count_recent_failures(
        &self,
        email: &Email,
        ventana: Duration,
    ) -> impl Future<Output = Result<u32, RepoError>> + Send;

    fn clear_failures(&self, email: &Email) -> impl Future<Output = Result<(), RepoError>> + Send;

    /// Borra sesiones sin usar en `duracion_sesion` e intentos más viejos que `ventana_fallos`.
    fn purge_expired(
        &self,
        duracion_sesion: Duration,
        ventana_fallos: Duration,
    ) -> impl Future<Output = Result<(), RepoError>> + Send;
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum LoginError {
    /// El mismo para email desconocido, contraseña equivocada o usuario desactivado: no delata
    /// qué emails existen.
    Invalido,
    Bloqueado,
    Repo(RepoError),
}

impl fmt::Display for LoginError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Invalido => write!(f, "Email o contraseña incorrectos."),
            Self::Bloqueado => write!(
                f,
                "Demasiados intentos. Espera {} minutos e inténtalo de nuevo.",
                BLOQUEO.as_secs() / 60
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
static HASH_FALSO: LazyLock<HashContrasena> = LazyLock::new(|| {
    hash_password("contraseña-que-nadie-tiene")
        .unwrap_or_else(|_| HashContrasena::from_stored(String::new()))
});

/// Revisa el bloqueo y la contraseña; si todo está bien, abre una sesión y regresa su token.
pub async fn login(
    usuarios: &impl UsuariosRepo,
    sesiones: &impl SesionesRepo,
    email: &str,
    contrasena: &str,
) -> Result<(Usuario, TokenSesion), LoginError> {
    let Ok(email) = Email::parse(email) else {
        verify_password(contrasena, &HASH_FALSO);
        return Err(LoginError::Invalido);
    };
    if sesiones.count_recent_failures(&email, BLOQUEO).await? >= INTENTOS_MAXIMOS {
        return Err(LoginError::Bloqueado);
    }
    let usuario = match usuarios.find_for_login(&email).await? {
        Some((usuario, hash)) if verify_password(contrasena, &hash) && usuario.activo => {
            Some(usuario)
        }
        Some(_) => None,
        None => {
            verify_password(contrasena, &HASH_FALSO);
            None
        }
    };
    let Some(usuario) = usuario else {
        sesiones.record_failure(&email).await?;
        return Err(LoginError::Invalido);
    };
    sesiones.clear_failures(&email).await?;
    sesiones.purge_expired(DURACION_SESION, BLOQUEO).await?;
    let token = TokenSesion::generate()?;
    sesiones.create(&token.huella(), &email).await?;
    Ok((usuario, token))
}

/// El usuario de la cookie, si la sesión sigue vigente y el usuario activo.
pub async fn current_user(
    usuarios: &impl UsuariosRepo,
    sesiones: &impl SesionesRepo,
    cookie: &str,
) -> Result<Option<Usuario>, RepoError> {
    let Some(token) = TokenSesion::from_cookie(cookie) else {
        return Ok(None);
    };
    let Some(email) = sesiones
        .find_email(&token.huella(), DURACION_SESION)
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

pub async fn logout(sesiones: &impl SesionesRepo, cookie: &str) -> Result<(), RepoError> {
    match TokenSesion::from_cookie(cookie) {
        Some(token) => sesiones.delete(&token.huella()).await,
        None => Ok(()),
    }
}

#[cfg(any(test, feature = "pruebas"))]
pub mod en_memoria {
    use std::sync::Mutex;
    use std::time::Instant;

    use super::*;

    #[derive(Default)]
    pub struct SesionesEnMemoria {
        sesiones: Mutex<Vec<(HuellaToken, Email, Instant)>>,
        fallos: Mutex<Vec<(Email, Instant)>>,
    }

    // Si una prueba falló con el candado tomado, los datos siguen sirviendo para las demás.
    fn lock<T>(m: &Mutex<T>) -> std::sync::MutexGuard<'_, T> {
        m.lock().unwrap_or_else(|e| e.into_inner())
    }

    impl SesionesRepo for SesionesEnMemoria {
        async fn create(&self, huella: &HuellaToken, email: &Email) -> Result<(), RepoError> {
            lock(&self.sesiones).push((huella.clone(), email.clone(), Instant::now()));
            Ok(())
        }

        async fn find_email(
            &self,
            huella: &HuellaToken,
            duracion: Duration,
        ) -> Result<Option<Email>, RepoError> {
            let mut sesiones = lock(&self.sesiones);
            let Some(sesion) = sesiones
                .iter_mut()
                .find(|(h, _, usada)| h == huella && usada.elapsed() < duracion)
            else {
                return Ok(None);
            };
            sesion.2 = Instant::now();
            Ok(Some(sesion.1.clone()))
        }

        async fn delete(&self, huella: &HuellaToken) -> Result<(), RepoError> {
            lock(&self.sesiones).retain(|(h, _, _)| h != huella);
            Ok(())
        }

        async fn record_failure(&self, email: &Email) -> Result<(), RepoError> {
            lock(&self.fallos).push((email.clone(), Instant::now()));
            Ok(())
        }

        async fn count_recent_failures(
            &self,
            email: &Email,
            ventana: Duration,
        ) -> Result<u32, RepoError> {
            let total = lock(&self.fallos)
                .iter()
                .filter(|(e, cuando)| e == email && cuando.elapsed() < ventana)
                .count();
            Ok(u32::try_from(total).unwrap_or(u32::MAX))
        }

        async fn clear_failures(&self, email: &Email) -> Result<(), RepoError> {
            lock(&self.fallos).retain(|(e, _)| e != email);
            Ok(())
        }

        async fn purge_expired(
            &self,
            duracion_sesion: Duration,
            ventana_fallos: Duration,
        ) -> Result<(), RepoError> {
            lock(&self.sesiones).retain(|(_, _, usada)| usada.elapsed() < duracion_sesion);
            lock(&self.fallos).retain(|(_, cuando)| cuando.elapsed() < ventana_fallos);
            Ok(())
        }
    }
}

/// Lo que toda implementación de `SesionesRepo` debe cumplir, junto con su `UsuariosRepo`
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
        sesiones: &impl SesionesRepo,
    ) {
        let email = ana(usuarios).await;
        let huella = TokenSesion::generate().unwrap().huella();

        sesiones.create(&huella, &email).await.unwrap();
        assert_eq!(
            sesiones.find_email(&huella, DURACION_SESION).await.unwrap(),
            Some(email)
        );

        sesiones.delete(&huella).await.unwrap();
        assert_eq!(
            sesiones.find_email(&huella, DURACION_SESION).await.unwrap(),
            None
        );
    }

    pub async fn huella_desconocida_no_se_encuentra(sesiones: &impl SesionesRepo) {
        let huella = TokenSesion::generate().unwrap().huella();
        assert_eq!(
            sesiones.find_email(&huella, DURACION_SESION).await.unwrap(),
            None
        );
    }

    pub async fn los_fallos_se_cuentan_por_email_y_se_borran(sesiones: &impl SesionesRepo) {
        let ana = Email::parse("ana@x.mx").unwrap();
        let beto = Email::parse("beto@x.mx").unwrap();
        for _ in 0..3 {
            sesiones.record_failure(&ana).await.unwrap();
        }
        sesiones.record_failure(&beto).await.unwrap();

        assert_eq!(
            sesiones.count_recent_failures(&ana, BLOQUEO).await.unwrap(),
            3
        );
        assert_eq!(
            sesiones
                .count_recent_failures(&beto, BLOQUEO)
                .await
                .unwrap(),
            1
        );

        sesiones.clear_failures(&ana).await.unwrap();
        assert_eq!(
            sesiones.count_recent_failures(&ana, BLOQUEO).await.unwrap(),
            0
        );
        assert_eq!(
            sesiones
                .count_recent_failures(&beto, BLOQUEO)
                .await
                .unwrap(),
            1
        );
    }

    pub async fn purgar_no_borra_lo_vigente(
        usuarios: &impl UsuariosRepo,
        sesiones: &impl SesionesRepo,
    ) {
        let email = ana(usuarios).await;
        let huella = TokenSesion::generate().unwrap().huella();
        sesiones.create(&huella, &email).await.unwrap();
        sesiones.record_failure(&email).await.unwrap();

        sesiones
            .purge_expired(DURACION_SESION, BLOQUEO)
            .await
            .unwrap();

        assert!(
            sesiones
                .find_email(&huella, DURACION_SESION)
                .await
                .unwrap()
                .is_some()
        );
        assert_eq!(
            sesiones
                .count_recent_failures(&email, BLOQUEO)
                .await
                .unwrap(),
            1
        );
    }
}

#[cfg(test)]
mod tests {
    use super::en_memoria::SesionesEnMemoria;
    use super::*;
    use crate::usuarios::NuevoUsuario;
    use crate::usuarios::en_memoria::UsuariosEnMemoria;

    // --- el token ---

    #[test]
    fn cada_token_es_distinto_y_de_64_caracteres_hexadecimales() {
        let a = TokenSesion::generate().unwrap();
        let b = TokenSesion::generate().unwrap();
        assert_ne!(a, b);
        assert_eq!(a.as_str().len(), 64);
        assert!(a.as_str().chars().all(|c| c.is_ascii_hexdigit()));
    }

    #[test]
    fn la_cookie_solo_acepta_tokens_bien_formados() {
        let token = TokenSesion::generate().unwrap();
        assert_eq!(TokenSesion::from_cookie(token.as_str()), Some(token));
        assert_eq!(TokenSesion::from_cookie(""), None);
        assert_eq!(TokenSesion::from_cookie("abc"), None);
        assert_eq!(TokenSesion::from_cookie(&"z".repeat(64)), None);
    }

    #[test]
    fn la_huella_es_el_sha256_del_token_y_no_el_token() {
        let token = TokenSesion::from_cookie(&"a".repeat(64)).unwrap();
        let esperado: [u8; 32] = Sha256::digest("a".repeat(64).as_bytes()).into();
        assert_eq!(token.huella(), HuellaToken(esperado));
    }

    #[test]
    fn el_token_no_aparece_en_debug() {
        let token = TokenSesion::generate().unwrap();
        assert_eq!(format!("{token:?}"), "TokenSesion(…)");
    }

    // --- el contrato, en memoria ---

    #[tokio::test]
    async fn en_memoria_cumple_el_contrato_sesion_creada() {
        contrato::sesion_creada_se_encuentra_y_borrada_ya_no(
            &UsuariosEnMemoria::default(),
            &SesionesEnMemoria::default(),
        )
        .await;
    }

    #[tokio::test]
    async fn en_memoria_cumple_el_contrato_huella_desconocida() {
        contrato::huella_desconocida_no_se_encuentra(&SesionesEnMemoria::default()).await;
    }

    #[tokio::test]
    async fn en_memoria_cumple_el_contrato_fallos() {
        contrato::los_fallos_se_cuentan_por_email_y_se_borran(&SesionesEnMemoria::default()).await;
    }

    #[tokio::test]
    async fn en_memoria_cumple_el_contrato_purgar() {
        contrato::purgar_no_borra_lo_vigente(
            &UsuariosEnMemoria::default(),
            &SesionesEnMemoria::default(),
        )
        .await;
    }

    // --- entrar, reconocer y salir ---

    async fn con_ana() -> (UsuariosEnMemoria, SesionesEnMemoria) {
        let usuarios = UsuariosEnMemoria::default();
        usuarios
            .add(NuevoUsuario::new("ana@x.mx", "Ana", "Cajero", "caja-de-lapices").unwrap())
            .await
            .unwrap();
        (usuarios, SesionesEnMemoria::default())
    }

    #[tokio::test]
    async fn con_la_contrasena_correcta_entra_y_la_cookie_la_reconoce() {
        let (usuarios, sesiones) = con_ana().await;

        let (usuario, token) = login(&usuarios, &sesiones, " ANA@x.mx ", "caja-de-lapices")
            .await
            .unwrap();
        assert_eq!(usuario.nombre, "Ana");

        let actual = current_user(&usuarios, &sesiones, token.as_str())
            .await
            .unwrap();
        assert_eq!(actual, Some(usuario));
    }

    #[tokio::test]
    async fn una_contrasena_equivocada_no_entra_y_cuenta_como_fallo() {
        let (usuarios, sesiones) = con_ana().await;

        let resultado = login(&usuarios, &sesiones, "ana@x.mx", "caja-de-plumas").await;

        assert_eq!(resultado.map(|_| ()), Err(LoginError::Invalido));
        let ana = Email::parse("ana@x.mx").unwrap();
        assert_eq!(
            sesiones.count_recent_failures(&ana, BLOQUEO).await.unwrap(),
            1
        );
    }

    #[tokio::test]
    async fn un_email_que_no_existe_da_el_mismo_error() {
        let (usuarios, sesiones) = con_ana().await;
        for email in ["nadie@x.mx", "no-es-email"] {
            let resultado = login(&usuarios, &sesiones, email, "caja-de-lapices").await;
            assert_eq!(resultado.map(|_| ()), Err(LoginError::Invalido), "{email}");
        }
    }

    #[tokio::test]
    async fn un_usuario_desactivado_no_entra() {
        let (usuarios, sesiones) = con_ana().await;
        usuarios.deactivate(&Email::parse("ana@x.mx").unwrap());

        let resultado = login(&usuarios, &sesiones, "ana@x.mx", "caja-de-lapices").await;

        assert_eq!(resultado.map(|_| ()), Err(LoginError::Invalido));
    }

    #[tokio::test]
    async fn tras_cinco_fallos_se_bloquea_aunque_la_contrasena_sea_correcta() {
        let (usuarios, sesiones) = con_ana().await;
        for _ in 0..INTENTOS_MAXIMOS {
            let _ = login(&usuarios, &sesiones, "ana@x.mx", "caja-de-plumas").await;
        }

        let resultado = login(&usuarios, &sesiones, "ana@x.mx", "caja-de-lapices").await;

        assert_eq!(resultado.map(|_| ()), Err(LoginError::Bloqueado));
    }

    #[tokio::test]
    async fn entrar_borra_los_fallos_anteriores() {
        let (usuarios, sesiones) = con_ana().await;
        for _ in 0..INTENTOS_MAXIMOS - 1 {
            let _ = login(&usuarios, &sesiones, "ana@x.mx", "caja-de-plumas").await;
        }
        login(&usuarios, &sesiones, "ana@x.mx", "caja-de-lapices")
            .await
            .unwrap();

        let ana = Email::parse("ana@x.mx").unwrap();
        assert_eq!(
            sesiones.count_recent_failures(&ana, BLOQUEO).await.unwrap(),
            0
        );
    }

    #[tokio::test]
    async fn una_cookie_mal_formada_o_desconocida_no_es_nadie() {
        let (usuarios, sesiones) = con_ana().await;
        for cookie in ["", "basura", &"a".repeat(64)] {
            assert_eq!(
                current_user(&usuarios, &sesiones, cookie).await.unwrap(),
                None
            );
        }
    }

    #[tokio::test]
    async fn si_desactivan_al_usuario_su_sesion_deja_de_servir() {
        let (usuarios, sesiones) = con_ana().await;
        let (_, token) = login(&usuarios, &sesiones, "ana@x.mx", "caja-de-lapices")
            .await
            .unwrap();

        usuarios.deactivate(&Email::parse("ana@x.mx").unwrap());

        assert_eq!(
            current_user(&usuarios, &sesiones, token.as_str())
                .await
                .unwrap(),
            None
        );
    }

    #[tokio::test]
    async fn al_salir_la_cookie_deja_de_servir() {
        let (usuarios, sesiones) = con_ana().await;
        let (_, token) = login(&usuarios, &sesiones, "ana@x.mx", "caja-de-lapices")
            .await
            .unwrap();

        logout(&sesiones, token.as_str()).await.unwrap();

        assert_eq!(
            current_user(&usuarios, &sesiones, token.as_str())
                .await
                .unwrap(),
            None
        );
    }
}
