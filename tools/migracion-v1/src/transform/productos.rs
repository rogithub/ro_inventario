//! Los productos de la v1 y su historial de precios, a los de la v2 (paso 7d2). Como las
//! categorías: se valida con los tipos del área y se juntan todos los problemas antes de detenerse.

use std::collections::HashMap;

use chrono::{NaiveDateTime, TimeZone};
use chrono_tz::Tz;
use inventario::articulos::{ArticuloId, Nid, PrecioVenta};
use inventario::categorias::CategoriaId;
use inventario::productos::{NewProducto, ProductoFields};
use inventario::unidades_medida::{UnidadMedida, UnidadMedidaId};
use kernel::{DateTime, Utc, Uuid};

use super::normalize_nombre;
use crate::read::{PrecioV1, ProductoV1};

/// Las fechas de la v1 no llevan zona: son hora de la papelería, el único negocio de la v1.
const ZONA_V1: Tz = chrono_tz::America::Cancun;

/// Cada unidad de la v1 y la de la v2 a la que pasa. Otra detiene la migración.
const UNIDADES: [(&str, &str); 3] = [("PIEZA", "Pieza"), ("METRO", "Metro"), ("GRAMOS", "Gramo")];

/// Un producto por escribir en la v2, con el uuid y el NID que tenía en la v1.
#[derive(Debug)]
#[cfg_attr(
    not(test),
    expect(dead_code, reason = "se escriben en la v2 en el paso 7d2b")
)]
pub struct ProductoMigrado {
    pub id: ArticuloId,
    pub nid: Nid,
    /// Con el precio actual: el último del historial.
    pub producto: NewProducto,
    /// El primer precio: en la v1 `datestamp` se reescribió en 2026 y los precios son más viejos.
    pub created_at: DateTime<Utc>,
    /// La última edición o el último precio, el más reciente: en la v1 cambiar el precio no
    /// siempre movía `datestamp` (dueño, 2026-10-12).
    pub updated_at: DateTime<Utc>,
    /// Del más viejo al más nuevo, tal cual, aun los que repiten el precio anterior.
    pub historial: Vec<PrecioMigrado>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct PrecioMigrado {
    pub precio_venta: PrecioVenta,
    pub desde: DateTime<Utc>,
}

/// `categorias` dice a qué categoría de la v2 va cada una de la v1; `unidades` son las de la v2.
pub fn productos(
    v1: &[ProductoV1],
    precios: &[PrecioV1],
    categorias: &HashMap<Uuid, CategoriaId>,
    unidades: &[UnidadMedida],
) -> Result<Vec<ProductoMigrado>, Vec<String>> {
    let mut precios_by_producto: HashMap<Uuid, Vec<&PrecioV1>> = HashMap::new();
    for precio in precios {
        precios_by_producto
            .entry(precio.producto_id)
            .or_default()
            .push(precio);
    }

    let mut errors = Vec::new();
    let mut migrados = Vec::new();
    for producto in v1 {
        let precios = precios_by_producto
            .get(&producto.id)
            .map(Vec::as_slice)
            .unwrap_or_default();
        match migrate(producto, precios, categorias, unidades) {
            Ok(migrado) => migrados.push(migrado),
            Err(e) => errors.push(format!(
                "Producto NID {} «{}» de la v1: {e}",
                producto.nid, producto.nombre
            )),
        }
    }

    if errors.is_empty() {
        Ok(migrados)
    } else {
        Err(errors)
    }
}

fn migrate(
    producto: &ProductoV1,
    precios: &[&PrecioV1],
    categorias: &HashMap<Uuid, CategoriaId>,
    unidades: &[UnidadMedida],
) -> Result<ProductoMigrado, String> {
    let nid = (producto.nid > 0)
        .then_some(Nid(producto.nid))
        .ok_or("el NID debe ser mayor que cero.")?;
    // Una sola categoría: así ninguna de las migradas queda vacía (revisor de 7d1).
    let categoria_id = match producto.categoria_ids.as_slice() {
        [] => return Err("no tiene categoría.".into()),
        [only] => *categorias
            .get(only)
            .ok_or("su categoría no está entre las migradas.")?,
        _ => return Err("tiene dos categorías distintas; en la v2 solo puede tener una.".into()),
    };
    let unidad_medida_id = unidad(producto.unidad.as_deref(), unidades)?;
    let historial = historial(precios)?;
    let (Some(first), Some(last)) = (historial.first(), historial.last()) else {
        return Err("no tiene precio en su historial.".into());
    };

    let new_producto = NewProducto::new(ProductoFields {
        nombre: &normalize_nombre(&producto.nombre),
        categoria_id: Some(categoria_id),
        unidad_medida_id,
        precio_venta: &last.precio_venta.value().to_string(),
        descripcion: producto.descripcion.as_deref().unwrap_or_default(),
        marca: producto.marca.as_deref().unwrap_or_default(),
        modelo: producto.modelo.as_deref().unwrap_or_default(),
        color: producto.color.as_deref().unwrap_or_default(),
    })
    .map_err(|e| e.to_string())?;

    Ok(ProductoMigrado {
        id: ArticuloId(producto.id),
        nid,
        producto: new_producto,
        created_at: first.desde,
        updated_at: to_utc(producto.datestamp)?.max(last.desde),
        historial,
    })
}

fn unidad(v1: Option<&str>, unidades: &[UnidadMedida]) -> Result<UnidadMedidaId, String> {
    let v1 = v1.ok_or("no tiene unidad.")?;
    let (_, nombre) = UNIDADES
        .iter()
        .find(|(v1_nombre, _)| *v1_nombre == v1)
        .ok_or_else(|| format!("la unidad «{v1}» no es PIEZA, METRO ni GRAMOS."))?;
    unidades
        .iter()
        .find(|u| u.nombre == *nombre)
        .map(|u| u.id)
        .ok_or_else(|| format!("la unidad «{nombre}» no existe en la v2."))
}

/// En orden por fecha. Si dos precios tienen la misma hora, no se sabe cuál es el actual.
fn historial(precios: &[&PrecioV1]) -> Result<Vec<PrecioMigrado>, String> {
    let mut historial = precios
        .iter()
        .map(|p| {
            Ok(PrecioMigrado {
                precio_venta: PrecioVenta::new(p.precio_venta)
                    .map_err(|e| format!("el precio {} de su historial: {e}", p.precio_venta))?,
                desde: to_utc(p.fecha)?,
            })
        })
        .collect::<Result<Vec<_>, String>>()?;
    historial.sort_by_key(|p| p.desde);
    if historial.windows(2).any(|w| w[0].desde == w[1].desde) {
        return Err("tiene dos precios a la misma hora y no se sabe cuál es el actual.".into());
    }
    Ok(historial)
}

/// Una hora de la v1 (de Cancún, sin zona) como instante en UTC.
fn to_utc(fecha: NaiveDateTime) -> Result<DateTime<Utc>, String> {
    ZONA_V1
        .from_local_datetime(&fecha)
        .single()
        .map(|f| f.with_timezone(&Utc))
        .ok_or_else(|| format!("la hora {fecha} no existe o es ambigua en Cancún."))
}

#[cfg(test)]
mod tests;
