//! Lee y valida el archivo del negocio (`negocio.toml`, capa 2 de configuración).
//!
//! Detalle: docs/decisiones/2026-09-29-capas-de-configuracion.md

use std::fmt;
use std::path::{Path, PathBuf};

use serde::Deserialize;

/// Configuración de un negocio, ya validada. Solo trae lo que hoy se usa;
/// cada sección nueva llega con el módulo que la necesita.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct NegocioConfig {
    pub negocio: Negocio,
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Negocio {
    pub nombre: String,
    pub time_zone: String,
}

#[derive(Debug, PartialEq, Eq)]
pub enum ConfigError {
    /// No se pudo leer el archivo.
    Read { path: PathBuf, message: String },
    /// El archivo no es TOML válido o no tiene la forma esperada (falta una clave, sobra una, tipo equivocado).
    Parse(String),
    /// Tiene la forma correcta, pero un valor no tiene sentido (p. ej. un nombre vacío).
    Invalid(String),
}

impl fmt::Display for ConfigError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ConfigError::Read { path, message } => {
                write!(f, "no se pudo leer {}: {message}", path.display())
            }
            ConfigError::Parse(message) => write!(f, "negocio.toml mal formado: {message}"),
            ConfigError::Invalid(message) => write!(f, "negocio.toml no es válido: {message}"),
        }
    }
}

impl std::error::Error for ConfigError {}

/// Lee el archivo y lo valida.
pub fn load_negocio_config(path: &Path) -> Result<NegocioConfig, ConfigError> {
    let text = std::fs::read_to_string(path).map_err(|e| ConfigError::Read {
        path: path.to_path_buf(),
        message: e.to_string(),
    })?;
    parse_negocio_config(&text)
}

/// Valida el texto de un `negocio.toml`. Función pura: no toca el disco.
pub fn parse_negocio_config(text: &str) -> Result<NegocioConfig, ConfigError> {
    let config: NegocioConfig =
        toml::from_str(text).map_err(|e| ConfigError::Parse(e.to_string()))?;
    validate(&config)?;
    Ok(config)
}

fn validate(config: &NegocioConfig) -> Result<(), ConfigError> {
    if config.negocio.nombre.trim().is_empty() {
        return Err(ConfigError::Invalid("[negocio] nombre está vacío".into()));
    }
    if config.negocio.time_zone.trim().is_empty() {
        return Err(ConfigError::Invalid(
            "[negocio] time_zone está vacío".into(),
        ));
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    const REPO_EXAMPLE: &str = include_str!("../../../negocio.ejemplo.toml");

    #[test]
    fn el_ejemplo_del_repo_es_valido() {
        let config = parse_negocio_config(REPO_EXAMPLE).unwrap();
        assert_eq!(config.negocio.time_zone, "America/Cancun");
    }

    #[test]
    fn si_falta_un_campo_no_se_acepta() {
        let error = parse_negocio_config("[negocio]\nnombre = \"Papelería\"\n").unwrap_err();
        assert!(
            matches!(error, ConfigError::Parse(ref m) if m.contains("time_zone")),
            "{error}"
        );
    }

    #[test]
    fn una_clave_mal_escrita_no_se_acepta() {
        let text = "[negocio]\nnombr = \"Papelería\"\ntime_zone = \"America/Cancun\"\n";
        let error = parse_negocio_config(text).unwrap_err();
        assert!(
            matches!(error, ConfigError::Parse(ref m) if m.contains("nombr")),
            "{error}"
        );
    }

    #[test]
    fn un_nombre_vacio_no_se_acepta() {
        let text = "[negocio]\nnombre = \"  \"\ntime_zone = \"America/Cancun\"\n";
        let error = parse_negocio_config(text).unwrap_err();
        assert!(
            matches!(error, ConfigError::Invalid(ref m) if m.contains("nombre")),
            "{error}"
        );
    }

    #[test]
    fn una_zona_horaria_vacia_no_se_acepta() {
        let text = "[negocio]\nnombre = \"Papelería\"\ntime_zone = \"\"\n";
        let error = parse_negocio_config(text).unwrap_err();
        assert!(
            matches!(error, ConfigError::Invalid(ref m) if m.contains("time_zone")),
            "{error}"
        );
    }

    #[test]
    fn un_archivo_que_no_existe_da_error_de_lectura() {
        let error = load_negocio_config(Path::new("/no/existe/negocio.toml")).unwrap_err();
        assert!(matches!(error, ConfigError::Read { .. }), "{error}");
    }
}
