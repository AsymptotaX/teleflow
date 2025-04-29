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

    let device_ids: Vec<String> = (0..n)
        .map(|_| format!("dev-{}", rng.random_range(1..=num_devices)))
        .collect();

    let timestamps: Vec<i64> = (0..n)
        .map(|_| rng.random_range(1625097600..1626097600))
        .collect();

    let temps: Vec<f64> = (0..n)
        .map(|_| rng.random_range(20.0..30.0))
        .collect();

    let voltages: Vec<f64> = (0..n)
        .map(|_| rng.random_range(3.0..4.0))
        .collect();

    let statuses: Vec<String> = (0..n)
        .map(|_| {
            if rng.random_bool(0.5) {
                "OK".to_string()
            } else {
                "ERROR".to_string()
            }
        })
        .collect();

    let signal_types: Vec<String> = (0..n)
        .map(|_| {
            if rng.random_bool(0.5) {
                "voltage".to_string()
            } else {
                "temp".to_string()
            }
        })
        .collect();

    DataFrame::new(vec![
        Column::from(Series::new(PlSmallStr::from("device_id"), device_ids)),
        Column::from(Series::new(PlSmallStr::from("timestamp"), timestamps)),
        Column::from(Series::new(PlSmallStr::from("temp"), temps)),
        Column::from(Series::new(PlSmallStr::from("voltage"), voltages)),
        Column::from(Series::new(PlSmallStr::from("status"), statuses)),
        Column::from(Series::new(PlSmallStr::from("signal_type"), signal_types)),
    ])
}
