use crate::error::TelemetryError;
use crate::sink::SinkWriter;
use flate2::Compression;
use flate2::write::GzEncoder;
use log::warn;
use polars::prelude::*;
use reqwest::header::{CONTENT_ENCODING, CONTENT_TYPE};
use reqwest::{Client, RequestBuilder, Response};
use serde_json::Value;
use std::io::Write;
use tokio::time::{Duration, sleep};

/// A writer for sending data to an HTTP endpoint.
/// This struct handles formatting, compressing, and sending data batches to a specified HTTP endpoint.
pub struct HttpSinkWriter {
    /// The HTTP endpoint to send data to.
    endpoint: String,
    /// Optional authentication token for the HTTP endpoint.
    token: Option<String>,
}

impl HttpSinkWriter {
    /// Creates a new `HttpSinkWriter`.
    ///
    /// # Arguments
    ///
    /// * `base_endpoint` - The base URL of the HTTP endpoint.
    /// * `token` - Optional authentication token for the HTTP endpoint.
    /// * `org` - Optional organization name for the endpoint.
    /// * `bucket` - Optional bucket name for the endpoint.
    /// * `precision` - Optional timestamp precision for the endpoint.
    ///
    /// # Returns
    ///
    /// A new instance of `HttpSinkWriter`.
    pub fn new(
        base_endpoint: String,
        token: Option<String>,
        org: Option<String>,
        bucket: Option<String>,
        precision: Option<String>,
    ) -> Self {
        let mut endpoint = base_endpoint;

        if let (Some(org), Some(bucket)) = (&org, &bucket) {
            let precision = precision.unwrap_or_else(|| "ns".to_string());
            endpoint = format!(
                "{}?org={}&bucket={}&precision={}",
                endpoint, org, bucket, precision
            );
        }

        Self { endpoint, token }
    }
}

#[async_trait::async_trait]
impl SinkWriter for HttpSinkWriter {
    /// Sends a `DataFrame` to the HTTP endpoint.
    ///
    /// # Arguments
    ///
    /// * `df` - The `DataFrame` containing the data to be sent.
    ///
    /// # Returns
    ///
    /// A `Result` indicating success or failure.
    async fn send(&self, df: &DataFrame) -> Result<(), TelemetryError> {
        let client = Client::new();

        let columns = df.get_columns();
        let headers: Vec<String> = columns.iter().map(|s| s.name().to_string()).collect();
        let height = df.height();

        let mut lines = Vec::with_capacity(height);

        for idx in 0..height {
            let mut record = std::collections::HashMap::new();
            for (col_idx, header) in headers.iter().enumerate() {
                let val = columns[col_idx].get(idx).unwrap_or(AnyValue::Null);
                let v = match val {
                    AnyValue::Null => Value::Null,
                    AnyValue::Boolean(v) => Value::Bool(v),
                    AnyValue::String(v) => Value::String(v.to_string()),
                    AnyValue::Int64(v) => Value::Number(v.into()),
                    AnyValue::UInt64(v) => Value::Number((v as i64).into()),
                    AnyValue::Float64(v) => serde_json::Number::from_f64(v)
                        .map(Value::Number)
                        .unwrap_or(Value::Null),
                    AnyValue::Datetime(ts, _, _) => Value::Number(ts.into()),
                    AnyValue::Date(days) => Value::Number(days.into()),
                    _ => Value::Null,
                };
                record.insert(header.to_string(), v);
            }

            let device_id = record
                .get("device_id")
                .and_then(|v| v.as_str())
                .unwrap_or("unknown");
            let timestamp = record
                .get("timestamp")
                .and_then(|v| v.as_i64())
                .unwrap_or(0)
                * 1_000_000;

            let fields: Vec<String> = record
                .iter()
                .filter(|(k, _)| *k != "device_id" && *k != "timestamp")
                .filter_map(|(k, v)| match v {
                    Value::Number(num) => Some(format!("{}={}", k, num)),
                    Value::String(s) => Some(format!("{}=\"{}\"", k, s)),
                    Value::Bool(b) => Some(format!("{}={}", k, b)),
                    _ => None,
                })
                .collect();

            if fields.is_empty() {
                continue;
            }

            let fields_part = fields.join(",");

            let line = format!(
                "teleflow,device_id={} {} {}",
                device_id, fields_part, timestamp
            );
            lines.push(line);
        }

        if lines.is_empty() {
            return Ok(());
        }

        let raw_payload = lines.join("\n");

        let mut encoder = GzEncoder::new(Vec::new(), Compression::default());
        encoder.write_all(raw_payload.as_bytes())?;
        let compressed_payload = encoder.finish()?;

        let mut req = client
            .post(&self.endpoint)
            .header(CONTENT_ENCODING, "gzip")
            .header(CONTENT_TYPE, "text/plain; charset=utf-8")
            .body(compressed_payload);

        if let Some(token) = &self.token {
            req = req.bearer_auth(token);
        }

        let resp = send_with_retries(req, 5).await?;

        let status = resp.status();
        let text = resp.text().await.unwrap_or_default();

        if !status.is_success() {
            return Err(TelemetryError::InvalidInput(format!(
                "HTTP sink failed: {}: {}",
                status, text
            )));
        }

        Ok(())
    }
}

/// Sends an HTTP request with retries in case of failure.
///
/// # Arguments
///
/// * `req` - The `RequestBuilder` for the HTTP request.
/// * `retries` - The maximum number of retry attempts.
///
/// # Returns
///
/// A `Result` containing the `Response` on success, or a `TelemetryError` on failure.
async fn send_with_retries(
    req: RequestBuilder,
    retries: usize,
) -> Result<Response, TelemetryError> {
    let mut attempt = 0;
    let mut delay = Duration::from_millis(500);
    let max_delay = Duration::from_secs(10);

    loop {
        let req = req
            .try_clone()
            .ok_or_else(|| TelemetryError::InvalidInput("Cannot clone request".into()))?;

        match req.send().await {
            Ok(resp) => return Ok(resp),
            Err(e) if attempt < retries => {
                warn!(
                    "HTTP send attempt {} failed: {}. Retrying in {:?}",
                    attempt + 1,
                    e,
                    delay
                );
                sleep(delay).await;
                attempt += 1;
                delay = (delay * 2).min(max_delay);
            }
            Err(e) => {
                return Err(TelemetryError::InvalidInput(format!(
                    "HTTP send failed after retries: {e}"
                )));
            }
        }
    }
}
