use clap::Parser;
use log::{error, info, warn};
use std::sync::Arc;
use teleflow::output::writer::NoopWriter;
use teleflow::sink::NoopSinkWriter;
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

/// Initializes the appropriate output writer based on the configuration.
///
/// Supports Parquet, CSV, and JSON formats. If `output.enabled = false`, a no-op writer is returned.
fn create_output_writer(
    config: &teleflow::config::types::ProcessingConfig,
) -> Arc<dyn OutputWriter> {
    if config.output.enabled == Some(false) {
        return Arc::new(NoopWriter);
    }

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

/// Initializes the appropriate sink writer based on the configuration.
///
/// Currently supports HTTP sinks (e.g. InfluxDB). If `sink.enabled = false`, returns a no-op sink.
fn create_sink_writer(sink_config: &SinkConfig) -> Result<Arc<dyn SinkWriter>, TelemetryError> {
    if sink_config.enabled == Some(false) {
        return Ok(Arc::new(NoopSinkWriter));
    }

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

/// The main entry point for the Teleflow application.
///
/// This function:
/// - Parses CLI arguments
/// - Loads configuration from file
/// - Dispatches one of the following commands:
///     - `process`: Reads telemetry from a Parquet file
///     - `process-mqtt`: Connects to an MQTT broker and streams data
///     - `generate-test`: Generates synthetic telemetry data
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
            eventloop_buffer_size,
        } => {
            info!("Starting MQTT processing (config: {})", config);
            let cli_config = load_config(&config)?;
            let output_writer = create_output_writer(&cli_config);

            let sink_writer = if let Some(sink_config) = &cli_config.sink {
                Some(create_sink_writer(sink_config)?)
            } else {
                None
            };

            let buffer_size = cli_config.mqtt.buffer_size.unwrap_or(buffer_size);

            let eventloop_buffer_size = cli_config
                .mqtt
                .eventloop_buffer_size
                .unwrap_or(eventloop_buffer_size);

            if eventloop_buffer_size >= buffer_size {
                warn!(
                    "eventloop_buffer_size ({}) >= buffer_size ({}). This may cause congestion.",
                    eventloop_buffer_size, buffer_size
                );
            }

            info!(
                "MQTT buffer size: {}, eventloop buffer size: {}",
                buffer_size, eventloop_buffer_size
            );

            process_mqtt(
                cli_config,
                output_writer,
                sink_writer,
                buffer_size,
                eventloop_buffer_size,
            )
            .await?;
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

/// Handles graceful shutdown by listening for Ctrl+C (SIGINT).
///
/// Exits the process cleanly when triggered.
async fn shutdown_signal() -> Result<(), TelemetryError> {
    use tokio::signal;
    signal::ctrl_c().await.map_err(|e| {
        TelemetryError::InvalidInput(format!("Failed to listen for shutdown signal: {e}"))
    })?;
    info!("Shutdown signal received. Stopping Teleflow...");
    std::process::exit(0);
}
