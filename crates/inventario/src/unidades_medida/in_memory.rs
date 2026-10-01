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

    async fn add(&self, new_unidad: NewUnidadMedida) -> Result<UnidadMedida, UnidadMedidaError> {
        let mut unidades = self.lock();
        let key = new_unidad.nombre().to_lowercase();
        if unidades.iter().any(|u| u.nombre.to_lowercase() == key) {
            return Err(UnidadMedidaError::DuplicateNombre(new_unidad.nombre));
        }
        // Ids predecibles en memoria; Postgres los genera con gen_random_uuid().
        let id = UnidadMedidaId(Uuid::from_u128(unidades.len() as u128 + 1));
        let unidad = UnidadMedida {
            id,
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
