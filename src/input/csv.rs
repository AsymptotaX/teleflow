use crate::error::TelemetryError;
use polars::prelude::*;
use std::fs::File;
use std::io::BufReader;

/// Read a Parquet file into a DataFrame.
///
/// # Arguments
///
/// * `path` - Path to the input Parquet file
///
/// # Errors
///
/// Returns `TelemetryError` if the file cannot be read or parsed.
pub fn read_parquet(path: &str) -> Result<DataFrame, TelemetryError> {
    let file = File::open(path).map_err(|e| {
        TelemetryError::Io(std::io::Error::new(
            e.kind(),
            format!("Failed to open file {path}: {e}"),
        ))
    })?;
    let reader = BufReader::new(file);

    ParquetReader::new(reader).finish().map_err(|e| {
        TelemetryError::Polars(PolarsError::ComputeError(
            format!("Failed to parse parquet {path}: {e}").into(),
        ))
    })
}
