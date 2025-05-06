# 🐳 Running Teleflow with Docker

Teleflow can be run as a self-contained Docker container without installing Rust or dependencies on your host system.

---

## 🧱 Build the Docker Image

```bash
docker build -t teleflow .
```

This will compile the Teleflow binary and produce a minimal runtime container.

---

## 🚀 Run with a Config File

You must mount your own `config.yaml` into the container:

```bash
docker run --rm -v $PWD/config.yaml:/app/config.yaml teleflow
```

By default, this runs the `process-mqtt` command.

---

## 🛠 Overriding the Default Command

You can override the default command using Docker CLI arguments:

```bash
docker run --rm -v $PWD/config.yaml:/app/config.yaml teleflow process --config config.yaml
```

This allows you to use `process`, `generate-test`, or any other supported CLI option.

---

## 🔒 Notes

- Ensure your MQTT and sink endpoints are accessible from inside the container.
- If you're using secure brokers or InfluxDB over HTTPS, configure TLS and authentication fields in `config.yaml`.
- Use volume mounting (`-v`) to collect output files on the host system.

---

## 🧪 Example with Output Volume

```bash
docker run --rm -v $PWD/config.yaml:/app/config.yaml -v $PWD/output:/app/output teleflow
```

This will store output files in the `output/` directory on your host.