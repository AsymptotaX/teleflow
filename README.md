# Teleflow

A lightweight, high-performance telemetry ingestion pipeline. Designed to be simple, modular, and production-ready.

---

## Features

- 📈 **Batching and Filtering** of incoming MQTT messages
- 🔗 **Flexible Output**:
  - Save as **Parquet** / **CSV** / **JSON** files
  - Stream data directly to **InfluxDB** (HTTP Sink)
- ⚡ **Automatic retry** mechanism for reliable HTTP delivery
- 🛡️ **Graceful shutdown** on Ctrl+C (safe exit)
- 🔄 **Dynamic semaphore** controls concurrency
- ✅ **Auto-cleanup** of old batch files
- 🔍 **Config-driven** architecture
- 🔄 **Realtime dashboards** via Grafana

---

## Requirements

- **Rust** (1.74+)
- **Docker** (for InfluxDB and Grafana)
- **MQTT Broker** (e.g., Mosquitto)
- **Grafana** (optional, for visualization)

---

## Building

```bash
cargo build --release
```

---

## Running

```bash
# Process telemetry from MQTT
teleflow process-mqtt --config ./config.yaml

# Process a static Parquet file
teleflow process --input ./yourfile.parquet --config ./config.yaml

# Generate test data
teleflow generate-test --rows 10000 --devices 50 --output ./test.parquet
```

---

## Configuration Example (`config.yaml`)

```yaml
# Column mappings for telemetry data
columns:
  device_id: device_id
  timestamp: timestamp

# Filters to apply on incoming data
filters:
  - column: status
    operator: eq
    value: OK

  - column: signal_type
    operator: eq
    value: voltage

  - column: temp
    operator: gt
    value: 22.0

# Rolling window size for telemetry calculations
rolling: 10

# MQTT Configuration
mqtt:
  host: "broker.example.com"
  port: 8883
  use_tls: true
  username: "user"
  password: "pass"
  topic: "telemetry/#"
  keep_alive: 30
  connect_timeout: 60
  client_id: null
  buffer_size: 10000
  eventloop_buffer_size: 9000

# Output batch file config
output:
  enabled: false
  format: "parquet"
  path: "./output"
  compression: "zstd"
  batch_naming: "timestamp"
  max_batches: 100
  min_disk_space_gb: 1.0

# Sink config for streaming to InfluxDB, Kafka, etc.
sink:
  enabled: false
  type: "http"
  endpoint: "http://influxdb:8086/api/v2/write"
  auth_token: "your-token"
  format: "influx"
  org: "my-org"
  bucket: "teleflow"
  precision: "ns"
  buffer_size: 10000
  eventloop_buffer_size: 15000
```

---

## Architecture Overview

```
[ MQTT Broker ]  -->  [ Teleflow ]  -->  [ Parquet / CSV / JSON Files ]
                                      \-->  [ InfluxDB / Grafana ]
```

---

## Highlights

- 🧮 Dynamic batching based on MQTT throughput
- 🔁 Automatic backoff and retry logic for sinks
- ✂️ Real-time filtering and column mapping
- 🏡 Minimal dependencies and memory usage
- 🚀 Ready for edge, embedded, and production deployment

---

## Grafana Integration

1. Connect Grafana to your InfluxDB bucket (`teleflow`)
2. Use queries like `from(bucket: "teleflow") |> range(start: -1h)`
3. Build live dashboards with device metrics

---

## Roadmap

- Kafka sink support
- Full e2e integration tests
- gRPC sink support

---

## 🧪 Benchmark Results

### 🖥️ Test System

| Component   | Specification                                |
|-------------|----------------------------------------------|
| CPU         | 11th Gen Intel® Core™ i7-11850H @ 2.50 GHz   |
| RAM         | 32 GB DDR4                                   |
| OS          | Windows 11                                   |
| Broker      | HiveMQ Cloud (TLS, port 8883)                |
| Teleflow    | Built with `cargo build --release`           |
| Storage     | *(not applicable — InfluxDB writes disabled)*|

📌 **Note**: Benchmark results were measured **without synchronization to InfluxDB**, to avoid I/O bottlenecks and provide hardware-neutral performance data.

### 🚀 Teleflow Performance (in-memory processing only)

| Batch Size | Telemetry Processing Time | Total Time (MQTT + Telemetry) |
|------------|---------------------------|--------------------------------|
| 26,625     | 2.223 ms                  | 2.766 ms                       |
| 44,943     | 2.159 ms                  | 3.011 ms                       |
| 29,605     | 1.743 ms                  | 2.608 ms                       |
| 21,094     | 1.690 ms                  | 2.509 ms                       |

### 📈 Effective Throughput

- Peaks of **10+ million rows/second** processed in-memory.
- Stable performance with dynamic semaphore limiting and batching enabled.
- Parallelism adjusted adaptively.
             
---

## License

MIT License

> **Teleflow** — Built for speed. Designed for simplicity. Ready for production. 🚀