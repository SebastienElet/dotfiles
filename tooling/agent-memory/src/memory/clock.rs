use super::UtcTimestamp;
use jiff::Timestamp;

pub trait Clock: Send + Sync {
    fn now(&self) -> UtcTimestamp;
}

#[derive(Clone, Copy, Debug, Default)]
pub struct SystemClock;

impl Clock for SystemClock {
    fn now(&self) -> UtcTimestamp {
        UtcTimestamp::from_validated(Timestamp::now().to_string())
    }
}

pub(crate) fn timestamp(value: &UtcTimestamp) -> Option<Timestamp> {
    value.as_str().parse().ok()
}
