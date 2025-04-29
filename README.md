
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
mqtt:
  host: "localhost"
  port: 1883
  topic: "telemetry/#"
  client_id: "teleflow"

processing:
  output:
    format: "parquet"      # Options: parquet | csv | json
    path: "./output"
    compression: "zstd"    # For Parquet only
    max_batches: 1000       # Max batch files before cleanup
    min_disk_space_gb: 5    # Minimum free disk space

sink:
  type: "http"              # Sink type: http
  endpoint: "http://influxdb:8086/api/v2/write"
  auth_token: "your-token"
  org: "my-org"
  bucket: "teleflow"
  precision: "ns"
```

---

## Typical Architecture

```text
[ MQTT Broker ]  -->  [ Teleflow ]  -->  [ Parquet / CSV Files ]
                                      \-->  [ InfluxDB / Grafana ]
```

---

## Highlights

- 🧮 **Dynamic batching** based on MQTT load
- 🕐 **Real timestamps** in incoming data
- 📣 **Retry strategy** on HTTP failures (exponential backoff)
- 🏡 **Minimal resource usage**, optimized for embedded / VPS deployments
- 💡 **Designed for easy extensions** (Kafka sink, MQTT sink in future)

---

## Grafana Integration

1. Point it to your InfluxDB bucket `teleflow`
2. Enjoy live device metrics and stats!

---

## Roadmap

- 📉 Kafka sink support
- ⚖️ Comprehensive integration tests
- 💡 Auto-scaling concurrency

---

## License

MIT License

---

> **Teleflow** — Built for speed. Designed for simplicity. Ready for production. 🚀
