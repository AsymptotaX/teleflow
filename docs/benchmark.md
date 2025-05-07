# Benchmark Results

These benchmarks were recorded on the following machine:

**System Info**  
- CPU: 11th Gen Intel(R) Core(TM) i7-11850H @ 2.50GHz  
- RAM: 32 GB  
- OS: Windows 11

---

## Performance Highlights

- ⚡ Peaks of **10+ million rows/second** processed in-memory
- 🧠 Stable performance with dynamic semaphore limiting and adaptive batching
- 🔁 Parallelism is automatically tuned based on processing load

These results showcase Teleflow's ability to handle extremely high-throughput telemetry with minimal resource usage — ideal for embedded, edge, and cloud environments.

---

## Raw Benchmark Sample (with sinks disabled)

| Batch Size | Telemetry Processing Time | Total Time (MQTT + Telemetry) |
|------------|---------------------------|-------------------------------|
| 26,625     | 2.223 ms                  | 2.766 ms                      |
| 44,943     | 2.159 ms                  | 3.011 ms                      |
| 29,605     | 1.743 ms                  | 2.608 ms                      |
| 21,094     | 1.690 ms                  | 2.509 ms                      |

⚠️ These results were collected with `sink.enabled = false` and `output.enabled = false` for maximum raw throughput measurement.

---

## Factors That Affect Performance

- Output sinks (InfluxDB, file I/O)
- Network/disk latency
- Static vs real-time ingestion
- Internal buffer sizes and semaphore settings