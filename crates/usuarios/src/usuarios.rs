//! Las personas que entran al sistema. Un usuario tiene un rol; se desactiva, nunca se borra.

use std::fmt;
use std::future::Future;

use kernel::RepoError;

use crate::passwords::{PasswordError, PasswordHash, hash_password};
use crate::permisos::{Permiso, Rol};

/// Con él se entra. Se guarda sin espacios y en minúsculas: "Ana@X.mx" y "ana@x.mx" son la misma.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Email(String);

impl Email {
    pub fn parse(texto: &str) -> Result<Self, UsuarioError> {
        let email = texto.trim().to_lowercase();
        let valido = match email.split_once('@') {
            Some((nombre, dominio)) => {
                !nombre.is_empty()
                    && !dominio.is_empty()
                    && !dominio.contains('@')
                    && !email.contains(char::is_whitespace)
            }
            None => false,
        };
        if valido {
            Ok(Self(email))
        } else {
            Err(UsuarioError::EmailInvalido)
        }
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Usuario {
    pub email: Email,
    pub nombre: String,
    pub rol: Rol,
    /// `false` si se desactivó: ya no puede hacer nada, pero sus ventas conservan quién las hizo.
    pub activo: bool,
}

impl Usuario {
    pub fn can(&self, permiso: Permiso) -> bool {
        self.activo && self.rol.can(permiso)
    }
}

/// Un usuario por crear, ya validado y con la contraseña convertida en hash.
#[derive(Debug, Clone)]
pub struct NuevoUsuario {
    email: Email,
    nombre: String,
    rol: String,
    password_hash: PasswordHash,
}

impl NuevoUsuario {
    pub fn new(email: &str, nombre: &str, rol: &str, password: &str) -> Result<Self, UsuarioError> {
        let email = Email::parse(email)?;
        let nombre = nombre.trim();
        if nombre.is_empty() {
            return Err(UsuarioError::NombreVacio);
        }
        let password_hash = hash_password(password).map_err(UsuarioError::Password)?;
        Ok(Self {
            email,
            nombre: nombre.to_string(),
            rol: rol.trim().to_string(),
            password_hash,
        })
    }

    pub fn email(&self) -> &Email {
        &self.email
    }

    pub fn nombre(&self) -> &str {
        &self.nombre
    }

    /// Nombre del rol; el repositorio revisa que exista en este negocio.
    pub fn rol(&self) -> &str {
        &self.rol
    }

    pub fn password_hash(&self) -> &PasswordHash {
        &self.password_hash
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum UsuarioError {
    EmailInvalido,
    NombreVacio,
    Password(PasswordError),
    EmailRepetido(String),
    RolNoExiste(String),
    Repo(RepoError),
}

impl fmt::Display for UsuarioError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::EmailInvalido => write!(f, "El email no es válido."),
            Self::NombreVacio => write!(f, "Escribe el nombre de la persona."),
            Self::Password(error) => write!(f, "{error}"),
            Self::EmailRepetido(email) => write!(f, "Ya existe un usuario con el email {email}."),
            Self::RolNoExiste(rol) => write!(f, "No existe el rol «{rol}»."),
            Self::Repo(_) => write!(f, "No se pudo guardar el usuario."),
        }
    }
}

impl std::error::Error for UsuarioError {}

impl From<RepoError> for UsuarioError {
    fn from(error: RepoError) -> Self {
        Self::Repo(error)
    }
}

/// Dónde viven los usuarios y los roles. Lo implementa `db`; para pruebas, `en_memoria`.
pub trait UsuariosRepo {
    fn add(
        &self,
        nuevo: NuevoUsuario,
    ) -> impl Future<Output = Result<Usuario, UsuarioError>> + Send;

    /// Para entrar: el usuario con el hash de su contraseña, o `None` si no existe.
    fn find_for_login(
        &self,
        email: &Email,
    ) -> impl Future<Output = Result<Option<(Usuario, PasswordHash)>, RepoError>> + Send;
}

#[cfg(any(test, feature = "pruebas"))]
pub mod en_memoria {
    use std::sync::Mutex;

    use super::*;
    use crate::permisos::roles_de_arranque;

