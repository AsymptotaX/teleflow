# CLI Reference

Teleflow provides several subcommands for processing and testing telemetry pipelines.

---

## 🧪 `process`

Run a batch processing job from a static source (e.g., file or test data). Useful for development and debugging.

```bash
teleflow process --config config.yaml
```

- Reads telemetry from configured sources
- Applies all filters and output logic
- Does not require a live MQTT connection

---

## 📡 `process-mqtt`

Start a live pipeline that listens to a MQTT broker and processes incoming telemetry in real time.

```bash
teleflow process-mqtt --config config.yaml --buffer-size 10000 --eventloop-buffer-size 15000
```

- Connects to MQTT broker
- Batches messages and sends them to processing
- Supports adaptive concurrency

---

## 🧰 `generate-test`

Generate synthetic telemetry and write it to file using the configured output settings.

```bash
teleflow generate-test --config config.yaml
```

- Great for testing dashboards, sink connections, and local pipelines
- Data is deterministic and reproducible

---

## ⚙️ Global Options

| Option                       | Description                                      |
|------------------------------|--------------------------------------------------|
| `--config <path>`            | Path to your configuration YAML file            |
| `--buffer-size <usize>`      | Internal buffer for batching telemetry rows     |
| `--eventloop-buffer-size`    | MQTT client event buffer (in-flight messages)   |

Use `--help` after any subcommand for detailed options:

```bash
teleflow process-mqtt --help
```