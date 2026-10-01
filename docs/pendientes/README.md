# Pendientes

Lo que sale al trabajar (un hallazgo del revisor, una duda, algo que se pospuso) y no se resuelve en el paso en que apareció. Así no se pierde entre sesiones ni se queda en una conversación.

- Archivo: `tema-corto.md`, una nota por pendiente.
- Se anota **qué pasa, por qué importa y qué se propone**, y de dónde salió (paso, revisión).
- **Al resolverlo, la nota se borra** en el mismo commit que lo resuelve (el historial la conserva). Si lo resuelto deja una decisión que no es obvia, va a `docs/decisiones/`.
- Si un pendiente se descarta, también se borra, y el commit dice por qué.
- Corta: si no cabe en una pantalla, probablemente son dos.

## Plantilla

```markdown
# Título en una frase

**Salió de:** paso o revisión, AAAA-MM-DD · **Cuándo:** antes de …, con …, cuando haga falta

## Qué pasa
Lo observado, con archivo y línea si aplica.

## Por qué importa
Qué se rompe o qué cuesta dejarlo.

## Propuesta
Qué se haría. Si hay que decidir algo, la pregunta para el dueño.
```

## Índice

| Pendiente | Cuándo |
|---|---|
| [Paso 7: artículos y productos, en cinco partes](paso-7-productos.md) | durante el paso 7 |
| [Migrar las categorías desde la v1](migracion-v1-categorias.md) | con artículos |
| [Los nombres de la v1 vienen en mayúsculas y algunos con dobles espacios](nombres-de-la-v1.md) | con artículos |
| [Herramienta para consultar los logs de producción](logs-de-produccion.md) | con la v2 en producción |
