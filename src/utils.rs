use polars::prelude::*;
use rand::Rng;

/// Generate test telemetry data as a DataFrame.
///
/// # Arguments
///
/// * `n` - Number of rows to generate
/// * `num_devices` - Number of unique device IDs
pub fn generate_test_data(n: usize, num_devices: usize) -> Result<DataFrame, PolarsError> {
    let mut rng = rand::rng();

    let mut device_ids = Vec::with_capacity(n);
    let mut timestamps = Vec::with_capacity(n);
    let mut statuses = Vec::with_capacity(n);
    let mut signal_types = Vec::with_capacity(n);
    let mut values = Vec::with_capacity(n);

    for _ in 0..n {
        let device_id = format!("dev-{}", rng.random_range(1..=num_devices));
        let timestamp = rng.random_range(1625097600..1626097600);
        let status = if rng.random_bool(0.5) { "OK" } else { "ERROR" }.to_string();
        let signal_type = if rng.random_bool(0.5) {
            "voltage"
        } else {
            "temp"
        }
        .to_string();

        let value = match signal_type.as_str() {
            "temp" => rng.random_range(20.0..30.0),
            "voltage" => rng.random_range(3.0..4.0),
            _ => 0.0,
        };

        device_ids.push(device_id);
        timestamps.push(timestamp);
        statuses.push(status);
        signal_types.push(signal_type);
        values.push(value);
    }

    let df = df![
        "device_id" => device_ids,
        "timestamp" => timestamps,
        "status" => statuses,
        "signal_type" => signal_types,
        "value" => values,
    ]?;

    Ok(df)
}
