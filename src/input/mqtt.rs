use crate::{
    config::types::ProcessingConfig, error::TelemetryError, output::writer::OutputWriter,
    processing::telemetry::process_telemetry, sink::SinkWriter,
};
use log::{info, warn};
use polars::df;
use polars::prelude::*;
use rumqttc::{
    AsyncClient, Event, MqttOptions, NetworkOptions, Packet, QoS, TlsConfiguration, Transport,
};
use serde_json::Value;
use std::collections::{HashMap, HashSet};
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::{mpsc, Semaphore};
use tokio::task::JoinHandle;
use tokio::time::{sleep, timeout, Duration as TokioDuration, Instant};

#[derive(Clone)]
struct DynamicSemaphore {
    semaphore: Arc<Semaphore>,
    stats: Arc<tokio::sync::Mutex<Stats>>,
}

struct Stats {
    last_check: Instant,
    completed_batches: usize,
}

impl DynamicSemaphore {
    fn new(initial: usize) -> Self {
        Self {
            semaphore: Arc::new(Semaphore::new(initial)),
            stats: Arc::new(tokio::sync::Mutex::new(Stats {
                last_check: Instant::now(),
                completed_batches: 0,
            })),
        }
    }

    async fn acquire(&self) -> tokio::sync::OwnedSemaphorePermit {
        self.semaphore.clone().acquire_owned().await.unwrap()
    }

    async fn batch_completed(&self) {
        let mut stats = self.stats.lock().await;
        stats.completed_batches += 1;

        let elapsed = stats.last_check.elapsed();
        if elapsed > TokioDuration::from_secs(10) {
            let batches_per_sec = stats.completed_batches as f64 / elapsed.as_secs_f64();

            if batches_per_sec > 5.0 && self.semaphore.available_permits() < 32 {
                self.semaphore.add_permits(1);
                info!(
                    "Increased parallelism to {} permits",
                    self.semaphore.available_permits()
                );
            } else if batches_per_sec < 2.0 && self.semaphore.available_permits() > 2 {
                let _ = self.semaphore.try_acquire();
                info!(
                    "Decreased parallelism to {} permits",
                    self.semaphore.available_permits()
                );
            }

            stats.last_check = Instant::now();
            stats.completed_batches = 0;
        }
    }

    fn available(&self) -> usize {
        self.semaphore.available_permits()
    }
}

pub async fn process_mqtt(
    config: ProcessingConfig,
    output_writer: Arc<dyn OutputWriter>,
    sink_writer: Option<Arc<dyn SinkWriter>>,
    buffer_size: usize,
) -> Result<(), TelemetryError> {
    let (_, mut eventloop) = reconnect_and_resubscribe(&config).await?;

    let (tx, rx) = mpsc::channel(buffer_size);
    let processor_handle =
        spawn_processor(rx, config.clone(), output_writer, sink_writer, buffer_size);

    let mut reconnect_delay_secs = 5;
    let max_reconnect_delay_secs = 60;

    loop {
        match eventloop.poll().await {
            Ok(Event::Incoming(Packet::Publish(publish))) => {
                if let Some(parsed) = parse_payload(&publish.payload) {
                    if let Err(e) = send_to_buffer(&tx, parsed).await {
                        warn!("Failed to send to processor: {:?}", e);
                        break;
                    }
                }
            }
            Ok(_) => continue,
            Err(e) => {
                warn!(
                    "MQTT eventloop error: {:?} — trying to reconnect after {}s",
                    e, reconnect_delay_secs
                );
                sleep(TokioDuration::from_secs(reconnect_delay_secs)).await;

                match reconnect_and_resubscribe(&config).await {
                    Ok((_, new_eventloop)) => {
                        eventloop = new_eventloop;
                        info!("Successfully reconnected and re-subscribed");
                        reconnect_delay_secs = 5;
                    }
                    Err(e) => {
                        warn!(
                            "Reconnect failed: {:?}. Will retry in {}s...",
                            e, reconnect_delay_secs
                        );
                        reconnect_delay_secs =
                            (reconnect_delay_secs * 2).min(max_reconnect_delay_secs);
                    }
                }
            }
        }
    }

    if let Err(e) = processor_handle.await {
        warn!("Processor task failed to join: {}", e);
    }
    Ok(())
}

