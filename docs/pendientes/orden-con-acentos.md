# El orden alfabético con acentos no es igual en memoria que en Postgres

**Salió de:** revisión del paso 6a (categorías), 2026-10-01 · **Cuándo:** por decidir

## Qué pasa
El contrato `la_lista_va_en_orden_alfabetico` (categorías y unidades de medida) solo usa nombres sin acentos.

- **En memoria** (`InMemoryCategorias`, `InMemoryUnidadesMedida`) se ordena con `to_lowercase()`, byte por byte: "Álbumes" y "Útiles" quedan después de "Zeta".
- **En Postgres** (`ORDER BY lower(nombre)`) manda la collation de la base. La de desarrollo es `en_US.utf8` y ordena "Álbumes, alfa, Media, Útiles, Zeta".

## Por qué importa
La prueba de contrato dice que las dos implementaciones se comportan igual, y con acentos no es así. En el catálogo los acentos son comunes ("Útiles", "Papelería"). Además, el orden en producción depende de una collation que no se ha revisado.

## Propuesta
1. Agregar al contrato una prueba con acentos, que falle con la implementación en memoria.
2. Arreglar la implementación en memoria para que ignore los acentos al ordenar.
3. Que el dueño confirme la collation de la base de producción, para dejarla fija (en la migración o en la decisión de base de datos).
