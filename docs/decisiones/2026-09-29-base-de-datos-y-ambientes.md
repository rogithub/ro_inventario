# La base de datos: migraciones, vistas regenerables, semillas y ambientes

**Fecha:** 2026-09-29 · **Estado:** vigente

## Contexto
En la versión uno la base se manejaba con cuatro archivos: el esquema completo (`postgresql_inventario.sql`), los datos iniciales (`inserts.sql`), las vistas idempotentes (`reportes.sql`) y los cambios incrementales (`updates.sql`). La idea funcionó, pero con dos dolores:
- El esquema completo y `updates.sql` se mantenían a mano por separado y se desalineaban; hizo falta un script (`verificar_schema.sh`) para detectarlo.
- En producción el dueño corría `updates.sql` y `reportes.sql` a mano, y el orden importaba: un deploy casi rompe la búsqueda por aplicar las vistas en el orden equivocado.

Lo que sí funcionó y se conserva: una base de desarrollo donde la IA puede escribir, producción intocable para la IA, y un usuario de pruebas que solo existe en desarrollo.

## Decisión

### Qué hay en el repo
| Carpeta o archivo | Qué es | Quién lo aplica |
|---|---|---|
| `migrations/NNNN_descripcion.sql` | Cambios de esquema, numerados. Cada uno se aplica una sola vez y la base registra cuáles tiene. Tablas y columnas documentadas con `COMMENT ON`. | La aplicación, al arrancar |
| `migrations/` (datos base) | Lo que toda instalación necesita: unidades de medida, catálogo de formas de pago del SAT, roles y permisos. Van como migraciones. | La aplicación, al arrancar |
| `db/vistas/` | Vistas y funciones, idempotentes (`CREATE OR REPLACE` o borrar y crear). Se regeneran completas en cada arranque. | La aplicación, al arrancar |
| `db/semilla-dev.sql` | Datos de desarrollo: productos de ejemplo y el usuario de pruebas de la IA. | Solo el script de desarrollo, **nunca** la aplicación |
| `docs/esquema.sql` | Esquema de referencia para leer, generado de la base con un script; CI revisa que esté al día. | Nadie: se genera |

- **Al arrancar, la aplicación aplica las migraciones pendientes y después regenera las vistas.** Cada migración corre en su propia transacción, y las vistas en otra: si un paso falla, ese paso no deja nada a medias y la instancia no arranca (las migraciones anteriores a la que falló sí quedan aplicadas; por eso se prueban antes en CI). Deployar es suficiente: ya no hay SQL manual en producción. El dueño sigue decidiendo cuándo pasa, porque él hace el deploy.
- **Una migración que ya llegó a `main` no se edita.** Un error se corrige con otra migración (no hay migraciones "de reversa"). **Excepción mientras la v2 no tenga producción:** una migración que solo se aplicó en bases de desarrollo se puede editar si el dueño lo acuerda; cada quien recrea su base con `herramientas/dev-db.sh` (así se editó la 0004 el 2026-09-30, al pasar sus nombres a inglés, y la 0002 y la 0003 en la limpieza de nombres fuera del glosario). Desde el primer deploy, la regla no tiene excepciones.
- **Las consultas se verifican al compilar:** sqlx revisa cada consulta SQL contra el esquema, así que renombrar una columna sin actualizar sus consultas no compila.
- **Los ids son `uuid` que genera la base** (`gen_random_uuid()`). En el código, cada tabla cuyo id se usa tiene su tipo (`CategoriaId(Uuid)`, `UnidadMedidaId(Uuid)`), definido en su área, para que el compilador no deje pasar el id de una cosa donde va el de otra. `kernel` reexporta `Uuid`, y las áreas no dependen de la librería directamente. *(2026-10-01)*
- **Collation `en_US.utf8` (libc) en todos los ambientes:** desarrollo (podman), CI (`postgres:18`) y producción (`template1` del Postgres del cluster) la tienen, confirmado el 2026-10-01. Los nombres se ordenan con `ORDER BY lower(nombre)`, y esta collation no toma en cuenta acentos, espacios ni signos ("Hojas a" antes que "Hoja z"). Las implementaciones en memoria la imitan con `kernel::nombres::sort_key`, y el contrato de cada repositorio compara las dos. Si un ambiente cambiara de collation, cambiaría el orden de las listas.
- **Lo derivado** (como el costo por promedio móvil) lo calcula la aplicación, no la base.
- **Un negocio nuevo en producción** no trae usuario administrador por omisión: se crea con un comando de la aplicación que pide la contraseña sin mostrarla.

### Ambientes
| Ambiente | Dónde | Quién escribe |
|---|---|---|
| **Producción** | Postgres del cluster, una base por negocio | Solo la aplicación, al hacer deploy. La IA nunca se conecta. |
| **Desarrollo** | Postgres de kukulkan (`dev_ro_inventario`) | El dueño y la IA. Se reconstruye con un script: migraciones, vistas y semilla. |
| **Pruebas de integración** | Una base nueva y vacía por prueba, creada y borrada por `sqlx::test` | Las pruebas |
| **Pruebas E2E** | La base de desarrollo, con la semilla | Playwright, con el usuario de pruebas |

- **El usuario de pruebas de la IA** lo crea `herramientas/dev-usuario-ia.sh` (cuando exista la semilla, será parte de ella) con `E2E_USER` y `E2E_PASS` del entorno, nunca del repo, y sin imprimirlos. En CI se crea uno con contraseña aleatoria en cada corrida. No existe en producción (actualizado 2026-09-30).
- **Datos reales en desarrollo:** antes del corte, la herramienta `migracion-v1` lee una copia de la base de la versión uno y llena la base de desarrollo. Después del corte, la base de desarrollo se puede refrescar desde una copia de producción.

## Descartado
- **Esquema completo más archivo de cambios a mano** (versión uno): se desalinean.
- **Vistas como migraciones:** cada ajuste a una vista sería una migración nueva que copia la vista entera; el historial se vuelve ruido.
- **Un ORM o generador de esquemas** (Diesel, SeaORM): el SQL a mano ya funcionó en la versión uno y sqlx lo verifica al compilar.
- **Un usuario administrador por omisión en las migraciones:** sería la misma contraseña en todos los negocios.
- **Aplicar cambios de base a mano en producción:** el orden depende de la memoria de quien lo corre.

## Consecuencias
- Una migración mala puede impedir que la instancia arranque. Por eso CI aplica todas las migraciones sobre una base vacía y sobre una copia migrada de datos reales, y antes de un deploy con migraciones se hace un respaldo manual (`pg_dump`) además del diario.
- **Las copias de producción en desarrollo no se anonimizan, por ahora:** el dueño es el único desarrollador y los datos reales facilitan depurar. Se revisa si entra otra persona al desarrollo.
- Las instrucciones de operación (comandos, variables de entorno, el script de desarrollo) van en `CLAUDE.md`; esta nota guarda el porqué.
