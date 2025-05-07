# Teleflow

**Teleflow** is a high-performance, lightweight telemetry pipeline written in Rust.  
It supports MQTT and static file ingestion, filtering, batching, and output to file or external sinks like InfluxDB.

---

## Quick Start

### 1. Build from source

```bash
cargo build --release
```

### 2. Or use Docker

```bash
docker build -t teleflow .
```

### 3. Run with MQTT input

```bash
docker run --rm \
  -v $PWD/your_config.yaml:/app/config.yaml \
  teleflow process-mqtt --config /app/config.yaml
```

### 4. Run with static file input

```bash
docker run --rm \
  -v $PWD/your_config.yaml:/app/config.yaml \
  teleflow process --config /app/config.yaml
```

---

## Additional Options

- Enable logging:

```bash
docker run -e RUST_LOG=debug ...
```

- Run in background:

```bash
docker run -d --name teleflow ...
```

---

## Full Documentation

For full configuration reference, architecture, and performance benchmarks, visit:

👉 [https://teleflow.readthedocs.io](https://teleflow.readthedocs.io)

---

## License

MIT License

> Built for speed. Designed for simplicity. Ready for production. 🚀