# CLI

Teleflow provides several subcommands for processing and testing telemetry pipelines.

---

## `process`

Run a batch processing job from a static source (e.g., file or test data). Useful for development and debugging.

```bash
teleflow process --input data.parquet --config config.yaml
```

- Reads telemetry from configured sources
- Applies all filters and output logic
- Does not require a live MQTT connection

---

## `process-mqtt`

Start a live pipeline that listens to a MQTT broker and processes incoming telemetry in real time.

```bash
teleflow process-mqtt --config config.yaml --buffer-size 10000 --eventloop-buffer-size 15000
```

- Connects to MQTT broker
- Batches messages and sends them to processing
- Supports adaptive concurrency

---

## `generate-test`

Generate synthetic telemetry and write it to a file.

```bash
teleflow generate-test --rows 100 --devices 10 --output testdata.parquet
```

- Great for testing dashboards, sink integrations, and benchmarking
- Output is saved in Parquet format
- Each row includes a synthetic signal with a unified `value` field

### Example output:

| device_id | timestamp   | status | signal_type | value               |
|-----------|-------------|--------|-------------|---------------------|
| dev-8     | 1625660871  | OK     | temp        | 22.80067840233416   |
| dev-1     | 1625622402  | OK     | voltage     | 3.626076362720073   |
| dev-6     | 1625804790  | ERROR  | temp        | 29.54769961012679   |
| dev-1     | 1626073168  | ERROR  | voltage     | 3.701339785285878   |

- `signal_type` defines the meaning of `value`
- Only one `value` column is used to simplify processing and reduce ambiguity

---

## Global Options

| Option                       | Description                                               |
|------------------------------|-----------------------------------------------------------|
| `--config <path>`            | Path to your configuration YAML file                      |
| `--buffer-size <usize>`      | Internal buffer for batching telemetry rows               |
| `--eventloop-buffer-size`    | MQTT client event buffer (in-flight messages)             |
| `--stdout`                   | Force output to stdout (only if `output.enabled = false`) |

**Important:** Global flags like `--stdout` must be passed **before** the subcommand:
```bash
teleflow --stdout process --input testdata.parquet --config config.yaml
```

Use `--help` after any subcommand for detailed options:

```bash
teleflow process --help
teleflow process-mqtt --help
teleflow generate-test --help
```