async fn reconnect_and_resubscribe(
    config: &ProcessingConfig,
) -> Result<(AsyncClient, rumqttc::EventLoop), TelemetryError> {
    let (client, eventloop) = create_mqtt_client(config).await?;
    subscribe_to_topic(&client, &config.mqtt.topic).await?;
    Ok((client, eventloop))
}

async fn create_mqtt_client(
    config: &ProcessingConfig,
) -> Result<(AsyncClient, rumqttc::EventLoop), TelemetryError> {
    let mqtt_config = &config.mqtt;
    let client_id = mqtt_config
        .client_id
        .clone()
        .unwrap_or_else(|| format!("teleflow-{}", uuid::Uuid::new_v4()));

    let mut mqttoptions = MqttOptions::new(client_id, &mqtt_config.host, mqtt_config.port);
    mqttoptions.set_keep_alive(Duration::from_secs(mqtt_config.keep_alive));

    if let (Some(username), Some(password)) = (&mqtt_config.username, &mqtt_config.password) {
        mqttoptions.set_credentials(username, password);
    }

    if mqtt_config.use_tls {
        mqttoptions.set_transport(Transport::Tls(TlsConfiguration::default()));
    }

    let mut network_options = NetworkOptions::new();
    network_options.set_connection_timeout(mqtt_config.connect_timeout);

    let (client, mut eventloop) = AsyncClient::new(mqttoptions, 30_000);
    eventloop.set_network_options(network_options);

    wait_for_connection(
        &mut eventloop,
        mqtt_config.connect_timeout,
        &mqtt_config.host,
        mqtt_config.port,
    )
    .await?;

    Ok((client, eventloop))
}

async fn wait_for_connection(
    eventloop: &mut rumqttc::EventLoop,
    timeout_secs: u64,
    host: &str,
    port: u16,
) -> Result<(), TelemetryError> {
    match timeout(TokioDuration::from_secs(timeout_secs), async {
        loop {
            match eventloop.poll().await {
                Ok(Event::Incoming(Packet::ConnAck(_))) => return Ok(()),
                Ok(_) => continue,
                Err(e) => return Err(TelemetryError::Mqtt(e.to_string())),
            }
        }
    })
    .await
    {
        Ok(Ok(())) => {
            info!("Connected to MQTT broker: {}:{}", host, port);
            Ok(())
        }
        Ok(Err(e)) => Err(e),
        Err(_) => Err(TelemetryError::Mqtt("Connection timed out".into())),
    }
}

async fn subscribe_to_topic(client: &AsyncClient, topic: &str) -> Result<(), TelemetryError> {
    info!("Subscribed to topic: {}", topic);
    client
        .subscribe(topic, QoS::AtMostOnce)
        .await
        .map_err(|e| io_error(e.to_string()))
}

fn spawn_processor(
    mut rx: mpsc::Receiver<Vec<(String, i64, Vec<(String, Value)>)>>,
    config: ProcessingConfig,
    output_writer: Arc<dyn OutputWriter>,
    sink_writer: Option<Arc<dyn SinkWriter>>,
    buffer_size: usize,
) -> JoinHandle<()> {
    tokio::spawn(async move {
        let mut buf = Vec::new();
        let max_wait = TokioDuration::from_secs(5);
        let timeout = sleep(max_wait);
        tokio::pin!(timeout);

        let semaphore = DynamicSemaphore::new(4);

        // tokio::spawn(async {
        //     loop {
        //         info!("teleflow processor still running...");
        //         sleep(TokioDuration::from_secs(5)).await;
        //     }
        // });

        loop {
            tokio::select! {
                biased;

                maybe_batch = rx.recv() => {
                    match maybe_batch {
                        Some(batch) => {
                            buf.extend(batch);
                            if buf.len() >= buffer_size {
                                process_batch(buf.split_off(0), config.clone(), output_writer.clone(), sink_writer.clone(), semaphore.clone()).await;
                                timeout.as_mut().reset(Instant::now() + max_wait);
                            }
                        }
                        None => {
                            warn!("MQTT input channel closed — exiting processor loop");
                            break;
                        }
                    }
                }

                () = &mut timeout => {
                    if !buf.is_empty() {
                        process_batch(buf.split_off(0), config.clone(), output_writer.clone(), sink_writer.clone(), semaphore.clone()).await;
                    }
                    timeout.as_mut().reset(Instant::now() + max_wait);
                }
            }
        }
    })
}

