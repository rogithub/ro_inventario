//! De los datos de la v1 a los de la v2, sin base de datos: aquí viven las reglas de la migración.
//! Se valida con los tipos del área, como lo que se captura en pantalla, y se juntan todos los
//! problemas antes de detenerse, para corregirlos de una vez.

use std::collections::{BTreeMap, HashMap};

use inventario::categorias::{CategoriaId, NewCategoria};
use kernel::Uuid;

use crate::read::CategoriaV1;

/// Un nombre de la v1 como se guarda en la v2: sin espacios en las orillas y con los espacios
/// repetidos juntados en uno. Las mayúsculas se quedan (decisión del dueño, 2026-10-01).
pub fn normalize_nombre(nombre: &str) -> String {
    nombre.split_whitespace().collect::<Vec<_>>().join(" ")
}

/// Las categorías por escribir en la v2.
#[derive(Debug)]
pub struct Categorias {
    /// Cada una con el uuid que tenía en la v1, en orden por nombre.
    pub to_insert: Vec<(CategoriaId, NewCategoria)>,
    /// A qué categoría de la v2 va cada una de la v1: las fusionadas apuntan a la que se quedó.
    pub by_v1_id: HashMap<Uuid, CategoriaId>,
}

/// En la v1 hay categorías que solo difieren en un espacio al final ("CINTA " y "CINTA"); en la
/// v2 el nombre no se repite sin importar mayúsculas, así que se fusionan en una. Se conserva el
/// uuid de la que ya venía limpia. Si no hay exactamente una limpia, no se adivina: es un error.
pub fn categorias(v1: &[CategoriaV1]) -> Result<Categorias, Vec<String>> {
    let mut groups: BTreeMap<String, Vec<&CategoriaV1>> = BTreeMap::new();
    for categoria in v1 {
        let key = normalize_nombre(&categoria.nombre).to_lowercase();
        groups.entry(key).or_default().push(categoria);
    }

    let mut errors = Vec::new();
    let mut categorias = Categorias {
        to_insert: Vec::new(),
        by_v1_id: HashMap::new(),
    };
    for group in groups.values() {
        let Some(kept) = survivor(group) else {
            let names: Vec<String> = group.iter().map(|c| format!("«{}»", c.nombre)).collect();
            errors.push(format!(
                "Las categorías {} de la v1 son la misma y no se sabe cuál conservar.",
                names.join(", ")
            ));
            continue;
        };
        match NewCategoria::new(&normalize_nombre(&kept.nombre)) {
            Ok(new_categoria) => {
                let id = CategoriaId(kept.id);
                categorias.to_insert.push((id, new_categoria));
                for categoria in group {
                    categorias.by_v1_id.insert(categoria.id, id);
                }
            }
            Err(e) => errors.push(format!(
                "Categoría {} «{}» de la v1: {e}",
                kept.id, kept.nombre
            )),
        }
    }

    if errors.is_empty() {
        Ok(categorias)
    } else {
        Err(errors)
    }
}

/// La que se conserva de un grupo de categorías que en la v2 son la misma.
fn survivor<'a>(group: &[&'a CategoriaV1]) -> Option<&'a CategoriaV1> {
    if let [only] = group {
        return Some(only);
    }
    let mut clean = group
        .iter()
        .filter(|c| c.nombre == normalize_nombre(&c.nombre));
    match (clean.next(), clean.next()) {
        (Some(clean), None) => Some(clean),
        _ => None,
    }
}

#[cfg(test)]
mod tests;
