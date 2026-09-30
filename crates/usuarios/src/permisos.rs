//! Qué puede hacer cada quien. La lista vive aquí; los roles son datos de cada negocio.

use std::collections::BTreeSet;
use std::fmt;
use std::str::FromStr;

/// Una acción que importa. En la base se guarda con su nombre (`as_str`).
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Permiso {
    Vender,
    CancelarVenta,
    Devolver,
    VerCostos,
    EditarCatalogo,
    Comprar,
    AjustarInventario,
    OperarCaja,
    GestionarClientes,
    VetarClientes,
    VerReportes,
    ConfigurarNegocio,
    AdministrarUsuarios,
}

impl Permiso {
    pub const ALL: [Permiso; 13] = [
        Self::Vender,
        Self::CancelarVenta,
        Self::Devolver,
        Self::VerCostos,
        Self::EditarCatalogo,
        Self::Comprar,
        Self::AjustarInventario,
        Self::OperarCaja,
        Self::GestionarClientes,
        Self::VetarClientes,
        Self::VerReportes,
        Self::ConfigurarNegocio,
        Self::AdministrarUsuarios,
    ];

    pub fn as_str(self) -> &'static str {
        match self {
            Self::Vender => "vender",
            Self::CancelarVenta => "cancelar_venta",
            Self::Devolver => "devolver",
            Self::VerCostos => "ver_costos",
            Self::EditarCatalogo => "editar_catalogo",
            Self::Comprar => "comprar",
            Self::AjustarInventario => "ajustar_inventario",
            Self::OperarCaja => "operar_caja",
            Self::GestionarClientes => "gestionar_clientes",
            Self::VetarClientes => "vetar_clientes",
            Self::VerReportes => "ver_reportes",
            Self::ConfigurarNegocio => "configurar_negocio",
            Self::AdministrarUsuarios => "administrar_usuarios",
        }
    }
}

impl FromStr for Permiso {
    type Err = UnknownPermiso;

    fn from_str(text: &str) -> Result<Self, Self::Err> {
        Self::ALL
            .into_iter()
            .find(|p| p.as_str() == text)
            .ok_or_else(|| UnknownPermiso(text.to_string()))
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct UnknownPermiso(pub String);

impl fmt::Display for UnknownPermiso {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "permiso desconocido: {}", self.0)
    }
}

impl std::error::Error for UnknownPermiso {}

/// Un conjunto de permisos con nombre. Es un dato del negocio: cada uno tiene los suyos.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Rol {
    pub nombre: String,
    pub permisos: BTreeSet<Permiso>,
}

impl Rol {
    pub fn can(&self, permiso: Permiso) -> bool {
        self.permisos.contains(&permiso)
    }
}

/// Los roles con que arranca cualquier negocio (diseño 07). La migración 0003 los crea igual;
/// una prueba en `db` lo verifica.
pub fn default_roles() -> Vec<Rol> {
    use Permiso::*;
    let all = BTreeSet::from(Permiso::ALL);
    let rol = |nombre: &str, permisos: BTreeSet<Permiso>| Rol {
        nombre: nombre.into(),
        permisos,
    };
    vec![
        rol("Dueño", all.clone()),
        rol(
            "Encargado",
            all.into_iter()
                .filter(|p| !matches!(p, ConfigurarNegocio | AdministrarUsuarios))
                .collect(),
        ),
        rol(
            "Cajero",
            BTreeSet::from([Vender, GestionarClientes, OperarCaja]),
        ),
    ]
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn cada_permiso_se_lee_desde_su_nombre() {
        for permiso in Permiso::ALL {
            assert_eq!(permiso.as_str().parse(), Ok(permiso));
        }
    }

    #[test]
    fn los_nombres_son_los_del_diseno() {
        let names: Vec<&str> = Permiso::ALL.iter().map(|p| p.as_str()).collect();
        assert_eq!(
            names,
            [
                "vender",
                "cancelar_venta",
                "devolver",
                "ver_costos",
                "editar_catalogo",
                "comprar",
                "ajustar_inventario",
                "operar_caja",
                "gestionar_clientes",
                "vetar_clientes",
                "ver_reportes",
                "configurar_negocio",
                "administrar_usuarios",
            ]
        );
    }

    #[test]
    fn un_nombre_desconocido_no_se_lee() {
        assert_eq!(
            "ser_jefe".parse::<Permiso>(),
            Err(UnknownPermiso("ser_jefe".into()))
        );
    }

    fn rol(nombre: &str) -> Rol {
        default_roles()
            .into_iter()
            .find(|r| r.nombre == nombre)
            .unwrap()
    }

    #[test]
    fn el_dueno_puede_todo() {
        assert!(Permiso::ALL.iter().all(|p| rol("Dueño").can(*p)));
    }

    #[test]
    fn el_encargado_puede_todo_menos_configurar_y_administrar_usuarios() {
        let encargado = rol("Encargado");
        for permiso in Permiso::ALL {
            let expected = !matches!(
                permiso,
                Permiso::ConfigurarNegocio | Permiso::AdministrarUsuarios
            );
            assert_eq!(encargado.can(permiso), expected, "{permiso:?}");
        }
    }

    #[test]
    fn el_cajero_vende_atiende_clientes_y_opera_la_caja() {
        assert_eq!(
            rol("Cajero").permisos,
            BTreeSet::from([
                Permiso::Vender,
                Permiso::GestionarClientes,
                Permiso::OperarCaja
            ])
        );
    }

    #[test]
    fn son_tres_roles_de_arranque() {
        assert_eq!(default_roles().len(), 3);
    }
}
