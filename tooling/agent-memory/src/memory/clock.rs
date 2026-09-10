use super::UtcTimestamp;
use time::{OffsetDateTime, format_description::well_known::Rfc3339};

#[cfg(test)]
mod tests;

pub trait Clock: Send + Sync {
    fn now(&self) -> UtcTimestamp;
}

#[derive(Clone, Copy, Debug, Default)]
pub struct SystemClock;

impl Clock for SystemClock {
    fn now(&self) -> UtcTimestamp {
        utc_timestamp(OffsetDateTime::now_utc())
    }
}

fn utc_timestamp(now: OffsetDateTime) -> UtcTimestamp {
    let fraction = format!("{:09}", now.nanosecond());
    let fraction = fraction.trim_end_matches('0');
    let separator = if fraction.is_empty() { "" } else { "." };
    UtcTimestamp::from_validated(format!(
        "{}T{:02}:{:02}:{:02}{separator}{fraction}Z",
        now.date(),
        now.hour(),
        now.minute(),
        now.second(),
    ))
}

pub fn timestamp(value: &UtcTimestamp) -> Option<OffsetDateTime> {
    if value
        .as_str()
        .split_once('.')
        .is_some_and(|(_, fraction)| fraction.len() > 10)
    {
        return None;
    }
    OffsetDateTime::parse(value.as_str(), &Rfc3339).ok()
}
