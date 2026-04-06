use crate::type_system::Value;
use crate::{CorvoError, CorvoResult};
use chrono::{Local, TimeZone};
use std::collections::HashMap;
use std::time::{SystemTime, UNIX_EPOCH};

/// Format a Unix timestamp in the **local** timezone (honours `TZ`) using a `strftime` pattern.
/// Args: `seconds: number`, `[nanoseconds: number]`, `format: string`
pub fn format_local(args: &[Value], _named_args: &HashMap<String, Value>) -> CorvoResult<Value> {
    let secs = args.first().and_then(|v| v.as_number()).ok_or_else(|| {
        CorvoError::invalid_argument("time.format_local requires seconds (number)")
    })? as i64;

    let nsec = args
        .get(1)
        .and_then(|v| v.as_number())
        .map(|n| n.clamp(0.0, 1e9 - 1.0) as u32)
        .unwrap_or(0);

    let fmt = args
        .get(2)
        .and_then(|v| v.as_string())
        .ok_or_else(|| {
            CorvoError::invalid_argument(
                "time.format_local requires a format string as third argument (chrono strftime)",
            )
        })?
        .as_str();

    let dt = Local
        .timestamp_opt(secs, nsec)
        .single()
        .ok_or_else(|| CorvoError::invalid_argument("time.format_local: invalid timestamp"))?;

    Ok(Value::String(dt.format(fmt).to_string()))
}

/// Seconds since Unix epoch in local time interpretation for `format_local`.
pub fn unix_now(args: &[Value], _named_args: &HashMap<String, Value>) -> CorvoResult<Value> {
    if !args.is_empty() {
        return Err(CorvoError::invalid_argument(
            "time.unix_now takes no arguments",
        ));
    }
    let secs = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_err(|e| CorvoError::runtime(e.to_string()))?
        .as_secs_f64();
    Ok(Value::Number(secs))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn empty() -> HashMap<String, Value> {
        HashMap::new()
    }

    #[test]
    fn format_epoch_seconds() {
        let args = vec![
            Value::Number(0.0),
            Value::Number(0.0),
            Value::String("%s".to_string()),
        ];
        let s = format_local(&args, &empty()).unwrap();
        assert_eq!(s, Value::String("0".to_string()));
    }

    #[test]
    fn unix_now_positive() {
        let v = unix_now(&[], &empty()).unwrap();
        assert!(v.as_number().unwrap() > 1_600_000_000.0);
    }
}
