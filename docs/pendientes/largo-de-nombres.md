# Los nombres del catálogo no tienen largo máximo

**Salió de:** revisión del paso 6a (categorías), 2026-10-01 · **Cuándo:** paso 6b (renombrar categorías)

## Qué pasa
`NewCategoria::new` y `NewUnidadMedida::new` solo rechazan el nombre vacío. Un nombre de varios KB choca con el índice único sobre `lower(nombre)`: Postgres no acepta entradas de índice de más de unos 2.7 KB. La pantalla muestra entonces un error 500 con su id en vez de un aviso. Si el texto se comprime bien, se puede guardar un nombre enorme, que luego se pinta en cada lista.

## Por qué importa
Solo lo puede provocar alguien con `editar_catalogo`, así que el riesgo es bajo. Pero es un 500 donde debería haber un 422 con mensaje, y el mismo hueco se va a repetir en artículos, proveedores, clientes…

## Decisión del dueño (2026-10-01)
**150 caracteres para todos los nombres del catálogo.** Se valida en el `New…` del área, que devuelve su error con mensaje (422 en la pantalla), y con el mismo `CHECK` en la migración. El paso 6b lo aplica a categorías y a unidades de medida y borra esta nota. Los catálogos que vengan después (artículos, proveedores…) nacen con el tope.

**Comprobado contra la copia de la v1** (`herramientas/v1.sh`, 2026-10-01): ningún nombre pasa de 150.
- Productos: 2,336, el más largo de 68 caracteres; el 99 % tiene 54 o menos.
- Categorías: 275, la más larga de 29.
- Unidades: 5, la más larga de 9.
- Presentaciones: 4, la más larga de 20.
