# Stage 1: Build Teleflow from source
FROM rust:1.86-slim AS builder

WORKDIR /app

# Copy project files (excluding those in .dockerignore, e.g., target/)
COPY . .

RUN cargo build --release

# Stage 2: Runtime image
FROM debian:bullseye-slim

# Add runtime dependencies (if needed)
RUN apt-get update && apt-get install -y ca-certificates && rm -rf /var/lib/apt/lists/*

WORKDIR /app
COPY --from=builder /app/target/release/teleflow /usr/local/bin/teleflow

# Default command: MQTT processing with config.yaml
ENTRYPOINT ["teleflow"]
CMD ["process-mqtt", "--config", "config.yaml"]