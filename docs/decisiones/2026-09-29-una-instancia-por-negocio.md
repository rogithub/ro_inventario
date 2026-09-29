# Una instancia y una base de datos por negocio

**Fecha:** 2026-09-29 · **Estado:** vigente

## Contexto
El sistema va a atender a varios negocios (la papelería primero; después otros parecidos). Hay que decidir cómo conviven sus datos. Los datos incluyen ventas, costos y, más adelante, información fiscal: una fuga entre negocios sería grave. El operador es una sola persona, en un cluster k3s propio, con respaldos por `pg_dump` y restauraciones frecuentes de copias de producción.

## Decisión
- **Cada negocio tiene su propia instancia de las aplicaciones (privada y pública) y su propia base de datos.** Las bases pueden vivir en el mismo servidor Postgres.
- **El código siempre atiende a un solo negocio.** No existe `tenant_id` ni ningún concepto de "negocio actual" en consultas o reglas: qué negocio es lo dice la configuración con que arranca la instancia.
- **Dar de alta un negocio es configuración y despliegue, no código:** una base nueva, un archivo de configuración y los manifests de su instancia (a futuro, generados con un ApplicationSet de ArgoCD).
- **Los negocios se comunican solo por la red** (API), nunca con consultas entre bases, aunque estén en el mismo servidor. Un negocio vecino y uno en otra infraestructura se conectan igual.

## Descartado
- **Una base compartida con columna `tenant_id`:** un `WHERE` olvidado muestra los datos de un negocio a otro; es el error de seguridad más común en sistemas multi-negocio. Respaldar, restaurar o borrar un solo negocio se vuelve delicado.
- **Un esquema de Postgres por negocio:** casi la misma complejidad de operación que bases separadas, con menos aislamiento.
- **Una sola instancia que atiende a varios negocios con una base cada uno:** mete en el código el concepto de "negocio actual" (elegir la conexión por dominio en cada petición). Es la salida si algún día hay cientos de negocios; hoy no hace falta.

## Consecuencias
- **Aislamiento total:** se le puede prometer a un cliente que sus datos no conviven con los de nadie.
- **Operar un negocio es operar una base:** respaldo, restauración, migración desde otro sistema o baja, sin tocar a los demás. La papelería migra desde la versión uno a su propia base.
- **Un negocio puede mudarse** a otro servidor o a su propia infraestructura sin cambios de código.
- **Las migraciones de esquema corren en cada base.** Cada instancia aplica las suyas al arrancar; una versión nueva se despliega negocio por negocio.
- **Costo por negocio:** un par de procesos Rust (decenas de MB de memoria) y una base. Con decenas de negocios es poco; con cientos habría que revisar esta decisión.
- **La red comercial nace distribuida:** la cadena de suministro se diseña como mensajes entre instancias desde el principio.