    pub struct UsuariosEnMemoria {
        roles: Vec<Rol>,
        usuarios: Mutex<Vec<(Usuario, PasswordHash)>>,
    }

    /// Con los roles de arranque, como una base recién migrada.
    impl Default for UsuariosEnMemoria {
        fn default() -> Self {
            Self {
                roles: roles_de_arranque(),
                usuarios: Mutex::default(),
            }
        }
    }

    impl UsuariosRepo for UsuariosEnMemoria {
        async fn add(&self, nuevo: NuevoUsuario) -> Result<Usuario, UsuarioError> {
            let rol = self
                .roles
                .iter()
                .find(|r| r.nombre == nuevo.rol)
                .ok_or_else(|| UsuarioError::RolNoExiste(nuevo.rol.clone()))?;
            let mut usuarios = self.lock();
            if usuarios.iter().any(|(u, _)| u.email == nuevo.email) {
                return Err(UsuarioError::EmailRepetido(nuevo.email.0));
            }
            let usuario = Usuario {
                email: nuevo.email,
                nombre: nuevo.nombre,
                rol: rol.clone(),
                activo: true,
            };
            usuarios.push((usuario.clone(), nuevo.password_hash));
            Ok(usuario)
        }

        async fn find_for_login(
            &self,
            email: &Email,
        ) -> Result<Option<(Usuario, PasswordHash)>, RepoError> {
            Ok(self.lock().iter().find(|(u, _)| &u.email == email).cloned())
        }
    }

    impl UsuariosEnMemoria {
        /// Como poner `desactivado_at` en la base.
        pub fn deactivate(&self, email: &Email) {
            for (usuario, _) in self.lock().iter_mut() {
                if &usuario.email == email {
                    usuario.activo = false;
                }
            }
        }

        // Si una prueba falló con el candado tomado, los datos siguen sirviendo para las demás.
        fn lock(&self) -> std::sync::MutexGuard<'_, Vec<(Usuario, PasswordHash)>> {
            self.usuarios.lock().unwrap_or_else(|e| e.into_inner())
        }
    }
}

/// Lo que toda implementación de `UsuariosRepo` debe cumplir. Corre contra la de memoria (aquí)
/// y contra la de Postgres (en `db`), las dos empezando sin usuarios y con los roles de arranque.
#[cfg(any(test, feature = "pruebas"))]
#[allow(clippy::unwrap_used)] // código de pruebas
pub mod contrato {
    use std::collections::BTreeSet;

    use super::*;
    use crate::passwords::verify_password;

    fn nuevo(email: &str, rol: &str) -> NuevoUsuario {
        NuevoUsuario::new(email, "Ana López", rol, "caja-de-lapices").unwrap()
    }

    pub async fn agregado_se_encuentra_por_email_sin_importar_mayusculas(repo: &impl UsuariosRepo) {
        repo.add(nuevo("Ana@Papeleria.mx", "Cajero")).await.unwrap();

        let (usuario, hash) = repo
            .find_for_login(&Email::parse("ana@papeleria.mx").unwrap())
            .await
            .unwrap()
            .unwrap();
        assert_eq!(usuario.email.as_str(), "ana@papeleria.mx");
        assert_eq!(usuario.nombre, "Ana López");
        assert_eq!(usuario.rol.nombre, "Cajero");
        assert_eq!(
            usuario.rol.permisos,
            BTreeSet::from([
                Permiso::Vender,
                Permiso::GestionarClientes,
                Permiso::OperarCaja
            ])
        );
        assert!(usuario.activo);
        assert!(verify_password("caja-de-lapices", &hash));
    }

    pub async fn email_repetido_se_rechaza(repo: &impl UsuariosRepo) {
        repo.add(nuevo("ana@papeleria.mx", "Cajero")).await.unwrap();

        let resultado = repo.add(nuevo("ANA@papeleria.mx", "Dueño")).await;

        assert_eq!(
            resultado,
            Err(UsuarioError::EmailRepetido("ana@papeleria.mx".into()))
        );
    }

