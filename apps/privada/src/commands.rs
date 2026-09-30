//! Comandos de administración que corren sin la web: `privada crear-usuario …`.

use usuarios::usuarios::{NewUsuario, Usuario, UsuarioError, UsuariosRepo};

pub const CREATE_USUARIO_USAGE: &str = "uso: privada crear-usuario --email <email> --nombre <nombre> --rol <rol>\n\
     La contraseña se pide sin mostrarla (o se lee de la entrada si no hay terminal).";

#[derive(Debug, PartialEq, Eq)]
pub struct NewUsuarioArgs {
    pub email: String,
    pub nombre: String,
    pub rol: String,
}

/// Lee `--email`, `--nombre` y `--rol` (los tres obligatorios, en cualquier orden).
pub fn parse_create_usuario(args: &[String]) -> Result<NewUsuarioArgs, String> {
    let (mut email, mut nombre, mut rol) = (None, None, None);
    let mut rest = args.iter();
    while let Some(option) = rest.next() {
        let target = match option.as_str() {
            "--email" => &mut email,
            "--nombre" => &mut nombre,
            "--rol" => &mut rol,
            other => {
                return Err(format!(
                    "opción desconocida: {other}\n{CREATE_USUARIO_USAGE}"
                ));
            }
        };
        match rest.next() {
            Some(value) => *target = Some(value.clone()),
            None => {
                return Err(format!(
                    "{option} necesita un valor\n{CREATE_USUARIO_USAGE}"
                ));
            }
        }
    }
    let missing = |option: &str| format!("falta {option}\n{CREATE_USUARIO_USAGE}");
    Ok(NewUsuarioArgs {
        email: email.ok_or_else(|| missing("--email"))?,
        nombre: nombre.ok_or_else(|| missing("--nombre"))?,
        rol: rol.ok_or_else(|| missing("--rol"))?,
    })
}

/// El primer usuario de un negocio, o uno nuevo. No hay usuario administrador por omisión.
pub async fn create_usuario(
    repo: &impl UsuariosRepo,
    input: &NewUsuarioArgs,
    password: &str,
) -> Result<Usuario, UsuarioError> {
    let new_usuario = NewUsuario::new(&input.email, &input.nombre, &input.rol, password)?;
    repo.add(new_usuario).await
}

#[cfg(test)]
mod tests {
    use super::*;
    use usuarios::passwords::verify_password;
    use usuarios::usuarios::Email;
    use usuarios::usuarios::in_memory::InMemoryUsuarios;

    fn args(text: &str) -> Vec<String> {
        text.split('|').map(String::from).collect()
    }

    #[test]
    fn lee_los_tres_datos_en_cualquier_orden() {
        assert_eq!(
            parse_create_usuario(&args("--rol|Dueño|--email|ana@x.mx|--nombre|Ana López")),
            Ok(NewUsuarioArgs {
                email: "ana@x.mx".into(),
                nombre: "Ana López".into(),
                rol: "Dueño".into(),
            })
        );
    }

    #[test]
    fn si_falta_un_dato_explica_el_uso() {
        let error = parse_create_usuario(&args("--email|ana@x.mx|--nombre|Ana")).unwrap_err();
        assert!(error.contains("falta --rol"), "{error}");
        assert!(error.contains(CREATE_USUARIO_USAGE));
    }

    #[test]
    fn una_opcion_sin_valor_o_desconocida_no_se_acepta() {
        assert!(parse_create_usuario(&args("--email")).is_err());
        assert!(
            parse_create_usuario(&args("--email|a@x.mx|--nombre|Ana|--rol|Dueño|--admin|si"))
                .unwrap_err()
                .contains("--admin")
        );
    }

    #[tokio::test]
    async fn crea_el_usuario_con_su_contrasena() {
        let repo = InMemoryUsuarios::default();
        let input = NewUsuarioArgs {
            email: "Ana@X.mx".into(),
            nombre: "Ana".into(),
            rol: "Dueño".into(),
        };

        let usuario = create_usuario(&repo, &input, "caja-de-lapices")
            .await
            .unwrap();

        assert_eq!(usuario.email.as_str(), "ana@x.mx");
        let (_, hash) = repo
            .find_for_login(&Email::parse("ana@x.mx").unwrap())
            .await
            .unwrap()
            .unwrap();
        assert!(verify_password("caja-de-lapices", &hash));
    }

    #[tokio::test]
    async fn una_contrasena_corta_no_crea_nada() {
        let repo = InMemoryUsuarios::default();
        let input = NewUsuarioArgs {
            email: "ana@x.mx".into(),
            nombre: "Ana".into(),
            rol: "Dueño".into(),
        };

        assert!(create_usuario(&repo, &input, "corta").await.is_err());
        let email = Email::parse("ana@x.mx").unwrap();
        assert!(repo.find_for_login(&email).await.unwrap().is_none());
    }
}
