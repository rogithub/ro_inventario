use std::sync::Mutex;

use kernel::Uuid;

use super::*;

/// Productos en memoria. Solo conoce las categorías, unidades y usuarios que se le dan al crearla,
/// como Postgres solo conoce los que tiene en sus tablas.
pub struct InMemoryProductos {
    categorias: Vec<CategoriaId>,
    unidades: Vec<UnidadMedidaId>,
    usuarios: Vec<Email>,
    productos: Mutex<Vec<Producto>>,
}

impl InMemoryProductos {
    pub fn new(
        categorias: Vec<CategoriaId>,
        unidades: Vec<UnidadMedidaId>,
        usuarios: Vec<Email>,
    ) -> Self {
        Self {
            categorias,
            unidades,
            usuarios,
            productos: Mutex::default(),
        }
    }

    // Si una prueba falló con el candado tomado, los datos siguen sirviendo para las demás.
    fn lock(&self) -> std::sync::MutexGuard<'_, Vec<Producto>> {
        self.productos.lock().unwrap_or_else(|e| e.into_inner())
    }
}

impl ProductosRepo for InMemoryProductos {
    async fn list(&self) -> Result<Vec<Producto>, RepoError> {
        let mut productos = self.lock().clone();
        productos.sort_by_key(|p| (kernel::nombres::sort_key(&p.nombre), p.nid));
        Ok(productos)
    }

    async fn add(&self, new: NewProducto, by: &Email) -> Result<Producto, ProductoError> {
        if !self.usuarios.contains(by) {
            return Err(ProductoError::UsuarioNotFound);
        }
        if !self.categorias.contains(&new.categoria_id) {
            return Err(ProductoError::CategoriaNotFound);
        }
        if !self.unidades.contains(&new.unidad_medida_id) {
            return Err(ProductoError::UnidadMedidaNotFound);
        }
        let mut productos = self.lock();
        // NID next e ids predecibles en memoria; Postgres los genera.
        let next = productos.len() as i32 + 1;
        let producto = Producto {
            id: ArticuloId(Uuid::from_u128(next as u128)),
            nid: Nid(next),
            nombre: new.nombre,
            categoria_id: new.categoria_id,
            unidad_medida_id: new.unidad_medida_id,
            precio_venta: new.precio_venta,
            descripcion: new.descripcion,
            marca: new.marca,
            modelo: new.modelo,
            color: new.color,
        };
        productos.push(producto.clone());
        Ok(producto)
    }
}
