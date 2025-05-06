# 📘 Teleflow Documentation

**Teleflow** is a high-performance telemetry processing engine built in Rust.

It collects, filters, transforms, and exports real-time telemetry data over MQTT. Designed for speed and flexibility, it supports batching, rolling aggregations, file outputs, and streaming to external sinks like InfluxDB.

---

## 🚀 Features

- ⚡ Fast, memory-efficient telemetry ingestion
- 🔌 MQTT-based real-time pipeline
- 📊 Batch processing with dynamic parallelism
- 🧮 Filters and rolling window transformations
- 💾 File export: Parquet, CSV, JSON
- 🌐 Sink integration: HTTP, InfluxDB, custom
- 🐳 Docker-ready deployment
- 📚 YAML-based configuration

---

## 📚 Documentation Sections

| Section                        | Description                                          |
|--------------------------------|------------------------------------------------------|
| [Overview & Architecture](architecture.md)  | Learn how Teleflow works internally               |
| [Configuration Reference](config.md)         | YAML structure and quick field reference          | |
| [CLI Reference](cli.md)                      | Commands, flags, and usage examples               |
| [Running with Docker](docker.md)             | Building and deploying using Docker               |

---

## 🧰 Quick Start

```bash
# Clone and build
git clone https://github.com/AsymptotaX/teleflow
cd teleflow
cargo build --release

# Or run in Docker
docker build -t teleflow .
docker run --rm -v $PWD/config.yaml:/app/config.yaml teleflow
```

---

## 📎 See Also

- [Project Repository](https://github.com/AsymptotaX/teleflow)
- [Issues](https://github.com/AsymptotaX/teleflow/issues)
- [Releases](https://github.com/AsymptotaX/teleflow/releases)