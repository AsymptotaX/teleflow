use crate::config::types::FilterRule;
use crate::error::TelemetryError;
use polars::prelude::*;

/// Applies a series of filtering rules to a `LazyFrame`.
///
/// # Arguments
///
/// * `lf` - The `LazyFrame` to which the filters will be applied.
/// * `filters` - A slice of `FilterRule` objects defining the filtering criteria.
///
/// # Returns
///
/// A `Result` containing the filtered `LazyFrame` on success, or a `TelemetryError` on failure.
///
/// # Errors
///
/// Returns a `TelemetryError` if an unsupported operator or value type is encountered.
///
/// # Filtering Rules
///
/// Each `FilterRule` specifies:
/// - `column`: The name of the column to filter.
/// - `operator`: The comparison operator (e.g., "eq", "neq", "gt", "lt", "ge", "le").
/// - `value`: The value to compare against, which can be a string or a number.
pub fn apply_filters(lf: LazyFrame, filters: &[FilterRule]) -> Result<LazyFrame, TelemetryError> {
    let mut filtered_lf = lf;

    for rule in filters {
        let column = col(&rule.column);
        let value = &rule.value;

        let expr = match value {
            serde_json::Value::String(s) => match rule.operator.as_str() {
                "eq" => column.eq(lit(s.clone())),
                "neq" => column.neq(lit(s.clone())),
                _ => {
                    return Err(TelemetryError::InvalidInput(format!(
                        "Unsupported operator: {}",
                        rule.operator
                    )));
                }
            },
            serde_json::Value::Number(num) => {
                if let Some(n) = num.as_f64() {
                    match rule.operator.as_str() {
                        "eq" => column.eq(lit(n)),
                        "neq" => column.neq(lit(n)),
                        "gt" => column.gt(lit(n)),
                        "lt" => column.lt(lit(n)),
                        "ge" => column.gt_eq(lit(n)),
                        "le" => column.lt_eq(lit(n)),
                        _ => {
                            return Err(TelemetryError::InvalidInput(format!(
                                "Unsupported operator: {}",
                                rule.operator
                            )));
                        }
                    }
                } else {
                    return Err(TelemetryError::InvalidInput("Invalid number".into()));
                }
            }
            _ => {
                return Err(TelemetryError::InvalidInput(
                    "Unsupported value type".into(),
                ));
            }
        };

        filtered_lf = filtered_lf.filter(expr);
    }

    Ok(filtered_lf)
}
