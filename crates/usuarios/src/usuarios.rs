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
    pub fn parse(text: &str) -> Result<Self, UsuarioError> {
        let email = text.trim().to_lowercase();
        let is_valid = match email.split_once('@') {
            Some((local, domain)) => {
                !local.is_empty()
                    && !domain.is_empty()
                    && !domain.contains('@')
                    && !email.contains(char::is_whitespace)
            }
            None => false,
        };
        if is_valid {
            Ok(Self(email))
        } else {
            Err(UsuarioError::InvalidEmail)
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
    pub is_active: bool,
}

impl Usuario {
    pub fn can(&self, permiso: Permiso) -> bool {
        self.is_active && self.rol.can(permiso)
    }
}

/// Un usuario por crear, ya validado y con la contraseña convertida en hash.
#[derive(Debug, Clone)]
pub struct NewUsuario {
    email: Email,
    nombre: String,
    rol: String,
    password_hash: PasswordHash,
}

impl NewUsuario {
    pub fn new(email: &str, nombre: &str, rol: &str, password: &str) -> Result<Self, UsuarioError> {
        let email = Email::parse(email)?;
        let nombre = nombre.trim();
        if nombre.is_empty() {
            return Err(UsuarioError::EmptyNombre);
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
    InvalidEmail,
    EmptyNombre,
    Password(PasswordError),
    DuplicateEmail(String),
    RolNotFound(String),
    Repo(RepoError),
}

impl fmt::Display for UsuarioError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InvalidEmail => write!(f, "El email no es válido."),
            Self::EmptyNombre => write!(f, "Escribe el nombre de la persona."),
            Self::Password(error) => write!(f, "{error}"),
            Self::DuplicateEmail(email) => write!(f, "Ya existe un usuario con el email {email}."),
            Self::RolNotFound(rol) => write!(f, "No existe el rol «{rol}»."),
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

/// Dónde viven los usuarios y los roles. Lo implementa `db`; para pruebas, `in_memory`.
pub trait UsuariosRepo {
    fn add(
        &self,
        new_usuario: NewUsuario,
    ) -> impl Future<Output = Result<Usuario, UsuarioError>> + Send;

    /// Para entrar: el usuario con el hash de su contraseña, o `None` si no existe.
    fn find_for_login(
        &self,
        email: &Email,
    ) -> impl Future<Output = Result<Option<(Usuario, PasswordHash)>, RepoError>> + Send;
}

#[cfg(any(test, feature = "test-support"))]
pub mod in_memory {
    use std::sync::Mutex;

    use super::*;
    use crate::permisos::default_roles;

    pub struct InMemoryUsuarios {
        roles: Vec<Rol>,
        usuarios: Mutex<Vec<(Usuario, PasswordHash)>>,
    }

    /// Con los roles de arranque, como una base recién migrada.
    impl Default for InMemoryUsuarios {
        fn default() -> Self {
            Self {
                roles: default_roles(),
                usuarios: Mutex::default(),
            }
        }
    }

    impl UsuariosRepo for InMemoryUsuarios {
        async fn add(&self, new_usuario: NewUsuario) -> Result<Usuario, UsuarioError> {
            let rol = self
                .roles
                .iter()
                .find(|r| r.nombre == new_usuario.rol)
                .ok_or_else(|| UsuarioError::RolNotFound(new_usuario.rol.clone()))?;
            let mut usuarios = self.lock();
            if usuarios.iter().any(|(u, _)| u.email == new_usuario.email) {
                return Err(UsuarioError::DuplicateEmail(new_usuario.email.0));
            }
            let usuario = Usuario {
                email: new_usuario.email,
                nombre: new_usuario.nombre,
                rol: rol.clone(),
                is_active: true,
            };
            usuarios.push((usuario.clone(), new_usuario.password_hash));
            Ok(usuario)
        }

        async fn find_for_login(
            &self,
            email: &Email,
        ) -> Result<Option<(Usuario, PasswordHash)>, RepoError> {
            Ok(self.lock().iter().find(|(u, _)| &u.email == email).cloned())
        }
    }

    impl InMemoryUsuarios {
        /// Como poner `deactivated_at` en la base.
        pub fn deactivate(&self, email: &Email) {
            for (usuario, _) in self.lock().iter_mut() {
                if &usuario.email == email {
                    usuario.is_active = false;
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
#[cfg(any(test, feature = "test-support"))]
#[allow(clippy::unwrap_used)] // código de pruebas
pub mod contract {
    use std::collections::BTreeSet;

    use super::*;
    use crate::passwords::verify_password;

    fn new_usuario(email: &str, rol: &str) -> NewUsuario {
        NewUsuario::new(email, "Ana López", rol, "caja-de-lapices").unwrap()
    }

    pub async fn agregado_se_encuentra_por_email_sin_importar_mayusculas(repo: &impl UsuariosRepo) {
        repo.add(new_usuario("Ana@Papeleria.mx", "Cajero"))
            .await
            .unwrap();

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
        assert!(usuario.is_active);
        assert!(verify_password("caja-de-lapices", &hash));
    }

    pub async fn email_repetido_se_rechaza(repo: &impl UsuariosRepo) {
        repo.add(new_usuario("ana@papeleria.mx", "Cajero"))
            .await
            .unwrap();

        let result = repo.add(new_usuario("ANA@papeleria.mx", "Dueño")).await;

        assert_eq!(
            result,
            Err(UsuarioError::DuplicateEmail("ana@papeleria.mx".into()))
        );
    }

    pub async fn rol_que_no_existe_se_rechaza(repo: &impl UsuariosRepo) {
        let result = repo.add(new_usuario("ana@papeleria.mx", "Jefa")).await;

        assert_eq!(result, Err(UsuarioError::RolNotFound("Jefa".into())));
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
    use super::in_memory::InMemoryUsuarios;
    use super::*;
    use crate::permisos::default_roles;

    #[test]
    fn el_email_se_guarda_sin_espacios_y_en_minusculas() {
        assert_eq!(
            Email::parse("  Ana@Papeleria.MX ").unwrap().as_str(),
            "ana@papeleria.mx"
        );
    }

    #[test]
    fn un_email_sin_arroba_o_sin_partes_no_es_valido() {
        for text in ["", "ana", "@papeleria.mx", "ana@", "ana @x.mx", "a@b@c"] {
            assert_eq!(
                Email::parse(text),
                Err(UsuarioError::InvalidEmail),
                "{text:?}"
            );
        }
    }

    #[test]
    fn un_usuario_nuevo_necesita_nombre() {
        assert_eq!(
            NewUsuario::new("ana@x.mx", "  ", "Cajero", "caja-de-lapices").map(|_| ()),
            Err(UsuarioError::EmptyNombre)
        );
    }

    #[test]
    fn un_usuario_nuevo_con_contrasena_corta_no_se_acepta() {
        assert_eq!(
            NewUsuario::new("ana@x.mx", "Ana", "Cajero", "corta").map(|_| ()),
            Err(UsuarioError::Password(PasswordError::TooShort))
        );
    }

    #[test]
    fn un_usuario_nuevo_queda_limpio() {
        let new_usuario =
            NewUsuario::new(" Ana@X.mx ", "  Ana López ", " Cajero ", "caja-de-lapices").unwrap();
        assert_eq!(new_usuario.email().as_str(), "ana@x.mx");
        assert_eq!(new_usuario.nombre(), "Ana López");
        assert_eq!(new_usuario.rol(), "Cajero");
    }

    fn usuario(rol: &str, is_active: bool) -> Usuario {
        Usuario {
            email: Email::parse("ana@x.mx").unwrap(),
            nombre: "Ana".into(),
            rol: default_roles()
                .into_iter()
                .find(|r| r.nombre == rol)
                .unwrap(),
            is_active,
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
        contract::agregado_se_encuentra_por_email_sin_importar_mayusculas(
            &InMemoryUsuarios::default(),
        )
        .await;
    }

    #[tokio::test]
    async fn en_memoria_cumple_el_contrato_email_repetido() {
        contract::email_repetido_se_rechaza(&InMemoryUsuarios::default()).await;
    }

    #[tokio::test]
    async fn en_memoria_cumple_el_contrato_rol_que_no_existe() {
        contract::rol_que_no_existe_se_rechaza(&InMemoryUsuarios::default()).await;
    }

    #[tokio::test]
    async fn en_memoria_cumple_el_contrato_email_que_no_existe() {
        contract::email_que_no_existe_no_se_encuentra(&InMemoryUsuarios::default()).await;
    }
}
