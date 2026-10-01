-- Búsqueda de artículos: sin importar acentos (unaccent; la ñ cuenta como n) y, más adelante, por
-- parecido para tolerar errores de dedo (pg_trgm). Las dos vienen con Postgres y son "confiables":
-- basta con ser dueño de la base. Decisión: docs/decisiones/2026-10-01-busqueda-de-productos.md.
CREATE EXTENSION IF NOT EXISTS unaccent;
CREATE EXTENSION IF NOT EXISTS pg_trgm;
