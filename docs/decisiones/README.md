# Decisiones de diseño

Una nota por decisión que no es obvia leyendo el código: **por qué** se hizo así y **qué se descartó**. El *qué* y el *cómo* los explican el código y las pruebas; aquí va lo que se pierde con el tiempo.

- Archivo: `AAAA-MM-DD-tema-corto.md`.
- Si una decisión se revierte, no se borra la nota: se agrega una nueva y en la vieja se marca **Estado: reemplazada por …**.
- Corta: si no cabe en una pantalla, probablemente son dos decisiones.

## Plantilla

```markdown
# Título en una frase

**Fecha:** AAAA-MM-DD · **Estado:** vigente

## Contexto
Qué problema o situación llevó a decidir.

## Decisión
Qué se hizo.

## Descartado
Qué alternativas se consideraron y por qué no.

## Consecuencias
Qué implica hacia adelante: costos, reglas nuevas, cómo revertir.
```

## Índice

| Fecha | Decisión |
|---|---|
| 2026-09-29 | [Una instancia y una base de datos por negocio](2026-09-29-una-instancia-por-negocio.md) |
| 2026-09-29 | [La configuración vive en tres capas, y cada dato en una sola](2026-09-29-capas-de-configuracion.md) |
| 2026-09-29 | [Un crate por área del negocio, y el compilador vigila las dependencias](2026-09-29-estructura-de-crates.md) |
| 2026-09-29 | [Nombres: programación en inglés, negocio en español](2026-09-29-nombres.md) |
| 2026-09-29 | [La base de datos: migraciones, vistas regenerables, semillas y ambientes](2026-09-29-base-de-datos-y-ambientes.md) |
| 2026-09-29 | [Logs estructurados desde el día uno, legibles por la IA](2026-09-29-logs-desde-el-dia-uno.md) |
