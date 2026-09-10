use super::{Clock, SystemClock, timestamp, utc_timestamp};
use crate::parse_utc_timestamp;

#[test]
fn system_clock_produces_a_valid_utc_timestamp() -> Result<(), Box<dyn std::error::Error>> {
    let now = SystemClock.now();
    assert_eq!(parse_utc_timestamp(now.as_str())?, now);
    assert!(timestamp(&now).is_some());
    Ok(())
}

#[test]
fn timestamp_conversion_preserves_supported_calendar_and_precision_boundaries()
-> Result<(), Box<dyn std::error::Error>> {
    for value in [
        "0001-01-01T00:00:00Z",
        "1969-12-31T23:59:59.999999999Z",
        "2000-02-29T12:34:56.000000001Z",
        "9999-12-31T23:59:59.999999999Z",
    ] {
        let parsed =
            timestamp(&parse_utc_timestamp(value)?).ok_or("timestamp conversion failed")?;
        assert_eq!(utc_timestamp(parsed).as_str(), value);
    }
    assert!(timestamp(&parse_utc_timestamp("2026-08-28T00:00:00.1234567891Z")?).is_none());
    Ok(())
}
