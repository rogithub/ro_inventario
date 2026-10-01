//! Unidades en que se cuenta el stock y se cobra: Pieza, Metro, Hoja…

use std::fmt;
use std::future::Future;

use kernel::RepoError;
use kernel::nombres::MAX_NOMBRE_CATALOGO;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct UnidadMedida {
    pub nombre: String,
    /// Si se puede vender en fracciones (1.5 metros) o solo en enteros (piezas).
    pub allows_fraction: bool,
}

/// Una unidad por agregar, ya validada: solo se construye con `new`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NewUnidadMedida {
    nombre: String,
    allows_fraction: bool,
}

impl NewUnidadMedida {
    /// Quita los espacios de las orillas del nombre; vacío o de más de 150 caracteres no se acepta.
    pub fn new(nombre: &str, allows_fraction: bool) -> Result<Self, UnidadMedidaError> {
        let nombre = nombre.trim();
        if nombre.is_empty() {
            return Err(UnidadMedidaError::EmptyNombre);
        }
        if nombre.chars().count() > MAX_NOMBRE_CATALOGO {
            return Err(UnidadMedidaError::LongNombre);
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
    EmptyNombre,
    /// Más de `MAX_NOMBRE_CATALOGO` caracteres.
    LongNombre,
    /// Ya hay una con ese nombre, sin importar mayúsculas ("hoja" choca con "Hoja").
    DuplicateNombre(String),
    Repo(RepoError),
}

impl fmt::Display for UnidadMedidaError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::EmptyNombre => write!(f, "Escribe el nombre de la unidad."),
            Self::LongNombre => write!(
                f,
                "El nombre no puede pasar de {MAX_NOMBRE_CATALOGO} caracteres."
            ),
            Self::DuplicateNombre(nombre) => write!(f, "Ya existe la unidad «{nombre}»."),
            Self::Repo(_) => write!(f, "No se pudo guardar la unidad."),
        }
    }
}

impl From<RepoError> for UnidadMedidaError {
    fn from(error: RepoError) -> Self {
        Self::Repo(error)
    }
}

/// Dónde viven las unidades. Lo implementa `db`; para pruebas, `in_memory`.
pub trait UnidadesMedidaRepo {
    /// Todas, en orden alfabético.
    fn list(&self) -> impl Future<Output = Result<Vec<UnidadMedida>, RepoError>> + Send;

    fn add(
        &self,
        new_unidad: NewUnidadMedida,
    ) -> impl Future<Output = Result<UnidadMedida, UnidadMedidaError>> + Send;
}

#[cfg(any(test, feature = "test-support"))]
pub mod in_memory {
    use std::sync::Mutex;

    use super::*;

    #[derive(Default)]
    pub struct InMemoryUnidadesMedida {
        unidades: Mutex<Vec<UnidadMedida>>,
    }

    impl UnidadesMedidaRepo for InMemoryUnidadesMedida {
        async fn list(&self) -> Result<Vec<UnidadMedida>, RepoError> {
            let mut unidades = self.lock().clone();
            unidades.sort_by_key(|u| kernel::nombres::sort_key(&u.nombre));
            Ok(unidades)
        }

        async fn add(
            &self,
            new_unidad: NewUnidadMedida,
        ) -> Result<UnidadMedida, UnidadMedidaError> {
            let mut unidades = self.lock();
            let key = new_unidad.nombre().to_lowercase();
            if unidades.iter().any(|u| u.nombre.to_lowercase() == key) {
                return Err(UnidadMedidaError::DuplicateNombre(new_unidad.nombre));
            }
            let unidad = UnidadMedida {
                nombre: new_unidad.nombre,
                allows_fraction: new_unidad.allows_fraction,
            };
            unidades.push(unidad.clone());
            Ok(unidad)
        }
    }

    impl InMemoryUnidadesMedida {
        // Si una prueba falló con el candado tomado, los datos siguen sirviendo para las demás.
        fn lock(&self) -> std::sync::MutexGuard<'_, Vec<UnidadMedida>> {
            self.unidades.lock().unwrap_or_else(|e| e.into_inner())
        }
    }
}

/// Lo que toda implementación de `UnidadesMedidaRepo` debe cumplir. Corre contra la de memoria
/// (aquí) y contra la de Postgres (en `db`). Cada función recibe un repo que puede traer
/// unidades de antes (las base de la migración); usa nombres que no existen.
#[cfg(any(test, feature = "test-support"))]
#[allow(clippy::unwrap_used)] // código de pruebas
pub mod contract {
    use super::*;

    pub async fn agregada_aparece_en_la_lista(repo: &impl UnidadesMedidaRepo) {
        let added = repo
            .add(NewUnidadMedida::new("Cuartilla", true).unwrap())
            .await
            .unwrap();
        assert_eq!(
            added,
            UnidadMedida {
                nombre: "Cuartilla".into(),
                allows_fraction: true
            }
        );
        assert!(repo.list().await.unwrap().contains(&added));
    }

