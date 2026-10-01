//! Categorías del catálogo: planas, sin jerarquía; productos y servicios las comparten.

use std::fmt;
use std::future::Future;

use kernel::RepoError;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Categoria {
    pub nombre: String,
}

/// Una categoría por agregar, ya validada: solo se construye con `new`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NewCategoria {
    nombre: String,
}

impl NewCategoria {
    /// Quita los espacios de las orillas del nombre; vacío no se acepta.
    pub fn new(nombre: &str) -> Result<Self, CategoriaError> {
        let nombre = nombre.trim();
        if nombre.is_empty() {
            return Err(CategoriaError::EmptyNombre);
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
    /// Ya hay una con ese nombre, sin importar mayúsculas ("cuadernos" choca con "Cuadernos").
    DuplicateNombre(String),
    Repo(RepoError),
}

impl fmt::Display for CategoriaError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::EmptyNombre => write!(f, "Escribe el nombre de la categoría."),
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
            categorias.sort_by_key(|c| c.nombre.to_lowercase());
            Ok(categorias)
        }

        async fn add(&self, new_categoria: NewCategoria) -> Result<Categoria, CategoriaError> {
            let mut categorias = self.lock();
            let key = new_categoria.nombre().to_lowercase();
            if categorias.iter().any(|c| c.nombre.to_lowercase() == key) {
                return Err(CategoriaError::DuplicateNombre(new_categoria.nombre));
            }
            let categoria = Categoria {
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
        assert_eq!(
            added,
            Categoria {
                nombre: "Cuadernos".into()
            }
        );
        assert!(repo.list().await.unwrap().contains(&added));
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
}
