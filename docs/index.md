# Teleflow Documentation

**Teleflow** is a high-performance telemetry processing engine built in Rust.

It collects, filters, transforms, and exports real-time telemetry data over MQTT. Designed for speed and flexibility, it supports batching, rolling aggregations, file outputs, and streaming to external sinks like InfluxDB.

---

## Features

- ⚡ Fast, memory-efficient telemetry ingestion
- 🔌 MQTT-based real-time pipeline
- 📊 Batch processing with dynamic parallelism
- 🧮 Filters and rolling window transformations
- 💾 File export: Parquet, CSV, JSON
- 🌐 Sink integration: HTTP, InfluxDB, custom
- 🐳 Docker-ready deployment
- 📚 YAML-based configuration

---

## Documentation Sections

```{toctree}
:maxdepth: 2
:caption: Getting Started
installation
static_file_guide
mqtt_guide
```

```{toctree}
:maxdepth: 2
:caption: Configuration
config
cli
```

```{toctree}
:maxdepth: 2
:caption: Reference
architecture
benchmark
changelog

```

---

## Quick Start

```bash
# Clone and build
git clone https://github.com/AsymptotaX/teleflow
cd teleflow
cargo build --release

# Or run in Docker
docker build -t teleflow .
docker run --rm \
  -v $PWD/your_config.yaml:/app/config.yaml \
  teleflow process-mqtt --config /app/config.yaml
```

---

## See Also

- [Project Repository](https://github.com/AsymptotaX/teleflow)
- [Issues](https://github.com/AsymptotaX/teleflow/issues)
- [Releases](https://github.com/AsymptotaX/teleflow/releases)