use std::collections::BTreeMap;

pub(super) struct Process {
    pub pid: u32,
    pub parent: u32,
    pub rss: u64,
    pub age: u64,
    pub command: String,
}

pub(super) fn parse(text: &str) -> Result<BTreeMap<u32, Process>, &'static str> {
    let mut processes = BTreeMap::new();
    for line in text.lines().filter(|line| !line.trim().is_empty()) {
        let mut fields = line.split_whitespace();
        let pid = fields
            .next()
            .and_then(|value| value.parse().ok())
            .ok_or("invalid ps PID")?;
        let parent = fields
            .next()
            .and_then(|value| value.parse().ok())
            .ok_or("invalid ps parent PID")?;
        let rss = fields
            .next()
            .and_then(|value| value.parse().ok())
            .ok_or("invalid ps RSS")?;
        let age = elapsed(fields.next().ok_or("missing ps age")?).ok_or("invalid ps age")?;
        let command = fields.collect::<Vec<_>>().join(" ");
        if command.is_empty() {
            return Err("missing ps command");
        }
        if processes
            .insert(
                pid,
                Process {
                    pid,
                    parent,
                    rss,
                    age,
                    command,
                },
            )
            .is_some()
        {
            return Err("duplicate ps PID");
        }
    }
    if processes.is_empty() {
        return Err("ps returned no processes");
    }
    Ok(processes)
}

fn elapsed(value: &str) -> Option<u64> {
    let (days, time) = value
        .split_once('-')
        .map_or(Some((0, value)), |(days, time)| {
            days.parse::<u64>().ok().map(|days| (days, time))
        })?;
    let parts = time
        .split(':')
        .map(str::parse::<u64>)
        .collect::<Result<Vec<_>, _>>()
        .ok()?;
    let seconds = match parts.as_slice() {
        [minutes, seconds] if *seconds < 60 => minutes.checked_mul(60)?.checked_add(*seconds)?,
        [hours, minutes, seconds] if *minutes < 60 && *seconds < 60 => hours
            .checked_mul(3600)?
            .checked_add(minutes.checked_mul(60)?)?
            .checked_add(*seconds)?,
        _ => return None,
    };
    days.checked_mul(86400)?.checked_add(seconds)
}
