# La base de producción debe ser del usuario de la aplicación

**Salió de:** revisor de la parte 7b1, 2026-10-01 · **Cuándo:** al crear la base de producción de la v2, antes del primer deploy

## Qué pasa
La migración 0009 crea las extensiones `unaccent` y `pg_trgm`. Las dos son "confiables": las puede crear el dueño de la base sin ser superusuario, pero se instalan por base, no por servidor (que la v1 las tenga no basta). En desarrollo la base se crea con `OWNER ro_inventario` (`herramientas/dev-db.sh`); para producción no hay nada escrito todavía.

## Por qué importa
Si la base del cluster se crea con otro dueño, la aplicación no puede crear las extensiones y el primer arranque falla al migrar.

## Propuesta
Al crear la base en el Postgres del cluster: `CREATE DATABASE … OWNER <usuario de la aplicación>`, igual que en desarrollo. Decisión: [`2026-10-01-busqueda-de-productos.md`](../decisiones/2026-10-01-busqueda-de-productos.md).
