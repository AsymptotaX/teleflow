use polars::error::PolarsError;
use thiserror::Error;

/// Error types used across Teleflow
#[derive(Error, Debug)]
#[non_exhaustive]
pub enum TelemetryError {
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("Polars error: {0}")]
    Polars(#[from] PolarsError),

    #[error("Invalid input: {0}")]
    InvalidInput(String),

    #[error("Parse error: {0}")]
    Parse(String),

    #[error("Join error: {0}")]
    Join(String),

    #[error("MQTT error: {0}")]
    Mqtt(String),
}
