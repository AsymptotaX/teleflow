# Running Teleflow with Docker

Teleflow can be run as a self-contained Docker container without installing Rust or dependencies on your host system.

---

## Build the Docker Image

```bash
docker build -t teleflow .
```

This will compile the Teleflow binary and produce a minimal runtime container.

---

## Run with a Config File

Since the image does not include a configuration file, you need to mount it manually:

Start Teleflow with MQTT input
```bash
docker run --rm \
  -v $PWD/your_config.yaml:/app/config.yaml \
  teleflow process-mqtt --config /app/config.yaml

```

Start Teleflow to process a static input file
```bash
docker run --rm \
  -v $PWD/your_config.yaml:/app/config.yaml \
  teleflow process --config /app/config.yaml
```

Enable debug-level logging using RUST_LOG
```bash
docker run --rm \
  -v $PWD/your_config.yaml:/app/config.yaml \
  -e RUST_LOG=debug \
  teleflow process-mqtt --config /app/config.yaml
```

Run container in background mode (-d)
```bash
docker run -d \
  -v $PWD/your_config.yaml:/app/config.yaml \
  --name teleflow \
  teleflow process-mqtt --config /app/config.yaml
```

---

## Notes

- Ensure your MQTT and sink endpoints are accessible from inside the container.
- If you're using secure brokers or InfluxDB over HTTPS, configure TLS and authentication fields in `config.yaml`.
- Use volume mounting (`-v`) to collect output files on the host system.

---

## Example with Output Volume

```bash
docker run --rm -v $PWD/config.yaml:/app/config.yaml -v $PWD/output:/app/output teleflow
```

This will store output files in the `output/` directory on your host.