    pub async fn nombre_repetido_se_rechaza_sin_importar_mayusculas(
        repo: &impl UnidadesMedidaRepo,
    ) {
        repo.add(NewUnidadMedida::new("Hoja", false).unwrap())
            .await
            .unwrap();
        let before = repo.list().await.unwrap();

        let result = repo
            .add(NewUnidadMedida::new(" hOJA ", true).unwrap())
            .await;

        assert_eq!(
            result,
            Err(UnidadMedidaError::DuplicateNombre("hOJA".into()))
        );
        assert_eq!(repo.list().await.unwrap(), before, "no debe agregar nada");
    }

    pub async fn la_lista_va_en_orden_alfabetico(repo: &impl UnidadesMedidaRepo) {
        for nombre in ["Zeta", "Alfa", "Media"] {
            repo.add(NewUnidadMedida::new(nombre, false).unwrap())
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
        let mut sorted = nombres.clone();
        sorted.sort_by_key(|n| n.to_lowercase());
        assert_eq!(nombres, sorted);
    }

    /// Como Postgres con la collation `en_US.utf8` (desarrollo, CI y producción): sin importar
    /// mayúsculas ni acentos, la ñ como n, y sin contar espacios ni signos.
    pub async fn la_lista_ordena_como_postgres_acentos_espacios_y_signos(
        repo: &impl UnidadesMedidaRepo,
    ) {
        let esperado = [
            "albumes", "Álbumes", "alfa", "cob", "co-op", "coop", "Hojas a", "Hoja z", "ñandú",
            "nube", "Útiles",
        ];
        for nombre in [
            "Útiles", "Hoja z", "coop", "ñandú", "alfa", "co-op", "Álbumes", "nube", "Hojas a",
            "cob", "albumes",
        ] {
            repo.add(NewUnidadMedida::new(nombre, false).unwrap())
                .await
                .unwrap();
        }

        let nombres: Vec<String> = repo
            .list()
            .await
            .unwrap()
            .into_iter()
            .map(|u| u.nombre)
            .filter(|n| esperado.contains(&n.as_str()))
            .collect();
        assert_eq!(nombres, esperado);
    }
}

#[cfg(test)]
mod tests {
    use super::in_memory::InMemoryUnidadesMedida;
    use super::*;

    #[test]
    fn el_nombre_se_guarda_sin_espacios_en_las_orillas() {
        let new_unidad = NewUnidadMedida::new("  Hoja carta  ", false).unwrap();
        assert_eq!(new_unidad.nombre(), "Hoja carta");
        assert!(!new_unidad.allows_fraction());
    }

    #[test]
    fn un_nombre_vacio_o_solo_espacios_no_se_acepta() {
        assert_eq!(
            NewUnidadMedida::new("", true),
            Err(UnidadMedidaError::EmptyNombre)
        );
        assert_eq!(
            NewUnidadMedida::new("   ", true),
            Err(UnidadMedidaError::EmptyNombre)
        );
    }

    #[test]
    fn un_nombre_de_150_caracteres_se_acepta_aunque_lleve_enies() {
        let nombre = "ñ".repeat(150);
        assert_eq!(
            NewUnidadMedida::new(&nombre, false).unwrap().nombre(),
            nombre
        );
    }

    #[test]
    fn un_nombre_de_151_caracteres_no_se_acepta() {
        assert_eq!(
            NewUnidadMedida::new(&"a".repeat(151), false),
            Err(UnidadMedidaError::LongNombre)
        );
    }

    #[test]
    fn los_espacios_de_las_orillas_no_cuentan_para_el_largo() {
        let nombre = format!("  {}  ", "a".repeat(150));
        assert!(NewUnidadMedida::new(&nombre, false).is_ok());
    }

    #[tokio::test]
    async fn en_memoria_cumple_el_contrato_agregada_aparece_en_la_lista() {
        contract::agregada_aparece_en_la_lista(&InMemoryUnidadesMedida::default()).await;
    }

    #[tokio::test]
    async fn en_memoria_cumple_el_contrato_nombre_repetido() {
        contract::nombre_repetido_se_rechaza_sin_importar_mayusculas(
            &InMemoryUnidadesMedida::default(),
        )
        .await;
    }

    #[tokio::test]
    async fn en_memoria_cumple_el_contrato_orden_alfabetico() {
        contract::la_lista_va_en_orden_alfabetico(&InMemoryUnidadesMedida::default()).await;
    }

    #[tokio::test]
    async fn en_memoria_cumple_el_contrato_orden_como_postgres() {
        contract::la_lista_ordena_como_postgres_acentos_espacios_y_signos(
            &InMemoryUnidadesMedida::default(),
        )
        .await;
    }
}
