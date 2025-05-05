use crate::error::TelemetryError;
use chrono::Utc;
use log::{info, warn};
use polars::prelude::*;
use std::fs::{self, File};
use std::path::Path;
use sysinfo::Disks;
use tokio::task;

/// Trait defining the behavior of an output writer.
/// Implementations of this trait are responsible for writing data batches to specific formats.
/// Trait defining the behavior of an output writer for telemetry data.
///
/// Implementations handle the writing of `DataFrame` batches
/// to various output formats (Parquet, CSV, JSON, etc.).
pub trait OutputWriter: Send + Sync {
    /// Write a batch of telemetry data.
    fn write_batch(&self, df: &DataFrame) -> Result<(), TelemetryError>;
}

/// Writer for Parquet files.
/// Handles writing data batches to Parquet format with optional compression and cleanup.
/// Output writer for Parquet files with optional compression and batch cleanup.
#[derive(Clone)]
pub struct ParquetFileWriter {
    output_dir: String,
    compression: ParquetCompression,
    max_batches: Option<usize>,
    min_disk_space_gb: Option<f64>,
}

/// Writer for CSV files.
/// Handles writing data batches to CSV format.
/// Output writer for CSV files.
pub struct CsvFileWriter {
    output_dir: String,
}

/// Writer for JSON files.
/// Handles writing data batches to JSON format.
/// Output writer for JSON files.
pub struct JsonFileWriter {
    output_dir: String,
}

impl ParquetFileWriter {
    /// Creates a new `ParquetFileWriter`.
    ///
    /// - `output_dir`: Directory to store output Parquet files.
    /// - `compression`: Compression type for Parquet output.
    /// - `max_batches`: Maximum number of batch files to keep (cleanup applies).
    /// - `min_disk_space_gb`: Minimum free space required on disk (cleanup applies).
    pub fn new(
        output_dir: String,
        compression: ParquetCompression,
        max_batches: Option<usize>,
        min_disk_space_gb: Option<f64>,
    ) -> Self {
        Self {
            output_dir,
            compression,
            max_batches,
            min_disk_space_gb,
        }
    }

    /// Generates a unique filename for a new Parquet file.
    /// Generates a unique filename for a new CSV file.
    fn generate_filename(&self) -> String {
        format!(
            "{}/batch_{}.parquet",
            self.output_dir.trim_end_matches('/'),
            Utc::now().format("%Y%m%d%H%M%S%3f")
        )
    }

    /// Synchronously writes a batch to a Parquet file.
    ///
    /// Logs file size, row count, and issues warnings for very small or very large files.
    pub fn write_batch_sync(&self, df: &DataFrame) -> Result<(), TelemetryError> {
        let filename = self.generate_filename();

        if let Some(parent) = Path::new(&filename).parent() {
            fs::create_dir_all(parent)?;
        }

        let mut file = File::create(&filename)?;
        let mut df = df.clone();

        ParquetWriter::new(&mut file)
            .with_statistics(StatisticsOptions::full())
            .with_compression(self.compression)
            .finish(&mut df)?;

        let metadata = fs::metadata(&filename)?;
        let file_size_mb = metadata.len() as f64 / 1_048_576.0; // 1MB = 1024*1024 bytes
        let num_rows = df.height();

        info!(
            "Created file {} ({} rows, {:.2} MB)",
            filename, num_rows, file_size_mb
        );

        if file_size_mb < 0.01 {
            warn!(
                "Created very small batch file {} ({:.4} MB)",
                filename, file_size_mb
            );
        }
        if file_size_mb > 50.0 {
            warn!(
                "Created very large batch file {} ({:.2} MB)",
                filename, file_size_mb
            );
        }

        Ok(())
    }

