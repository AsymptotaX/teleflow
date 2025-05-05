pub mod http;

use crate::error::TelemetryError;
use polars::prelude::*;

/// Trait for sending processed telemetry to external systems (e.g., HTTP endpoints, Kafka, etc.)
#[async_trait::async_trait]
pub trait SinkWriter: Send + Sync {
    /// Sends a batch of telemetry data.
    async fn send(&self, df: &DataFrame) -> Result<(), TelemetryError>;
}

pub struct NoopSinkWriter;

#[async_trait::async_trait]
impl SinkWriter for NoopSinkWriter {
    async fn send(&self, _df: &DataFrame) -> Result<(), TelemetryError> {
        Ok(())
    }
}
