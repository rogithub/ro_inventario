//! Categorías del catálogo: planas, sin jerarquía; productos y servicios las comparten.

use std::fmt;
use std::future::Future;

use kernel::nombres::MAX_NOMBRE_CATALOGO;
use kernel::{RepoError, Uuid};

/// El id de una categoría: no se confunde con el de otra tabla.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct CategoriaId(pub Uuid);

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Categoria {
    pub id: CategoriaId,
    pub nombre: String,
}

/// Una categoría por agregar, ya validada: solo se construye con `new`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NewCategoria {
    nombre: String,
}

impl NewCategoria {
    /// Quita los espacios de las orillas del nombre; vacío o de más de 150 caracteres no se acepta.
    pub fn new(nombre: &str) -> Result<Self, CategoriaError> {
        let nombre = nombre.trim();
        if nombre.is_empty() {
            return Err(CategoriaError::EmptyNombre);
        }
        if nombre.chars().count() > MAX_NOMBRE_CATALOGO {
            return Err(CategoriaError::LongNombre);
        }
        Ok(Self {
            nombre: nombre.to_string(),
        })
    }

    pub fn nombre(&self) -> &str {
        &self.nombre
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CategoriaError {
    EmptyNombre,
    /// Más de `MAX_NOMBRE_CATALOGO` caracteres.
    LongNombre,
    /// Ya hay una con ese nombre, sin importar mayúsculas ("cuadernos" choca con "Cuadernos").
    DuplicateNombre(String),
    Repo(RepoError),
}

impl fmt::Display for CategoriaError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::EmptyNombre => write!(f, "Escribe el nombre de la categoría."),
            Self::LongNombre => write!(
                f,
                "El nombre no puede pasar de {MAX_NOMBRE_CATALOGO} caracteres."
            ),
            Self::DuplicateNombre(nombre) => write!(f, "Ya existe la categoría «{nombre}»."),
            Self::Repo(_) => write!(f, "No se pudo guardar la categoría."),
        }
    }
}

impl From<RepoError> for CategoriaError {
    fn from(error: RepoError) -> Self {
        Self::Repo(error)
    }
}

/// Dónde viven las categorías. Lo implementa `db`; para pruebas, `in_memory`.
pub trait CategoriasRepo {
    /// Todas, en orden alfabético.
    fn list(&self) -> impl Future<Output = Result<Vec<Categoria>, RepoError>> + Send;

    fn add(
        &self,
        new_categoria: NewCategoria,
    ) -> impl Future<Output = Result<Categoria, CategoriaError>> + Send;
}

#[cfg(any(test, feature = "test-support"))]
pub mod in_memory {
    use std::sync::Mutex;

    use super::*;

    #[derive(Default)]
    pub struct InMemoryCategorias {
        categorias: Mutex<Vec<Categoria>>,
    }

    impl CategoriasRepo for InMemoryCategorias {
        async fn list(&self) -> Result<Vec<Categoria>, RepoError> {
            let mut categorias = self.lock().clone();
            categorias.sort_by_key(|c| kernel::nombres::sort_key(&c.nombre));
            Ok(categorias)
        }

        async fn add(&self, new_categoria: NewCategoria) -> Result<Categoria, CategoriaError> {
            let mut categorias = self.lock();
            let key = new_categoria.nombre().to_lowercase();
            if categorias.iter().any(|c| c.nombre.to_lowercase() == key) {
                return Err(CategoriaError::DuplicateNombre(new_categoria.nombre));
            }
            // Ids predecibles en memoria; Postgres los genera con gen_random_uuid().
            let id = CategoriaId(Uuid::from_u128(categorias.len() as u128 + 1));
            let categoria = Categoria {
                id,
                nombre: new_categoria.nombre,
            };
            categorias.push(categoria.clone());
            Ok(categoria)
        }
    }

    impl InMemoryCategorias {
        // Si una prueba falló con el candado tomado, los datos siguen sirviendo para las demás.
        fn lock(&self) -> std::sync::MutexGuard<'_, Vec<Categoria>> {
            self.categorias.lock().unwrap_or_else(|e| e.into_inner())
        }
    }
}

/// Lo que toda implementación de `CategoriasRepo` debe cumplir. Corre contra la de memoria
/// (aquí) y contra la de Postgres (en `db`).
#[cfg(any(test, feature = "test-support"))]
#[allow(clippy::unwrap_used)] // código de pruebas
pub mod contract {
    use super::*;

    pub async fn agregada_aparece_en_la_lista(repo: &impl CategoriasRepo) {
        let added = repo
            .add(NewCategoria::new("Cuadernos").unwrap())
            .await
            .unwrap();
        assert_eq!(added.nombre, "Cuadernos");
        assert!(repo.list().await.unwrap().contains(&added));
    }

