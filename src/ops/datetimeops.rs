use chrono::{DateTime, TimeZone, Utc};

// Helper: convert DateTime<Utc> → nanoseconds since epoch
pub fn datetime_to_nanos(dt: DateTime<Utc>) -> u64 {
    let secs = dt.timestamp() as u64;
    let nanos = dt.timestamp_subsec_nanos() as u64;
    secs * 1_000_000_000 + nanos
}
/// Convert nanoseconds since epoch to `DateTime<Utc>`
pub fn convert_nano_to_datetime(ts_nanos: u64) -> Option<DateTime<Utc>> {
    let seconds = (ts_nanos / 1_000_000_000) as i64;
    let nanos = (ts_nanos % 1_000_000_000) as u32;
    Utc.timestamp_opt(seconds, nanos).single()
}

/// Find min and max datetime from a vector of nanoseconds
pub fn min_max_datetimes(timestamps: Vec<u64>) -> Option<(DateTime<Utc>, DateTime<Utc>)> {
    if timestamps.is_empty() {
        return None;
    }

    let mut min_ts = u64::MAX;
    let mut max_ts = u64::MIN;

    for &ts in &timestamps {
        if ts < min_ts {
            min_ts = ts;
        }
        if ts > max_ts {
            max_ts = ts;
        }
    }

    let min_dt = convert_nano_to_datetime(min_ts)?;
    let max_dt = convert_nano_to_datetime(max_ts)?;

    Some((min_dt, max_dt))
}