    pub async fn rol_que_no_existe_se_rechaza(repo: &impl UsuariosRepo) {
        let resultado = repo.add(nuevo("ana@papeleria.mx", "Jefa")).await;

        assert_eq!(resultado, Err(UsuarioError::RolNoExiste("Jefa".into())));
        let email = Email::parse("ana@papeleria.mx").unwrap();
        assert!(repo.find_for_login(&email).await.unwrap().is_none());
    }

    pub async fn email_que_no_existe_no_se_encuentra(repo: &impl UsuariosRepo) {
        let email = Email::parse("nadie@papeleria.mx").unwrap();
        assert!(repo.find_for_login(&email).await.unwrap().is_none());
    }
}

#[cfg(test)]
mod tests {
    use super::en_memoria::UsuariosEnMemoria;
    use super::*;
    use crate::permisos::roles_de_arranque;

    #[test]
    fn el_email_se_guarda_sin_espacios_y_en_minusculas() {
        assert_eq!(
            Email::parse("  Ana@Papeleria.MX ").unwrap().as_str(),
            "ana@papeleria.mx"
        );
    }

    #[test]
    fn un_email_sin_arroba_o_sin_partes_no_es_valido() {
        for texto in ["", "ana", "@papeleria.mx", "ana@", "ana @x.mx", "a@b@c"] {
            assert_eq!(
                Email::parse(texto),
                Err(UsuarioError::EmailInvalido),
                "{texto:?}"
            );
        }
    }

    #[test]
    fn un_usuario_nuevo_necesita_nombre() {
        assert_eq!(
            NuevoUsuario::new("ana@x.mx", "  ", "Cajero", "caja-de-lapices").map(|_| ()),
            Err(UsuarioError::NombreVacio)
        );
    }

    #[test]
    fn un_usuario_nuevo_con_contrasena_corta_no_se_acepta() {
        assert_eq!(
            NuevoUsuario::new("ana@x.mx", "Ana", "Cajero", "corta").map(|_| ()),
            Err(UsuarioError::Password(PasswordError::MuyCorta))
        );
    }

    #[test]
    fn un_usuario_nuevo_queda_limpio() {
        let nuevo =
            NuevoUsuario::new(" Ana@X.mx ", "  Ana López ", " Cajero ", "caja-de-lapices").unwrap();
        assert_eq!(nuevo.email().as_str(), "ana@x.mx");
        assert_eq!(nuevo.nombre(), "Ana López");
        assert_eq!(nuevo.rol(), "Cajero");
    }

    fn usuario(rol: &str, activo: bool) -> Usuario {
        Usuario {
            email: Email::parse("ana@x.mx").unwrap(),
            nombre: "Ana".into(),
            rol: roles_de_arranque()
                .into_iter()
                .find(|r| r.nombre == rol)
                .unwrap(),
            activo,
        }
    }

    #[test]
    fn un_usuario_puede_lo_que_su_rol_permite() {
        let cajero = usuario("Cajero", true);
        assert!(cajero.can(Permiso::Vender));
        assert!(!cajero.can(Permiso::VerCostos));
    }

    #[test]
    fn un_usuario_desactivado_no_puede_nada() {
        let dueno = usuario("Dueño", false);
        assert!(Permiso::ALL.iter().all(|p| !dueno.can(*p)));
    }

    #[tokio::test]
    async fn en_memoria_cumple_el_contrato_agregado_se_encuentra() {
        contrato::agregado_se_encuentra_por_email_sin_importar_mayusculas(
            &UsuariosEnMemoria::default(),
        )
        .await;
    }

    #[tokio::test]
    async fn en_memoria_cumple_el_contrato_email_repetido() {
        contrato::email_repetido_se_rechaza(&UsuariosEnMemoria::default()).await;
    }

    #[tokio::test]
    async fn en_memoria_cumple_el_contrato_rol_que_no_existe() {
        contrato::rol_que_no_existe_se_rechaza(&UsuariosEnMemoria::default()).await;
    }

    #[tokio::test]
    async fn en_memoria_cumple_el_contrato_email_que_no_existe() {
        contrato::email_que_no_existe_no_se_encuentra(&UsuariosEnMemoria::default()).await;
    }
}
