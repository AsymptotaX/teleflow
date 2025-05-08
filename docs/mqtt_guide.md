# MQTT Guide

In this guide, you'll learn how to connect Teleflow to a live MQTT broker and stream telemetry data in real-time.

---

## 1. MQTT Configuration

Add the following section to your `config.yaml`:

```yaml
mqtt:
  host: "your-broker.com"
  port: 1883
  topic: "telemetry/#"
  username: "user"          # optional
  password: "pass"          # optional
  use_tls: false
  client_id: null
  keep_alive: 10
  connect_timeout: 5
  buffer_size: 10000
  eventloop_buffer_size: 15000
```

This tells Teleflow how to connect to your broker and how much buffering to allow.

---

## 2. Run the MQTT Processor

### With binary:

```bash
teleflow process-mqtt --config config.yaml
```

### With Docker:

```bash
docker run --rm \
  -v $PWD/config.yaml:/config.yaml \
  teleflow process-mqtt --config /config.yaml
```

---

## 3. Output to InfluxDB (Sink)

To push incoming messages to InfluxDB or another endpoint:

```yaml
sink:
  enabled: true
  type: "http"
  endpoint: "https://your-influxdb-endpoint"
  auth_token: "your-token"
  format: "influx"
  org: "my-org"
  bucket: "teleflow"
  precision: "ns"
  buffer_size: 10000
  eventloop_buffer_size: 15000
```

You can also disable the file output if you're using a sink only:

```yaml
output:
  enabled: false
```

---

## 4. Filtering Live Data

You can apply the same filter rules to MQTT streams:

```yaml
filters:
  - signal_type: temp
  - status: ERROR
```

---

## Result

Your device will push JSON like:

```json
{
  "device_id": "dev-9",
  "timestamp": 1625576753,
  "signal_type": "voltage",
  "value": 3.65,
  "status": "OK"
}
```

Teleflow will ingest, filter, and route it — in real-time 🚀