fn parse_payload(payload: &[u8]) -> Option<(String, i64, Vec<(String, Value)>)> {
    match serde_json::from_slice::<Value>(payload) {
        Ok(val) => {
            let device_id = val.get("device_id")?.as_str()?.to_string();
            let timestamp = val.get("timestamp")?.as_i64()?;
            let values = val
                .as_object()?
                .iter()
                .filter(|(k, _)| *k != "device_id" && *k != "timestamp")
                .map(|(k, v)| (k.clone(), v.clone()))
                .collect::<Vec<_>>();

            Some((device_id, timestamp, values))
        }
        Err(e) => {
            warn!("Failed to parse JSON payload: {}", e);
            None
        }
    }
}

async fn send_to_buffer(
    tx: &mpsc::Sender<Vec<(String, i64, Vec<(String, Value)>)>>,
    parsed: (String, i64, Vec<(String, Value)>),
) -> Result<(), TelemetryError> {
    tx.send(vec![parsed])
        .await
        .map_err(|e| io_error(e.to_string()))
}

fn build_dataframe(
    buf: &[(String, i64, Vec<(String, Value)>)],
) -> Result<DataFrame, TelemetryError> {
    let n_rows = buf.len();

    let device_ids: Vec<_> = buf.iter().map(|r| r.0.clone()).collect();
    let timestamps: Vec<_> = buf.iter().map(|r| r.1).collect();

    let mut df = df![
        "device_id" => device_ids,
        "timestamp" => timestamps
    ]
    .map_err(TelemetryError::Polars)?;

    let mut numeric_columns: HashMap<&str, Vec<Option<f64>>> = HashMap::new();
    let mut string_columns: HashMap<&str, Vec<Option<String>>> = HashMap::new();
    let mut all_columns = HashSet::new();

    for (_, _, values) in buf {
        for (k, _) in values {
            all_columns.insert(k.as_str());
        }
    }

    for &col in &all_columns {
        numeric_columns.insert(col, vec![None; n_rows]);
        string_columns.insert(col, vec![None; n_rows]);
    }

    for (i, (_, _, values)) in buf.iter().enumerate() {
        for (k, v) in values {
            let key = k.as_str();
            if let Some(val) = v.as_f64() {
                if let Some(col) = numeric_columns.get_mut(key) {
                    col[i] = Some(val);
                }
            } else if let Some(val) = v.as_str() {
                if let Some(col) = string_columns.get_mut(key) {
                    col[i] = Some(val.to_string());
                }
            }
        }
    }

    for (col, vals) in numeric_columns {
        if vals.iter().any(|v| v.is_some()) {
            let s = Series::new(PlSmallStr::from(col), vals);
            df.with_column(s).map_err(TelemetryError::Polars)?;
        }
    }

    for (col, vals) in string_columns {
        if vals.iter().any(|v| v.is_some()) {
            let s = Series::new(PlSmallStr::from(col), vals);
            df.with_column(s).map_err(TelemetryError::Polars)?;
        }
    }

    Ok(df)
}

fn io_error(msg: String) -> TelemetryError {
    TelemetryError::Io(std::io::Error::new(std::io::ErrorKind::Other, msg))
}

async fn process_batch(
    buf: Vec<(String, i64, Vec<(String, Value)>)>,
    config: ProcessingConfig,
    output_writer: Arc<dyn OutputWriter>,
    sink_writer: Option<Arc<dyn SinkWriter>>,
    semaphore: DynamicSemaphore,
) {
    let df = match build_dataframe(&buf) {
        Ok(df) => df,
        Err(e) => {
            warn!("Failed to build DataFrame: {:?}", e);
            return;
        }
    };

    let permit = semaphore.acquire().await;
    let batch_size = buf.len();
    let available = semaphore.available();

    tokio::spawn(async move {
        let start = Instant::now();
        if let Err(e) = process_telemetry(df, config, output_writer, sink_writer).await {
            warn!("Error in processing batch: {:?}", e);
        } else {
            info!(
                "Processed batch of {} records in {:.3?} (permits available: {})",
                batch_size,
                start.elapsed(),
                available
            );
        }
        drop(permit);
        semaphore.batch_completed().await;
    });
}
