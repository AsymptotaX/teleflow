use clap::{Parser, Subcommand};

/// Teleflow - A fast and lightweight telemetry ingestion pipeline.
///
/// Provides a CLI interface for processing telemetry from static files or MQTT,
/// generating synthetic test data, and exporting to various formats or sinks.
#[derive(Parser, Debug)]
#[command(author, version, about = "A lightweight telemetry ingestion system", long_about = None)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Commands,
    #[clap(long, help = "Force output to stdout")]
    pub stdout: bool,
}

/// Subcommands available in the Teleflow CLI.
#[derive(Subcommand, Debug)]
pub enum Commands {
    /// Process telemetry from a static file (e.g. Parquet).
    ///
    /// Applies filters, transforms, and writes output to configured destinations.
    Process {
        /// Input file (Parquet format)
        #[arg(short, long, value_name = "FILE", help = "Path to input Parquet file")]
        input: String,

        /// Path to configuration file
        #[arg(short, long, value_name = "CONFIG", help = "Path to config.yaml")]
        config: String,
    },

    /// Connects to an MQTT broker and streams telemetry data.
    ///
    /// Batches incoming messages and processes them according to config.
    ProcessMqtt {
        /// Path to configuration file
        #[arg(short, long, value_name = "CONFIG", help = "Path to config.yaml")]
        config: String,

        /// Internal MQTT buffer size (default: 10000)
        #[arg(
            short,
            long,
            value_name = "BUFFER",
            default_value_t = 10000,
            help = "Optional buffer size"
        )]
        buffer_size: usize,

        /// Eventloop buffer size for MQTT client (default: 15000)
        #[arg(
            long,
            value_name = "EVENTLOOP_BUFFER",
            default_value_t = 15000,
            help = "MQTT client eventloop buffer size"
        )]
        eventloop_buffer_size: usize,
    },

    /// Generate synthetic telemetry data for testing and benchmarking.
    GenerateTest {
        /// Number of rows to generate
        #[arg(short, long, value_name = "ROWS", help = "Total number of rows")]
        rows: usize,

        /// Number of unique device IDs
        #[arg(short, long, value_name = "DEVICES", help = "Number of devices")]
        devices: usize,

        /// Output file path
        #[arg(
            short,
            long,
            value_name = "OUTPUT",
            help = "Path to save generated file (Parquet)"
        )]
        output: String,
    },
}
