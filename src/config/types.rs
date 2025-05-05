use serde::{Deserialize, Serialize};

/// Mapping of columns in the input data.
#[derive(Debug, Deserialize, Clone)]
pub struct ColumnMapping {
    /// Column name containing device IDs
    pub device_id: String,

    /// Column name containing timestamps
    pub timestamp: String,
}

/// A single rule for filtering data.
#[derive(Debug, Deserialize, Clone)]
pub struct FilterRule {
    /// Column to apply the filter on
    pub column: String,

    /// Operator to use ("eq", "gt", "lt", etc.)
    pub operator: String,

    /// Value to compare against
    pub value: serde_json::Value,
}

/// Main processing configuration.
#[derive(Debug, Deserialize, Clone)]
pub struct ProcessingConfig {
    /// Mapping of columns in the dataset
    pub columns: ColumnMapping,

    /// Optional list of filtering rules
    pub filters: Option<Vec<FilterRule>>,

    /// Size of rolling window for processing
    pub rolling: usize,

    /// MQTT connection configuration
    pub mqtt: MqttConfig,

    /// Output configuration (e.g., file format, path)
    pub output: OutputConfig,

    /// Optional sink configuration (e.g., HTTP for InfluxDB)
    pub sink: Option<SinkConfig>,
}

/// Configuration for outputting processed data to files.
#[derive(Debug, Deserialize, Clone)]
pub struct OutputConfig {
    /// Output directory path
    pub path: String,

    /// Format of output files (parquet, csv, json)
    pub format: String,

    /// Naming convention for batch files
    pub batch_naming: String,

    /// Compression type for parquet files
    pub compression: Option<String>,

    /// Maximum number of batch files to retain
    pub max_batches: Option<usize>,

    /// Minimum free disk space (GB) required to continue writing
    pub min_disk_space_gb: Option<f64>,

    /// Enable or disable output
    pub enabled: Option<bool>,
}

/// Configuration for MQTT connection.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MqttConfig {
    /// MQTT broker hostname or IP
    pub host: String,

    /// MQTT broker port
    pub port: u16,

    /// Whether to use TLS encryption
    pub use_tls: bool,

    /// Optional username for MQTT authentication
    #[serde(default)]
    pub username: Option<String>,

    /// Optional password for MQTT authentication
    #[serde(default)]
    pub password: Option<String>,

    /// Topic to subscribe to
    pub topic: String,

    /// Keep-alive interval in seconds
    #[serde(default = "default_keep_alive")]
    pub keep_alive: u64,

    /// Connect timeout in seconds
    #[serde(default = "default_connect_timeout")]
    pub connect_timeout: u64,

    /// Optional MQTT client ID
    pub client_id: Option<String>,

    //  Optional buffer size
    pub buffer_size: Option<usize>,

    /// Option event loop buffer size
    pub eventloop_buffer_size: Option<usize>,
}

/// Configuration for data sink (e.g., HTTP push to InfluxDB).
#[derive(Debug, Deserialize, Clone)]
pub struct SinkConfig {
    /// Sink type (http, kafka, mqtt, etc.)
    pub r#type: String,

    /// Sink endpoint URL
    pub endpoint: String,

    /// Optional authentication token
    pub auth_token: Option<String>,

    /// Optional data format for sink (influx, json)
    pub format: Option<String>,

    /// Optional organization (for InfluxDB)
    pub org: Option<String>,

    /// Optional bucket/database (for InfluxDB)
    pub bucket: Option<String>,

    /// Optional time precision (e.g., ns, us)
    pub precision: Option<String>,

    /// Enable or disable the sink
    pub enabled: Option<bool>,

    /// Optional batch size for sink
    pub eventloop_buffer_size: Option<bool>,
}

/// Default keep-alive timeout (in seconds) for MQTT
fn default_keep_alive() -> u64 {
    5
}

/// Default connect timeout (in seconds) for MQTT
fn default_connect_timeout() -> u64 {
    60
}
