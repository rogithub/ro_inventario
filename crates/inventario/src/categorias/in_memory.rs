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

    async fn rename(
        &self,
        id: CategoriaId,
        new_categoria: NewCategoria,
    ) -> Result<Categoria, CategoriaError> {
        let mut categorias = self.lock();
        // Como Postgres: si el id no es de ninguna, no importa si el nombre está ocupado.
        if !categorias.iter().any(|c| c.id == id) {
            return Err(CategoriaError::NotFound);
        }
        let key = new_categoria.nombre().to_lowercase();
        if categorias
            .iter()
            .any(|c| c.id != id && c.nombre.to_lowercase() == key)
        {
            return Err(CategoriaError::DuplicateNombre(new_categoria.nombre));
        }
        let categoria = categorias
            .iter_mut()
            .find(|c| c.id == id)
            .ok_or(CategoriaError::NotFound)?;
        categoria.nombre = new_categoria.nombre;
        Ok(categoria.clone())
    }
}

impl InMemoryCategorias {
    // Si una prueba falló con el candado tomado, los datos siguen sirviendo para las demás.
    fn lock(&self) -> std::sync::MutexGuard<'_, Vec<Categoria>> {
        self.categorias.lock().unwrap_or_else(|e| e.into_inner())
    }
}