    pub async fn agregada_trae_su_id_y_la_lista_lo_conserva(repo: &impl CategoriasRepo) {
        let plumas = repo
            .add(NewCategoria::new("Plumas").unwrap())
            .await
            .unwrap();
        let hojas = repo.add(NewCategoria::new("Hojas").unwrap()).await.unwrap();

        assert_ne!(plumas.id, hojas.id);
        let list = repo.list().await.unwrap();
        for added in [plumas, hojas] {
            let listed = list.iter().find(|c| c.nombre == added.nombre).unwrap();
            assert_eq!(listed.id, added.id);
        }
    }

    pub async fn nombre_repetido_se_rechaza_sin_importar_mayusculas(repo: &impl CategoriasRepo) {
        repo.add(NewCategoria::new("Plumas").unwrap())
            .await
            .unwrap();
        let before = repo.list().await.unwrap();

        let result = repo.add(NewCategoria::new(" pLUMAS ").unwrap()).await;

        assert_eq!(
            result,
            Err(CategoriaError::DuplicateNombre("pLUMAS".into()))
        );
        assert_eq!(repo.list().await.unwrap(), before, "no debe agregar nada");
    }

    pub async fn la_lista_va_en_orden_alfabetico(repo: &impl CategoriasRepo) {
        for nombre in ["Zeta", "alfa", "Media"] {
            repo.add(NewCategoria::new(nombre).unwrap()).await.unwrap();
        }

        let nombres: Vec<String> = repo
            .list()
            .await
            .unwrap()
            .into_iter()
            .map(|c| c.nombre)
            .collect();
        assert_eq!(nombres, ["alfa", "Media", "Zeta"]);
    }

    /// Como Postgres con la collation `en_US.utf8` (desarrollo, CI y producción): sin importar
    /// mayúsculas ni acentos, la ñ como n, y sin contar espacios ni signos.
    pub async fn la_lista_ordena_como_postgres_acentos_espacios_y_signos(
        repo: &impl CategoriasRepo,
    ) {
        let esperado = [
            "albumes", "Álbumes", "alfa", "cob", "co-op", "coop", "Hojas a", "Hoja z", "ñandú",
            "nube", "Útiles",
        ];
        for nombre in [
            "Útiles", "Hoja z", "coop", "ñandú", "alfa", "co-op", "Álbumes", "nube", "Hojas a",
            "cob", "albumes",
        ] {
            repo.add(NewCategoria::new(nombre).unwrap()).await.unwrap();
        }

        let nombres: Vec<String> = repo
            .list()
            .await
            .unwrap()
            .into_iter()
            .map(|c| c.nombre)
            .filter(|n| esperado.contains(&n.as_str()))
            .collect();
        assert_eq!(nombres, esperado);
    }
}

#[cfg(test)]
mod tests {
    use super::in_memory::InMemoryCategorias;
    use super::*;

    #[test]
    fn el_nombre_se_guarda_sin_espacios_en_las_orillas() {
        let new_categoria = NewCategoria::new("  Hojas de color  ").unwrap();
        assert_eq!(new_categoria.nombre(), "Hojas de color");
    }

    #[test]
    fn un_nombre_vacio_o_solo_espacios_no_se_acepta() {
        assert_eq!(NewCategoria::new(""), Err(CategoriaError::EmptyNombre));
        assert_eq!(NewCategoria::new("   "), Err(CategoriaError::EmptyNombre));
    }

    #[test]
    fn un_nombre_de_150_caracteres_se_acepta_aunque_lleve_enies() {
        let nombre = "ñ".repeat(150);
        assert_eq!(NewCategoria::new(&nombre).unwrap().nombre(), nombre);
    }

    #[test]
    fn un_nombre_de_151_caracteres_no_se_acepta() {
        assert_eq!(
            NewCategoria::new(&"a".repeat(151)),
            Err(CategoriaError::LongNombre)
        );
    }

    #[test]
    fn los_espacios_de_las_orillas_no_cuentan_para_el_largo() {
        let nombre = format!("  {}  ", "a".repeat(150));
        assert!(NewCategoria::new(&nombre).is_ok());
    }

    #[tokio::test]
    async fn en_memoria_cumple_el_contrato_agregada_aparece_en_la_lista() {
        contract::agregada_aparece_en_la_lista(&InMemoryCategorias::default()).await;
    }

    #[tokio::test]
    async fn en_memoria_cumple_el_contrato_nombre_repetido() {
        contract::nombre_repetido_se_rechaza_sin_importar_mayusculas(
            &InMemoryCategorias::default(),
        )
        .await;
    }

    #[tokio::test]
    async fn en_memoria_cumple_el_contrato_orden_alfabetico() {
        contract::la_lista_va_en_orden_alfabetico(&InMemoryCategorias::default()).await;
    }

    #[tokio::test]
    async fn en_memoria_cumple_el_contrato_orden_como_postgres() {
        contract::la_lista_ordena_como_postgres_acentos_espacios_y_signos(
            &InMemoryCategorias::default(),
        )
        .await;
    }

    #[tokio::test]
    async fn en_memoria_cumple_el_contrato_trae_su_id() {
        contract::agregada_trae_su_id_y_la_lista_lo_conserva(&InMemoryCategorias::default()).await;
    }
}
