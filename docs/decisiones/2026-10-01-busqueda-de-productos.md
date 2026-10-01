# La búsqueda de productos la hace Postgres, con `unaccent` y `pg_trgm`

**Fecha:** 2026-10-01 · **Estado:** vigente

## Contexto
La pantalla de productos (paso 7b) busca por nombre o NID mientras se escribe, y caja (diseño 04) va a necesitar la misma búsqueda con stock y presentaciones. En la v1, 155 de 2,336 nombres llevan acento y 81 llevan ñ, y quien busca no siempre los escribe. La v1 busca en Postgres con `unaccent`, full text en español y `similarity()` de `pg_trgm`, con `LIMIT 100`, y le ha funcionado con este mismo catálogo.

## Decisión
- **Busca Postgres:** cada palabra escrita debe aparecer en `unaccent(lower(nombre))`, en cualquier orden; si lo escrito es un NID, ese producto también coincide y va primero. Regresa hasta 100 (`MAX_RESULTS`) y cuántos coinciden en total.
- **Acentos y ñ no cuentan:** «lapiz» encuentra «LÁPIZ»; «pina» y «piña» encuentran «PIÑA» (decisión del dueño).
- **Cuentan hasta 25 palabras** (`MAX_WORDS`); las demás se ignoran, para que una búsqueda no crezca sin límite (revisor de 7b1, dueño).
- **Lo escrito se busca tal cual:** `%`, `_` y `\` no son comodines.
- **Extensiones `unaccent` y `pg_trgm`** desde la migración 0009. `pg_trgm` todavía no se usa; se agrega porque la tolerancia a errores de dedo va a hacer falta (dueño).
- La versión en memoria del repositorio se aproxima con `kernel::nombres::fold_for_search` (acentos del español); el contrato de los dos cubre los casos del español, y la verdad es Postgres.

## Descartado
- **Cargar todos los productos y filtrar en Rust:** fácil de probar como función pura, pero trae el catálogo entero en cada tecla, crece con él y caja tendría que rehacerlo en SQL.
- **Full text (`search_vector`) y `similarity()` desde ya:** con unos 2,300 productos no hacen falta; se agregan cuando un caso real los pida (probablemente en caja).

## Consecuencias
- **La base de cada negocio debe ser propiedad del usuario de la aplicación** (`CREATE DATABASE … OWNER ro_inventario`, como en desarrollo): las dos extensiones son "confiables" y el dueño de la base las puede crear, pero se instalan por base, no por servidor. Si no, el arranque falla al migrar. El servidor del cluster (`pgvector/pgvector:0.8.2-pg18-trixie`) ya trae las dos.
- La regla de búsqueda vive en SQL, no en el área; el área solo limpia lo escrito (`SearchQuery`).
- Sin índice por ahora: `unaccent` no es `IMMUTABLE` y no se puede indexar directo. Con este tamaño de catálogo no hace falta.
