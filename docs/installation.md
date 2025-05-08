# Installation Instructions

This guide outlines all the ways you can install and run **Teleflow**, depending on your environment and preferences.

---

## Option 1: Download from GitHub Releases

You can find precompiled binaries for major platforms on the [Releases](https://github.com/AsymptotaX/teleflow/releases) page.
Choose the appropriate binary for your OS and architecture and download it.

### Windows:

```bash
# unzip the archive and try to run the binary
teleflow.exe --help
```

### Linux/macOS:

```bash
unzip teleflow*.zip
chmod +x teleflow
./teleflow --help
```

---

## Option 2: Use Docker (Recommended)

You can use the official Docker image to avoid building or downloading binaries manually.

### From Docker Hub

```bash
docker pull asymptotax/teleflow:latest
```

Then run:

```bash
docker run --rm asymptotax/teleflow --help
```

### Or build manually

```bash
git clone https://github.com/AsymptotaX/teleflow
cd teleflow
docker build -t teleflow .
```

---

## Option 3: Build from Source (Rust)

Requires Rust toolchain (via https://rustup.rs):

```bash
git clone https://github.com/AsymptotaX/teleflow
cd teleflow
cargo build --release
./target/release/teleflow --help
```

---

## Verification

After installing, try:

```bash
teleflow --help
```

You should see available commands like `process`, `process-mqtt`, and `generate-test`.