    /// Asynchronously cleans up old batch files based on `max_batches` and `min_disk_space_gb`.
    ///
    /// Runs in a background task via `tokio::spawn_blocking`.
    async fn cleanup_batches(self) -> Result<(), TelemetryError> {
        let output_dir = self.output_dir;
        let max_batches = self.max_batches;
        let min_disk_space_gb = self.min_disk_space_gb;

        task::spawn_blocking(move || {
            let output_path = Path::new(&output_dir);

            let mut files: Vec<_> = fs::read_dir(&output_dir)?
                .filter_map(|entry| entry.ok())
                .filter(|entry| entry.path().extension().map_or(false, |ext| ext == "parquet"))
                .collect();

            let mut deleted_files = 0;

            if let Some(max_batches) = max_batches {
                if files.len() > max_batches {
                    files.sort_by_key(|entry| {
                        entry.metadata()
                            .and_then(|meta| meta.modified())
                            .unwrap_or(std::time::SystemTime::UNIX_EPOCH)
                    });

                    let excess = files.len() - max_batches;
                    for entry in files.iter().take(excess) {
                        if let Err(e) = fs::remove_file(entry.path()) {
                            warn!("Failed to delete old batch file {:?}: {:?}", entry.path(), e);
                        } else {
                            deleted_files += 1;
                        }
                    }
                }
            }

            if let Some(min_gb) = min_disk_space_gb {
                let disks = Disks::new_with_refreshed_list();
                for disk in &disks {
                    if output_path.starts_with(disk.mount_point()) {
                        let free_space_gb = disk.available_space() as f64 / 1_073_741_824.0;
                        if free_space_gb < min_gb {
                            warn!(
                            "Low disk space: {:.2} GB free, target minimum {:.2} GB. Cleaning up...",
                            free_space_gb,
                            min_gb
                        );

                            files.sort_by_key(|entry| {
                                entry.metadata()
                                    .and_then(|meta| meta.modified())
                                    .unwrap_or(std::time::SystemTime::UNIX_EPOCH)
                            });

                            for entry in files.iter() {
                                if let Err(e) = fs::remove_file(entry.path()) {
                                    warn!("Failed to delete batch file {:?}: {:?}", entry.path(), e);
                                } else {
                                    deleted_files += 1;
                                }

                                let disks = Disks::new_with_refreshed_list();
                                for d in &disks {
                                    if output_path.starts_with(d.mount_point()) {
                                        let free_space_gb = d.available_space() as f64 / 1_073_741_824.0;
                                        if free_space_gb >= min_gb {
                                            break;
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            }

            if deleted_files > 0 {
                info!("🧹 Cleanup completed: deleted {deleted_files} old batch file(s).");
            }

            Ok::<_, TelemetryError>(())
        })
            .await
            .map_err(|e| TelemetryError::Join(format!("cleanup_batches: {e}")))?
    }
}

impl CsvFileWriter {
    /// Creates a new `ParquetFileWriter`.
    ///
    /// - `output_dir`: Directory to store output Parquet files.
    /// - `compression`: Compression type for Parquet output.
    /// - `max_batches`: Maximum number of batch files to keep (cleanup applies).
    /// - `min_disk_space_gb`: Minimum free space required on disk (cleanup applies).
    /// Creates a new `CsvFileWriter`.
    ///
    /// - `output_dir`: Directory to store CSV files.
    /// Creates a new `JsonFileWriter`.
    ///
    /// - `output_dir`: Directory to store JSON files.
    pub fn new(output_dir: String) -> Self {
        Self { output_dir }
    }

    /// Generates a unique filename for a new Parquet file.
    /// Generates a unique filename for a new CSV file.
    fn generate_filename(&self) -> String {
        format!(
            "{}/batch_{}.csv",
            self.output_dir.trim_end_matches('/'),
            Utc::now().format("%Y%m%d%H%M%S%3f")
        )
    }
}

impl JsonFileWriter {
    /// Creates a new `ParquetFileWriter`.
    ///
    /// - `output_dir`: Directory to store output Parquet files.
    /// - `compression`: Compression type for Parquet output.
    /// - `max_batches`: Maximum number of batch files to keep (cleanup applies).
    /// - `min_disk_space_gb`: Minimum free space required on disk (cleanup applies).
    /// Creates a new `CsvFileWriter`.
    ///
    /// - `output_dir`: Directory to store CSV files.
    /// Creates a new `JsonFileWriter`.
    ///
    /// - `output_dir`: Directory to store JSON files.
    pub fn new(output_dir: String) -> Self {
        Self { output_dir }
    }

    /// Generates a unique filename for a new Parquet file.
    /// Generates a unique filename for a new CSV file.
    fn generate_filename(&self) -> String {
        format!(
            "{}/batch_{}.json",
            self.output_dir.trim_end_matches('/'),
            Utc::now().format("%Y%m%d%H%M%S%3f")
        )
    }
}

impl OutputWriter for ParquetFileWriter {
    fn write_batch(&self, df: &DataFrame) -> Result<(), TelemetryError> {
        self.write_batch_sync(df)?;

        let writer_clone = self.clone();
        tokio::spawn(async move {
            if let Err(e) = writer_clone.cleanup_batches().await {
                warn!("Cleanup batches failed: {:?}", e);
            }
        });

        Ok(())
    }
}

impl OutputWriter for CsvFileWriter {
    fn write_batch(&self, df: &DataFrame) -> Result<(), TelemetryError> {
        let filename = self.generate_filename();

        if let Some(parent) = Path::new(&filename).parent() {
            fs::create_dir_all(parent)?;
        }

        let mut file = File::create(&filename)?;
        let mut df = df.clone();

        CsvWriter::new(&mut file)
            .include_header(true)
            .finish(&mut df)
            .map_err(TelemetryError::Polars)?;

        let metadata = fs::metadata(&filename)?;
        let file_size_mb = metadata.len() as f64 / 1_048_576.0;
        let num_rows = df.height();

        info!(
            "Created CSV file {} ({} rows, {:.2} MB)",
            filename, num_rows, file_size_mb
        );

        Ok(())
    }
}

impl OutputWriter for JsonFileWriter {
    fn write_batch(&self, df: &DataFrame) -> Result<(), TelemetryError> {
        let filename = self.generate_filename();

        if let Some(parent) = Path::new(&filename).parent() {
            fs::create_dir_all(parent)?;
        }

        let mut file = File::create(&filename)?;
        let mut df = df.clone();

        JsonWriter::new(&mut file)
            .with_json_format(JsonFormat::JsonLines)
            .finish(&mut df)
            .map_err(TelemetryError::Polars)?;

        let metadata = fs::metadata(&filename)?;
        let file_size_mb = metadata.len() as f64 / 1_048_576.0;
        let num_rows = df.height();

        info!(
            "Created JSON file {} ({} rows, {:.2} MB)",
            filename, num_rows, file_size_mb
        );

        Ok(())
    }
}

pub fn get_parquet_compression(compression: Option<String>) -> ParquetCompression {
    if let Some(comp) = compression {
        let lower = comp.to_lowercase();
        match lower.as_str() {
            "zstd" => ParquetCompression::Zstd(None),
            "snappy" => ParquetCompression::Snappy,
            "none" => ParquetCompression::Uncompressed,
            _ => ParquetCompression::Snappy,
        }
    } else {
        ParquetCompression::Snappy
    }
}

/// Output writer that performs no action (noop).
///
/// Used when `output.enabled = false` in config.
pub struct NoopWriter;

impl OutputWriter for NoopWriter {
    /// Does nothing. Always returns `Ok(())`.
    fn write_batch(&self, _df: &DataFrame) -> Result<(), TelemetryError> {
        Ok(())
    }
}
