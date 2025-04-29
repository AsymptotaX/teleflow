use clap::{Parser, Subcommand};

/// Teleflow - A fast telemetry pipeline
#[derive(Parser, Debug)]
#[command(author, version, about = "A lightweight telemetry ingestion system", long_about = None)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Commands,
}

/// Available commands
#[derive(Subcommand, Debug)]
pub enum Commands {
    /// Process a static file (e.g. Parquet)
    Process {
        /// Input file (Parquet format)
        #[arg(short, long, value_name = "FILE", help = "Path to input Parquet file")]
        input: String,

        /// Path to configuration file
        #[arg(short, long, value_name = "CONFIG", help = "Path to config.yaml")]
        config: String,
    },

    /// Start processing telemetry from MQTT
    ProcessMqtt {
        /// Path to configuration file
        #[arg(short, long, value_name = "CONFIG", help = "Path to config.yaml")]
        config: String,

        /// Internal MQTT buffer size (default: 1000)
        #[arg(short, long, value_name = "BUFFER", default_value_t = 1000, 
            help = "Optional MQTT buffer size")]
        buffer_size: usize,
    },

    /// Generate synthetic test data
    GenerateTest {
        /// Number of rows to generate
        #[arg(short, long, value_name = "ROWS", help = "Total number of rows")]
        rows: usize,

        /// Number of unique device IDs
        #[arg(short, long, value_name = "DEVICES", help = "Number of devices")]
        devices: usize,

        /// Output file path
        #[arg(short, long, value_name = "OUTPUT", help = "Path to save generated file (Parquet)")]
        output: String,
    },
}
