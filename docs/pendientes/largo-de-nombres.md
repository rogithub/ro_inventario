# Los nombres del catálogo no tienen largo máximo

**Salió de:** revisión del paso 6a (categorías), 2026-10-01 · **Cuándo:** por decidir

## Qué pasa
`NewCategoria::new` y `NewUnidadMedida::new` solo rechazan el nombre vacío. Un nombre de varios KB choca con el índice único sobre `lower(nombre)`: Postgres no acepta entradas de índice de más de unos 2.7 KB. La pantalla muestra entonces un error 500 con su id en vez de un aviso. Si el texto se comprime bien, se puede guardar un nombre enorme, que luego se pinta en cada lista.

## Por qué importa
Solo lo puede provocar alguien con `editar_catalogo`, así que el riesgo es bajo. Pero es un 500 donde debería haber un 422 con mensaje, y el mismo hueco se va a repetir en artículos, proveedores, clientes…

## Propuesta
Fijar un largo máximo para los nombres del catálogo (¿100 caracteres?). Se validaría en el `New…` del área, que devuelve su error con mensaje, con un `CHECK` igual en la migración. La pregunta para el dueño: ¿qué largo, y aplica a todos los nombres del catálogo o se decide por tabla?
