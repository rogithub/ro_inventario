-- El nombre de una unidad no se repite sin importar mayúsculas: "hoja" choca con "Hoja".
ALTER TABLE unidades_medida DROP CONSTRAINT unidades_medida_nombre_key;
CREATE UNIQUE INDEX unidades_medida_nombre_unique ON unidades_medida (lower(nombre));
