//! Área del negocio: usuarios, roles y permisos (diseño 07).
//!
//! Pura: no depende de la base de datos, de la web ni de la red. El código revisa permisos,
//! nunca el nombre de un rol.

pub mod passwords;
pub mod permisos;
pub mod sessions;
pub mod usuarios;
