use std::collections::BTreeSet;

use kernel::RepoError;
use sqlx::PgPool;
use usuarios::passwords::PasswordHash;
use usuarios::permisos::{Permiso, Rol};
use usuarios::usuarios::{Email, NewUsuario, Usuario, UsuarioError, UsuariosRepo};

#[derive(Clone)]
pub struct PgUsuarios {
    pool: PgPool,
}

impl PgUsuarios {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

impl UsuariosRepo for PgUsuarios {
    async fn add(&self, new_usuario: NewUsuario) -> Result<Usuario, UsuarioError> {
        // Sin fila si el rol no existe en este negocio.
        let inserted = sqlx::query_scalar!(
            "INSERT INTO usuarios (email, nombre, password_hash, rol_id)
             SELECT $1, $2, $3, id FROM roles WHERE nombre = $4
             RETURNING email",
            new_usuario.email().as_str(),
            new_usuario.nombre(),
            new_usuario.password_hash().as_str(),
            new_usuario.rol()
        )
        .fetch_optional(&self.pool)
        .await
        .map_err(
            |e| match e.as_database_error().and_then(|d| d.constraint()) {
                Some("usuarios_email_key") => {
                    UsuarioError::DuplicateEmail(new_usuario.email().as_str().to_string())
                }
                _ => UsuarioError::Repo(RepoError(e.to_string())),
            },
        )?;
        if inserted.is_none() {
            return Err(UsuarioError::RolNotFound(new_usuario.rol().to_string()));
        }
        match self.find_for_login(new_usuario.email()).await? {
            Some((usuario, _)) => Ok(usuario),
            None => Err(UsuarioError::Repo(RepoError(
                "el usuario recién creado no se encontró".into(),
            ))),
        }
    }

    async fn find_for_login(
        &self,
        email: &Email,
    ) -> Result<Option<(Usuario, PasswordHash)>, RepoError> {
        let row = sqlx::query!(
            r#"SELECT u.email, u.nombre, u.password_hash,
                      u.deactivated_at IS NULL AS "is_active!",
                      r.nombre AS rol,
                      array_remove(array_agg(rp.permiso), NULL) AS "permisos!"
               FROM usuarios u
               JOIN roles r ON r.id = u.rol_id
               LEFT JOIN roles_permisos rp ON rp.rol_id = r.id
               WHERE u.email = $1
               GROUP BY u.id, r.id"#,
            email.as_str()
        )
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| RepoError(e.to_string()))?;

        let Some(row) = row else {
            return Ok(None);
        };
        // Un permiso que el código no conoce es un error de datos, no algo que se ignore.
        let permisos = row
            .permisos
            .iter()
            .map(|p| p.parse::<Permiso>())
            .collect::<Result<BTreeSet<_>, _>>()
            .map_err(|e| RepoError(e.to_string()))?;
        let usuario = Usuario {
            email: Email::parse(&row.email).map_err(|e| RepoError(e.to_string()))?,
            nombre: row.nombre,
            rol: Rol {
                nombre: row.rol,
                permisos,
            },
            is_active: row.is_active,
        };
        Ok(Some((
            usuario,
            PasswordHash::from_stored(row.password_hash),
        )))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use usuarios::permisos::default_roles;
    use usuarios::usuarios::contract;

    #[sqlx::test]
    async fn cumple_el_contrato_agregado_se_encuentra(pool: PgPool) {
        contract::agregado_se_encuentra_por_email_sin_importar_mayusculas(&PgUsuarios::new(pool))
            .await;
    }

    #[sqlx::test]
    async fn cumple_el_contrato_email_repetido(pool: PgPool) {
        contract::email_repetido_se_rechaza(&PgUsuarios::new(pool)).await;
    }

    #[sqlx::test]
    async fn cumple_el_contrato_rol_que_no_existe(pool: PgPool) {
        contract::rol_que_no_existe_se_rechaza(&PgUsuarios::new(pool)).await;
    }

    #[sqlx::test]
    async fn cumple_el_contrato_email_que_no_existe(pool: PgPool) {
        contract::email_que_no_existe_no_se_encuentra(&PgUsuarios::new(pool)).await;
    }

    #[sqlx::test]
    async fn los_roles_de_la_migracion_son_los_del_codigo(pool: PgPool) {
        let rows: Vec<(String, String)> = sqlx::query_as(
            "SELECT r.nombre, rp.permiso FROM roles r JOIN roles_permisos rp ON rp.rol_id = r.id",
        )
        .fetch_all(&pool)
        .await
        .unwrap();
        let mut roles: Vec<Rol> = Vec::new();
        for (nombre, permiso) in rows {
            let permiso: Permiso = permiso.parse().unwrap();
            match roles.iter_mut().find(|r| r.nombre == nombre) {
                Some(rol) => {
                    rol.permisos.insert(permiso);
                }
                None => roles.push(Rol {
                    nombre,
                    permisos: BTreeSet::from([permiso]),
                }),
            }
        }
        roles.sort_by(|a, b| a.nombre.cmp(&b.nombre));
        let mut expected = default_roles();
        expected.sort_by(|a, b| a.nombre.cmp(&b.nombre));
        assert_eq!(roles, expected);
    }

    #[sqlx::test]
    async fn un_usuario_desactivado_se_encuentra_como_inactivo(pool: PgPool) {
        let repo = PgUsuarios::new(pool.clone());
        repo.add(NewUsuario::new("ana@x.mx", "Ana", "Dueño", "caja-de-lapices").unwrap())
            .await
            .unwrap();
        sqlx::query("UPDATE usuarios SET deactivated_at = now()")
            .execute(&pool)
            .await
            .unwrap();

        let (usuario, _) = repo
            .find_for_login(&Email::parse("ana@x.mx").unwrap())
            .await
            .unwrap()
            .unwrap();
        assert!(!usuario.is_active);
        assert!(!usuario.can(Permiso::Vender));
    }
}
