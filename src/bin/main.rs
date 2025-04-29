use clap::Parser;
use log::{error, info};
use std::sync::Arc;
use teleflow::{
    cli::commands::{Cli, Commands},
    config::parser::load_config,
    config::types::SinkConfig,
    error::TelemetryError,
    input::{csv::read_parquet, mqtt::process_mqtt},
    output::writer::{
        CsvFileWriter, JsonFileWriter, OutputWriter, ParquetFileWriter, get_parquet_compression,
    },
    processing::telemetry::process_telemetry,
    sink::{SinkWriter, http::HttpSinkWriter},
    utils::generate_test_data,
};

/// Creates an output writer based on the provided processing configuration.
///
/// # Arguments
///
/// * `config` - A reference to the `ProcessingConfig` containing output format and path details.
///
/// # Returns
///
/// An `Arc` containing a trait object implementing the `OutputWriter` trait.
///
/// # Panics
///
/// Panics if the specified output format is unsupported.
fn create_output_writer(
    config: &teleflow::config::types::ProcessingConfig,
) -> Arc<dyn OutputWriter> {
    match config.output.format.as_str() {
        "parquet" => Arc::new(ParquetFileWriter::new(
            config.output.path.clone(),
            get_parquet_compression(config.output.compression.clone()),
            config.output.max_batches,
            config.output.min_disk_space_gb,
        )),
        "csv" => Arc::new(CsvFileWriter::new(config.output.path.clone())),
        "json" => Arc::new(JsonFileWriter::new(config.output.path.clone())),
        _ => panic!("Unsupported output format: {}", config.output.format),
    }
}

/// Creates a sink writer based on the provided sink configuration.
///
/// # Arguments
///
/// * `sink_config` - A reference to the `SinkConfig` containing sink type and endpoint details.
///
/// # Returns
///
/// A `Result` containing an `Arc` with a trait object implementing the `SinkWriter` trait on success,
/// or a `TelemetryError` on failure.
///
/// # Errors
///
/// Returns a `TelemetryError` if the specified sink type is unsupported.
fn create_sink_writer(sink_config: &SinkConfig) -> Result<Arc<dyn SinkWriter>, TelemetryError> {
    match sink_config.r#type.as_str() {
        "http" => Ok(Arc::new(HttpSinkWriter::new(
            sink_config.endpoint.clone(),
            sink_config.auth_token.clone(),
            sink_config.org.clone(),
            sink_config.bucket.clone(),
            sink_config.precision.clone(),
        ))),
        _ => Err(TelemetryError::InvalidInput(format!(
            "Unsupported sink type: {}",
            sink_config.r#type
        ))),
    }
}

/// The main entry point for the application.
///
/// Parses command-line arguments, initializes logging, and executes the specified command.
///
/// # Returns
///
/// A `Result` indicating success or failure.
///
/// # Errors
///
/// Returns a `TelemetryError` if any step in the command execution fails.
#[tokio::main]
async fn main() -> Result<(), TelemetryError> {
    env_logger::init();
    let cli = Cli::parse();

    tokio::spawn(async {
        if let Err(e) = shutdown_signal().await {
            error!("Shutdown error: {:?}", e);
        }
    });

    match cli.command {
        Commands::Process { input, config } => {
            info!("Reading Parquet file: {}", input);
            let config = load_config(&config)?;
            let output_writer = create_output_writer(&config);

            let df = read_parquet(&input)?;
            process_telemetry(df, config, output_writer, None).await?;
        }

        Commands::ProcessMqtt {
            config,
            buffer_size,
        } => {
            info!("Starting MQTT processing (config: {})", config);
            let config = load_config(&config)?;
            let output_writer = create_output_writer(&config);

            let sink_writer = if let Some(sink_config) = &config.sink {
                Some(create_sink_writer(sink_config)?)
            } else {
                None
            };

            process_mqtt(config, output_writer, sink_writer, buffer_size).await?;
        }

        Commands::GenerateTest {
            rows,
            devices,
            output,
        } => {
            let mut df = generate_test_data(rows, devices)?;
            let mut file = std::fs::File::create(&output)?;
            polars::prelude::ParquetWriter::new(&mut file)
                .finish(&mut df)
                .map_err(TelemetryError::Polars)?;
            println!("Generated {rows} rows with {devices} devices → {output}");
        }
    }

    Ok(())
}

/// Waits for a shutdown signal (Ctrl+C) and gracefully stops the application.
///
/// # Returns
///
/// A `Result` indicating success or failure.
///
/// # Errors
///
/// Returns a `TelemetryError` if the shutdown signal cannot be listened for.
async fn shutdown_signal() -> Result<(), TelemetryError> {
    use tokio::signal;
    signal::ctrl_c().await.map_err(|e| {
        TelemetryError::InvalidInput(format!("Failed to listen for shutdown signal: {e}"))
    })?;
    info!("Shutdown signal received. Stopping Teleflow...");
    std::process::exit(0);
}
