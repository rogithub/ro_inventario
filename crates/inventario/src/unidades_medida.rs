//! Unidades en que se cuenta el stock y se cobra: Pieza, Metro, Hoja…

use std::fmt;
use std::future::Future;

use kernel::RepoError;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct UnidadMedida {
    pub nombre: String,
    /// Si se puede vender en fracciones (1.5 metros) o solo en enteros (piezas).
    pub allows_fraction: bool,
}

/// Una unidad por agregar, ya validada: solo se construye con `new`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NuevaUnidadMedida {
    nombre: String,
    allows_fraction: bool,
}

impl NuevaUnidadMedida {
    /// Quita los espacios de las orillas del nombre; vacío no se acepta.
    pub fn new(nombre: &str, allows_fraction: bool) -> Result<Self, UnidadMedidaError> {
        let nombre = nombre.trim();
        if nombre.is_empty() {
            return Err(UnidadMedidaError::NombreVacio);
        }
        Ok(Self {
            nombre: nombre.to_string(),
            allows_fraction,
        })
    }

    pub fn nombre(&self) -> &str {
        &self.nombre
    }

    pub fn allows_fraction(&self) -> bool {
        self.allows_fraction
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum UnidadMedidaError {
    NombreVacio,
    /// Ya hay una con ese nombre, sin importar mayúsculas ("hoja" choca con "Hoja").
    NombreRepetido(String),
    Repo(RepoError),
}

impl fmt::Display for UnidadMedidaError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::NombreVacio => write!(f, "Escribe el nombre de la unidad."),
            Self::NombreRepetido(nombre) => write!(f, "Ya existe la unidad «{nombre}»."),
            Self::Repo(_) => write!(f, "No se pudo guardar la unidad."),
        }
    }
}

impl From<RepoError> for UnidadMedidaError {
    fn from(error: RepoError) -> Self {
        Self::Repo(error)
    }
}

/// Dónde viven las unidades. Lo implementa `db`; para pruebas, `en_memoria`.
pub trait UnidadesMedidaRepo {
    /// Todas, en orden alfabético.
    fn list(&self) -> impl Future<Output = Result<Vec<UnidadMedida>, RepoError>> + Send;

    fn add(
        &self,
        nueva: NuevaUnidadMedida,
    ) -> impl Future<Output = Result<UnidadMedida, UnidadMedidaError>> + Send;
}

#[cfg(any(test, feature = "pruebas"))]
pub mod en_memoria {
    use std::sync::Mutex;

    use super::*;

    #[derive(Default)]
    pub struct UnidadesMedidaEnMemoria {
        unidades: Mutex<Vec<UnidadMedida>>,
    }

    impl UnidadesMedidaRepo for UnidadesMedidaEnMemoria {
        async fn list(&self) -> Result<Vec<UnidadMedida>, RepoError> {
            let mut unidades = self.lock().clone();
            unidades.sort_by_key(|u| u.nombre.to_lowercase());
            Ok(unidades)
        }

        async fn add(&self, nueva: NuevaUnidadMedida) -> Result<UnidadMedida, UnidadMedidaError> {
            let mut unidades = self.lock();
            let clave = nueva.nombre().to_lowercase();
            if unidades.iter().any(|u| u.nombre.to_lowercase() == clave) {
                return Err(UnidadMedidaError::NombreRepetido(nueva.nombre));
            }
            let unidad = UnidadMedida {
                nombre: nueva.nombre,
                allows_fraction: nueva.allows_fraction,
            };
            unidades.push(unidad.clone());
            Ok(unidad)
        }
    }

    impl UnidadesMedidaEnMemoria {
        // Si una prueba falló con el candado tomado, los datos siguen sirviendo para las demás.
        fn lock(&self) -> std::sync::MutexGuard<'_, Vec<UnidadMedida>> {
            self.unidades.lock().unwrap_or_else(|e| e.into_inner())
        }
    }
}

/// Lo que toda implementación de `UnidadesMedidaRepo` debe cumplir. Corre contra la de memoria
/// (aquí) y contra la de Postgres (en `db`). Cada función recibe un repo que puede traer
/// unidades de antes (las base de la migración); usa nombres que no existen.
#[cfg(any(test, feature = "pruebas"))]
#[allow(clippy::unwrap_used)] // código de pruebas
pub mod contrato {
    use super::*;

    pub async fn agregada_aparece_en_la_lista(repo: &impl UnidadesMedidaRepo) {
        let agregada = repo
            .add(NuevaUnidadMedida::new("Cuartilla", true).unwrap())
            .await
            .unwrap();
        assert_eq!(
            agregada,
            UnidadMedida {
                nombre: "Cuartilla".into(),
                allows_fraction: true
            }
        );
        assert!(repo.list().await.unwrap().contains(&agregada));
    }

    pub async fn nombre_repetido_se_rechaza_sin_importar_mayusculas(
        repo: &impl UnidadesMedidaRepo,
    ) {
        repo.add(NuevaUnidadMedida::new("Hoja", false).unwrap())
            .await
            .unwrap();
        let antes = repo.list().await.unwrap();

        let resultado = repo
            .add(NuevaUnidadMedida::new(" hOJA ", true).unwrap())
            .await;

        assert_eq!(
            resultado,
            Err(UnidadMedidaError::NombreRepetido("hOJA".into()))
        );
        assert_eq!(repo.list().await.unwrap(), antes, "no debe agregar nada");
    }

    pub async fn la_lista_va_en_orden_alfabetico(repo: &impl UnidadesMedidaRepo) {
        for nombre in ["Zeta", "Alfa", "Media"] {
            repo.add(NuevaUnidadMedida::new(nombre, false).unwrap())
                .await
                .unwrap();
        }

        let nombres: Vec<String> = repo
            .list()
            .await
            .unwrap()
            .into_iter()
            .map(|u| u.nombre)
            .collect();
        let mut ordenados = nombres.clone();
        ordenados.sort_by_key(|n| n.to_lowercase());
        assert_eq!(nombres, ordenados);
    }
}

#[cfg(test)]
mod tests {
    use super::en_memoria::UnidadesMedidaEnMemoria;
    use super::*;

    #[test]
    fn el_nombre_se_guarda_sin_espacios_en_las_orillas() {
        let nueva = NuevaUnidadMedida::new("  Hoja carta  ", false).unwrap();
        assert_eq!(nueva.nombre(), "Hoja carta");
        assert!(!nueva.allows_fraction());
    }

    #[test]
    fn un_nombre_vacio_o_solo_espacios_no_se_acepta() {
        assert_eq!(
            NuevaUnidadMedida::new("", true),
            Err(UnidadMedidaError::NombreVacio)
        );
        assert_eq!(
            NuevaUnidadMedida::new("   ", true),
            Err(UnidadMedidaError::NombreVacio)
        );
    }

    #[tokio::test]
    async fn en_memoria_cumple_el_contrato_agregada_aparece_en_la_lista() {
        contrato::agregada_aparece_en_la_lista(&UnidadesMedidaEnMemoria::default()).await;
    }

    #[tokio::test]
    async fn en_memoria_cumple_el_contrato_nombre_repetido() {
        contrato::nombre_repetido_se_rechaza_sin_importar_mayusculas(
            &UnidadesMedidaEnMemoria::default(),
        )
        .await;
    }

    #[tokio::test]
    async fn en_memoria_cumple_el_contrato_orden_alfabetico() {
        contrato::la_lista_va_en_orden_alfabetico(&UnidadesMedidaEnMemoria::default()).await;
    }
}
