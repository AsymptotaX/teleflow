use crate::{
    config::types::ProcessingConfig, error::TelemetryError, output::writer::OutputWriter,
    processing::filter::apply_filters, sink::SinkWriter,
};
use log::{info, warn};
use polars::prelude::*;
use std::sync::Arc;
use std::time::Instant;

/// Processes telemetry data by applying filters, rolling calculations, and writing the results.
///
/// # Arguments
///
/// * `df` - The `DataFrame` containing the telemetry data to process.
/// * `config` - The `ProcessingConfig` containing column mappings, filters, and rolling window size.
/// * `output_writer` - An `Arc` to an implementation of the `OutputWriter` trait for writing the processed data.
/// * `sink_writer` - An optional `Arc` to an implementation of the `SinkWriter` trait for sending the processed data.
///
/// # Returns
///
/// A `Result` indicating success or failure.
///
/// # Errors
///
/// Returns a `TelemetryError` if any step in the processing pipeline fails.
pub async fn process_telemetry(
    df: DataFrame,
    config: ProcessingConfig,
    output_writer: Arc<dyn OutputWriter>,
    sink_writer: Option<Arc<dyn SinkWriter>>,
) -> Result<(), TelemetryError> {
    let timer = Instant::now();
    let columns = config.columns;
    let filters = config.filters;
    let rolling = config.rolling;

    // Exclude device ID and timestamp columns from processing.
    let exclude_fields = vec![columns.device_id.clone(), columns.timestamp.clone()];
    let all_columns: Vec<String> = df
        .get_column_names()
        .iter()
        .map(|name| name.to_string())
        .filter(|name| !exclude_fields.contains(name))
        .collect();

    // Identify numeric columns for rolling calculations.
    let numeric_columns: Vec<String> = all_columns
        .iter()
        .filter(|name| {
            if let Ok(series) = df.column(name) {
                matches!(series.dtype(), DataType::Float64 | DataType::Int64)
            } else {
                false
            }
        })
        .cloned()
        .collect();

    let mut lf = df.lazy();

    // Apply filters if specified in the configuration.
    if let Some(rules) = filters {
        let lf_clone = lf.clone();
        lf = match apply_filters(lf_clone, &rules) {
            Ok(new_lf) => new_lf,
            Err(e) => {
                warn!(
                    "Failed to apply filters, continuing without filters: {:?}",
                    e
                );
                lf
            }
        };
    }

    // Add device ID and timestamp columns to the lazy frame.
    lf = lf.with_columns([
        col(&columns.device_id).alias("device_id"),
        col(&columns.timestamp)
            .cast(DataType::Int64)
            .alias("timestamp"),
    ]);

    // Retain non-numeric columns without modifications.
    for col_name in &all_columns {
        if !numeric_columns.contains(col_name) {
            lf = lf.with_columns([col(col_name).alias(col_name)]);
        }
    }

    // Apply rolling mean calculations to numeric columns.
    for value_col in &numeric_columns {
        lf = lf.with_columns([
            col(value_col).cast(DataType::Float64).alias(value_col),
            col(value_col)
                .rolling_mean(RollingOptionsFixedWindow {
                    window_size: rolling,
                    min_periods: rolling,
                    weights: None,
                    center: false,
                    fn_params: None,
                })
                .alias(&format!("{}_rolling", value_col)),
        ]);
    }

    // Collect the processed data into a `DataFrame`.
    let result = lf.collect()?;

    // Write the processed data to the output.
    output_writer.write_batch(&result)?;

    // Optionally send the processed data to a sink.
    if let Some(sink) = sink_writer {
        sink.send(&result).await?;
    }

    info!(
        "Processed {} rows in {:.3?}",
        result.height(),
        timer.elapsed()
    );

    Ok(())
}
