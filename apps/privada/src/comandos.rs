//! Comandos de administración que corren sin la web: `privada crear-usuario …`.

use usuarios::usuarios::{NuevoUsuario, Usuario, UsuarioError, UsuariosRepo};

pub const USO_CREAR_USUARIO: &str = "uso: privada crear-usuario --email <email> --nombre <nombre> --rol <rol>\n\
     La contraseña se pide sin mostrarla (o se lee de la entrada si no hay terminal).";

#[derive(Debug, PartialEq, Eq)]
pub struct DatosUsuario {
    pub email: String,
    pub nombre: String,
    pub rol: String,
}

/// Lee `--email`, `--nombre` y `--rol` (los tres obligatorios, en cualquier orden).
pub fn parse_crear_usuario(args: &[String]) -> Result<DatosUsuario, String> {
    let (mut email, mut nombre, mut rol) = (None, None, None);
    let mut resto = args.iter();
    while let Some(opcion) = resto.next() {
        let destino = match opcion.as_str() {
            "--email" => &mut email,
            "--nombre" => &mut nombre,
            "--rol" => &mut rol,
            otra => return Err(format!("opción desconocida: {otra}\n{USO_CREAR_USUARIO}")),
        };
        match resto.next() {
            Some(valor) => *destino = Some(valor.clone()),
            None => return Err(format!("{opcion} necesita un valor\n{USO_CREAR_USUARIO}")),
        }
    }
    let falta = |opcion: &str| format!("falta {opcion}\n{USO_CREAR_USUARIO}");
    Ok(DatosUsuario {
        email: email.ok_or_else(|| falta("--email"))?,
        nombre: nombre.ok_or_else(|| falta("--nombre"))?,
        rol: rol.ok_or_else(|| falta("--rol"))?,
    })
}

/// El primer usuario de un negocio, o uno nuevo. No hay usuario administrador por omisión.
pub async fn crear_usuario(
    repo: &impl UsuariosRepo,
    datos: &DatosUsuario,
    contrasena: &str,
) -> Result<Usuario, UsuarioError> {
    let nuevo = NuevoUsuario::new(&datos.email, &datos.nombre, &datos.rol, contrasena)?;
    repo.add(nuevo).await
}

#[cfg(test)]
mod tests {
    use super::*;
    use usuarios::contrasenas::verify_password;
    use usuarios::usuarios::Email;
    use usuarios::usuarios::en_memoria::UsuariosEnMemoria;

    fn args(texto: &str) -> Vec<String> {
        texto.split('|').map(String::from).collect()
    }

    #[test]
    fn lee_los_tres_datos_en_cualquier_orden() {
        assert_eq!(
            parse_crear_usuario(&args("--rol|Dueño|--email|ana@x.mx|--nombre|Ana López")),
            Ok(DatosUsuario {
                email: "ana@x.mx".into(),
                nombre: "Ana López".into(),
                rol: "Dueño".into(),
            })
        );
    }

    #[test]
    fn si_falta_un_dato_explica_el_uso() {
        let error = parse_crear_usuario(&args("--email|ana@x.mx|--nombre|Ana")).unwrap_err();
        assert!(error.contains("falta --rol"), "{error}");
        assert!(error.contains(USO_CREAR_USUARIO));
    }

    #[test]
    fn una_opcion_sin_valor_o_desconocida_no_se_acepta() {
        assert!(parse_crear_usuario(&args("--email")).is_err());
        assert!(
            parse_crear_usuario(&args("--email|a@x.mx|--nombre|Ana|--rol|Dueño|--admin|si"))
                .unwrap_err()
                .contains("--admin")
        );
    }

    #[tokio::test]
    async fn crea_el_usuario_con_su_contrasena() {
        let repo = UsuariosEnMemoria::default();
        let datos = DatosUsuario {
            email: "Ana@X.mx".into(),
            nombre: "Ana".into(),
            rol: "Dueño".into(),
        };

        let usuario = crear_usuario(&repo, &datos, "caja-de-lapices")
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
        let repo = UsuariosEnMemoria::default();
        let datos = DatosUsuario {
            email: "ana@x.mx".into(),
            nombre: "Ana".into(),
            rol: "Dueño".into(),
        };

        assert!(crear_usuario(&repo, &datos, "corta").await.is_err());
        let email = Email::parse("ana@x.mx").unwrap();
        assert!(repo.find_for_login(&email).await.unwrap().is_none());
    }
}
