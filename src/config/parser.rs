use crate::{config::types::ProcessingConfig, error::TelemetryError};
use std::fs;

/// Loads the configuration file and parses it into a `ProcessingConfig`.
///
/// # Arguments
///
/// * `path` - The file path to the configuration file.
///
/// # Returns
///
/// A `Result` containing the parsed `ProcessingConfig` on success, or a `TelemetryError` on failure.
///
/// # Errors
///
/// Returns a `TelemetryError` if the file cannot be read, the format is unsupported,
/// or the content cannot be parsed as YAML or JSON.
pub fn load_config(path: &str) -> Result<ProcessingConfig, TelemetryError> {
    let content = fs::read_to_string(path)?;

    match path {
        p if p.ends_with(".yaml") || p.ends_with(".yml") => serde_yaml::from_str(&content)
            .map_err(|e| TelemetryError::InvalidInput(format!("YAML parse error: {}", e))),
        p if p.ends_with(".json") => serde_json::from_str(&content)
            .map_err(|e| TelemetryError::InvalidInput(format!("JSON parse error: {}", e))),
        _ => Err(TelemetryError::InvalidInput(
            "Config must be YAML or JSON".to_string(),
        )),
    }
}
