# Static File Processing Guide

If you already have telemetry data and want to filter it, print it to the screen, or save it to a file — this application makes it easy, and processes your data at **fantastic speeds**.

Let’s take a look at how it works using test data, with examples for both the prebuilt binary and Docker usage.       

This guide walks you through using **Teleflow** in two scenarios:

1. Using the prebuilt release (binary)
2. Using Docker

---

## 1. Generate test data

### Using binary

```bash
teleflow.exe generate-test --rows 100 --devices 10 --output data/testdata.parquet
```

### Using Docker

```bash
docker run --rm -v $PWD/data:/data teleflow generate-test --rows 100 --devices 10 --output /data/testdata.parquet
```

### Sample output:

| device_id | timestamp   | status | signal_type | value               |
|-----------|-------------|--------|-------------|---------------------|
| dev-8     | 1625660871  | OK     | temperature | 22.80067840233416   |
| dev-1     | 1625622402  | OK     | voltage     | 3.626076362720073   |
| dev-5     | 1625915930  | ERROR  | voltage     | 3.626076362720073   |
| dev-3     | 1625718516  | OK     | temperature | 25.839844508857286  |
| dev-10    | 1625350914  | ERROR  | voltage     | 3.626076362720073   |

---

## 2. Filter

Let’s filter the data using multiple rules — keeping only rows where: 
 - signal_type == temperature 
 - status == OK

We'll also disable file output and show the filtered result in the console:

```yaml
# config.yaml
output:
  enabled: false
  path: "data/"
  format: "parquet"

filters:
  - column: signal_type
    operator: eq
    value: temperature

  - column: status
    operator: eq
    value: OK
```

---

## 3. Run and print to stdout

### Binary:

```bash
teleflow.exe --stdout process --input data/testdata.parquet --config config.yaml
```

#### Optional: Enable debug logs
To see detailed internal logs (useful for debugging), set the `RUST_LOG` environment variable:
```bash
set RUST_LOG=debug
```

### Docker:

```bash
docker run --rm \
  -e RUST_LOG=debug
  -v $PWD/data:/app/data \
  -v $PWD/config.yaml:/config.yaml \
  teleflow --stdout process --input /app/data/testdata.parquet --config /config.yaml
```

---

## 4. Output to file

Now let’s try saving the filtered results to a file instead of printing to the screen.

Set `output.enabled: true` in `config.yaml`:

Supported formats: `csv` | `parquet` | `json`

```yaml
output:
  enabled: true
  path: "data/"
  format: "parquet"
  batch_naming: "timestamp"
  compression: "zstd"
# ...
```

Then run the same `process` command. File will be saved to `data/`.

---

## 5. Run the command

### Binary:

```bash
teleflow.exe process --input data/testdata.parquet --config config.yaml
```

### Docker:

```bash
docker run --rm \
  -e RUST_LOG=debug
  -v $PWD/data:/app/data \
  -v $PWD/config.yaml:/config.yaml \
  teleflow process --input /app/data/testdata.parquet --config /config.yaml
```

The output file will be written to the directory specified in `config.yaml` (e.g. `data/`).

---

## That's it!

Now you know how to use Teleflow to process telemetry data from static files.  
Just plug in your own data file, adjust the filters in `config.yaml`, and you're ready to go!