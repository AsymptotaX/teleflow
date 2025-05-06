# Configuration Guide (`config.yaml`)

Teleflow is configured using a single YAML file that defines input (MQTT), output, sinks, and processing logic.

---

## 📌 General Structure

```yaml
columns:
  device_id: device_id
  timestamp: timestamp

filters:
  - column: status
    operator: eq
    value: OK

rolling: 10

mqtt:
  host: ...
  port: ...
  buffer_size: ...
  eventloop_buffer_size: ...

output:
  enabled: true
  ...

sink:
  enabled: false
  ...
```

---

## 🧱 Column Mapping

```yaml
columns:
  device_id: device_id      # Required. Column containing device ID
  timestamp: timestamp      # Required. Column containing UNIX timestamp
```

These define how incoming telemetry maps to internal fields.

---

## 🔍 Filters

`filters` is an optional list of rules applied to incoming data before processing.

### 🔤 Structure

```yaml
filters:
  - column: <name>
    operator: <eq|ne|gt|lt|ge|le>
    value: <comparison value>
```

### 🔧 Supported Operators

| Operator | Description             | Example        |
|----------|-------------------------|----------------|
| `eq`     | Equals                  | `value: OK`    |
| `ne`     | Not equal               | `value: NOK`   |
| `gt`     | Greater than            | `value: 50`    |
| `lt`     | Less than               | `value: 10`    |
| `ge`     | Greater than or equal  | `value: 3.5`   |
| `le`     | Less than or equal     | `value: 0.01`  |

Use these to exclude irrelevant data early in the pipeline.

---

## 🔁 Rolling

```yaml
rolling: 10
```

Apply rolling mean to numeric columns. Window size is specified in rows.

---

## 📡 MQTT Section

```yaml
mqtt:
  host: "your-mqtt-host"
  port: 8883
  topic: "telemetry/#"
  use_tls: true
  username: "user"
  password: "pass"
  keep_alive: 30
  connect_timeout: 60
  client_id: null

  buffer_size: 10000               # Channel size (between MQTT and processor)
  eventloop_buffer_size: 15000     # MQTT client's internal buffer size
```

- `topic` may include wildcards like `+` and `#`
- TLS is enabled with `use_tls: true`
- `client_id` is optional (auto-generated if null)

---

## 💾 Output Section

```yaml
output:
  enabled: true
  format: parquet        # csv, parquet, json
  path: ./output
  compression: zstd      # only for parquet
  batch_naming: timestamp
  max_batches: 100
  min_disk_space_gb: 1.0
```

- Files are saved in batches
- Auto-cleanup if disk space or batch count is exceeded

---

## 🌐 Sink Section

```yaml
sink:
  enabled: true
  type: http
  endpoint: http://...
  auth_token: your-token
  format: influx         # influx or json
  org: my-org
  bucket: teleflow
  precision: ns
  buffer_size: 10000
  eventloop_buffer_size: 15000
```

If enabled, telemetry is streamed to an HTTP/InfluxDB-compatible endpoint.

---

## ✅ Minimal Example

```yaml
mqtt:
  enabled: true
  host: localhost
  port: 1883
  topic: telemetry/#
  buffer_size: 10000
  eventloop_buffer_size: 15000

output:
  enabled: true
  format: parquet
  path: ./output
```