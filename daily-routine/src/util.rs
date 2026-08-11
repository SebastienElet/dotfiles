use std::error::Error;
use std::time::{SystemTime, UNIX_EPOCH};

#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub struct Timestamp {
    utc_seconds: i64,
    nanos: u32,
}

pub fn days_from_civil(year: i64, month: u32, day: u32) -> i64 {
    let adjusted_year = year - i64::from(month <= 2);
    let era = if adjusted_year >= 0 {
        adjusted_year
    } else {
        adjusted_year - 399
    } / 400;
    let year_of_era = adjusted_year - era * 400;
    let month = i64::from(month);
    let day_of_year = (153 * (month + if month > 2 { -3 } else { 9 }) + 2) / 5 + i64::from(day) - 1;
    let day_of_era = year_of_era * 365 + year_of_era / 4 - year_of_era / 100 + day_of_year;

    era * 146_097 + day_of_era - 719_468
}

pub fn parse_date_days(value: &str) -> Result<i64, String> {
    let bytes = value.as_bytes();
    if bytes.len() < 10 || bytes[4] != b'-' || bytes[7] != b'-' {
        return Err("date must start with YYYY-MM-DD".to_owned());
    }

    let year = parse_ascii_digits(&bytes[0..4])
        .ok_or_else(|| "date must start with YYYY-MM-DD".to_owned())?;
    let month = parse_ascii_digits(&bytes[5..7])
        .ok_or_else(|| "date must start with YYYY-MM-DD".to_owned())?;
    let day = parse_ascii_digits(&bytes[8..10])
        .ok_or_else(|| "date must start with YYYY-MM-DD".to_owned())?;

    if !(1..=12).contains(&month) {
        return Err(format!("invalid month: {month}"));
    }
    let year = i64::from(year);
    let maximum_day = days_in_month(year, month);
    if !(1..=maximum_day).contains(&day) {
        return Err(format!("invalid day: {day}"));
    }

    Ok(days_from_civil(year, month, day))
}

pub fn parse_timestamp(value: &str) -> Result<Timestamp, String> {
    let bytes = value.as_bytes();
    if bytes.len() < 20
        || bytes.get(10) != Some(&b'T')
        || bytes.get(13) != Some(&b':')
        || bytes.get(16) != Some(&b':')
    {
        return Err("timestamp must use RFC3339 date and time separators".to_owned());
    }

    let days = parse_date_days(value)?;
    let hour = parse_ascii_digits(&bytes[11..13])
        .ok_or_else(|| "timestamp hour must contain two digits".to_owned())?;
    let minute = parse_ascii_digits(&bytes[14..16])
        .ok_or_else(|| "timestamp minute must contain two digits".to_owned())?;
    let second = parse_ascii_digits(&bytes[17..19])
        .ok_or_else(|| "timestamp second must contain two digits".to_owned())?;
    if hour > 23 {
        return Err(format!("invalid hour: {hour}"));
    }
    if minute > 59 {
        return Err(format!("invalid minute: {minute}"));
    }
    if second > 59 {
        return Err(format!("invalid second: {second}"));
    }

    let mut cursor = 19;
    let nanos = if bytes.get(cursor) == Some(&b'.') {
        cursor += 1;
        let fraction_start = cursor;
        while bytes.get(cursor).is_some_and(u8::is_ascii_digit) {
            cursor += 1;
        }
        let fraction_length = cursor - fraction_start;
        if !(1..=9).contains(&fraction_length) {
            return Err("timestamp fraction must contain 1 to 9 digits".to_owned());
        }
        let mut nanos = parse_ascii_digits(&bytes[fraction_start..cursor])
            .ok_or_else(|| "timestamp fraction must contain only digits".to_owned())?;
        for _ in fraction_length..9 {
            nanos *= 10;
        }
        nanos
    } else {
        0
    };

    let offset_seconds = match bytes.get(cursor) {
        Some(b'Z') if cursor + 1 == bytes.len() => 0,
        Some(sign @ (b'+' | b'-')) if cursor + 6 == bytes.len() => {
            if bytes.get(cursor + 3) != Some(&b':') {
                return Err("timestamp offset must use HH:MM".to_owned());
            }
            let offset_hour = parse_ascii_digits(&bytes[cursor + 1..cursor + 3])
                .ok_or_else(|| "timestamp offset hour must contain two digits".to_owned())?;
            let offset_minute = parse_ascii_digits(&bytes[cursor + 4..cursor + 6])
                .ok_or_else(|| "timestamp offset minute must contain two digits".to_owned())?;
            if offset_hour > 23 {
                return Err(format!("invalid offset hour: {offset_hour}"));
            }
            if offset_minute > 59 {
                return Err(format!("invalid offset minute: {offset_minute}"));
            }
            let offset = i64::from(offset_hour * 3_600 + offset_minute * 60);
            if *sign == b'+' { offset } else { -offset }
        }
        _ => return Err("timestamp must end with Z or an offset in +/-HH:MM form".to_owned()),
    };

    let local_seconds =
        days * 86_400 + i64::from(hour) * 3_600 + i64::from(minute) * 60 + i64::from(second);
    Ok(Timestamp {
        utc_seconds: local_seconds - offset_seconds,
        nanos,
    })
}

pub fn today_days() -> Result<i64, Box<dyn Error>> {
    let elapsed_days = SystemTime::now().duration_since(UNIX_EPOCH)?.as_secs() / 86_400;

    Ok(i64::try_from(elapsed_days)?)
}

pub fn percent_encode(value: &str) -> String {
    const HEX: &[u8; 16] = b"0123456789ABCDEF";

    let mut encoded = String::with_capacity(value.len());
    for byte in value.bytes() {
        if byte.is_ascii_alphanumeric() || matches!(byte, b'-' | b'_' | b'.' | b'~') {
            encoded.push(char::from(byte));
        } else {
            encoded.push('%');
            encoded.push(char::from(HEX[usize::from(byte >> 4)]));
            encoded.push(char::from(HEX[usize::from(byte & 0x0f)]));
        }
    }

    encoded
}

pub fn truncate_chars(value: &str, max_chars: usize) -> String {
    value.chars().take(max_chars).collect()
}

pub fn find_linear_id(title: &str, branch: &str, body: &str, keys: &[String]) -> Option<String> {
    [title, branch, body]
        .into_iter()
        .find_map(|field| find_linear_id_in(field, keys))
}

fn parse_ascii_digits(bytes: &[u8]) -> Option<u32> {
    bytes.iter().try_fold(0_u32, |value, byte| {
        byte.is_ascii_digit()
            .then(|| value * 10 + u32::from(byte - b'0'))
    })
}

fn days_in_month(year: i64, month: u32) -> u32 {
    match month {
        2 if is_leap_year(year) => 29,
        2 => 28,
        4 | 6 | 9 | 11 => 30,
        _ => 31,
    }
}

fn is_leap_year(year: i64) -> bool {
    year % 4 == 0 && (year % 100 != 0 || year % 400 == 0)
}

fn find_linear_id_in(field: &str, keys: &[String]) -> Option<String> {
    let bytes = field.as_bytes();

    for start in 0..bytes.len() {
        if start > 0 && is_ascii_word(bytes[start - 1]) {
            continue;
        }

        for key in keys.iter().filter(|key| !key.is_empty()) {
            let key_bytes = key.as_bytes();
            let Some(key_end) = start.checked_add(key_bytes.len()) else {
                continue;
            };
            let Some(candidate) = bytes.get(start..key_end) else {
                continue;
            };
            if !candidate.eq_ignore_ascii_case(key_bytes) || bytes.get(key_end) != Some(&b'-') {
                continue;
            }

            let digit_start = key_end + 1;
            let mut digit_end = digit_start;
            while bytes.get(digit_end).is_some_and(u8::is_ascii_digit) {
                digit_end += 1;
            }
            if digit_end == digit_start
                || bytes
                    .get(digit_end)
                    .is_some_and(|byte| is_ascii_word(*byte))
            {
                continue;
            }

            let mut id = key.to_ascii_uppercase();
            id.push('-');
            id.extend(
                bytes[digit_start..digit_end]
                    .iter()
                    .map(|byte| char::from(*byte)),
            );
            return Some(id);
        }
    }

    None
}

fn is_ascii_word(byte: u8) -> bool {
    byte.is_ascii_alphanumeric() || byte == b'_'
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn civil_days_handle_leap_years() {
        assert_eq!(
            days_from_civil(2024, 3, 1) - days_from_civil(2024, 2, 28),
            2
        );
    }

    #[test]
    fn date_parser_uses_the_iso_date_prefix() {
        assert_eq!(parse_date_days("1970-01-01T00:00:00Z"), Ok(0));
    }

    #[test]
    fn date_parser_rejects_invalid_dates_and_short_timestamps() {
        for value in [
            "1970-00-01",
            "1970-13-01",
            "1970-01-00",
            "2023-02-29",
            "1970-01-0",
        ] {
            assert!(parse_date_days(value).is_err(), "accepted {value}");
        }
    }

    #[test]
    fn date_parser_requires_iso_separators() {
        assert!(parse_date_days("1970/01/01").is_err());
    }

    #[test]
    fn timestamp_parser_rejects_missing_or_invalid_times() {
        assert!(parse_timestamp("2026-08-11").is_err());
        assert!(parse_timestamp("2026-08-11T99:00:00Z").is_err());
    }

    #[test]
    fn timestamp_order_applies_utc_offsets() {
        let earlier = parse_timestamp("2026-08-11T09:30:00+02:00").unwrap();
        let later = parse_timestamp("2026-08-11T08:00:00Z").unwrap();

        assert!(earlier < later);
    }

    #[test]
    fn timestamp_order_preserves_fractional_seconds() {
        let whole_second = parse_timestamp("2026-08-11T08:00:00Z").unwrap();
        let fractional = parse_timestamp("2026-08-11T08:00:00.355Z").unwrap();

        assert!(fractional > whole_second);
    }

    #[test]
    fn equivalent_utc_timestamp_forms_compare_equal() {
        assert_eq!(
            parse_timestamp("2026-08-11T08:00:00Z"),
            parse_timestamp("2026-08-11T08:00:00+00:00")
        );
    }

    #[test]
    fn today_uses_complete_days_since_the_unix_epoch() {
        let unix_days = || {
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .expect("system clock is before the Unix epoch")
                .as_secs()
                / 86_400
        };
        let before = unix_days();

        let actual = today_days().expect("current date should be available");

        let after = unix_days();
        assert!(
            i64::try_from(before).expect("day count should fit in i64") <= actual
                && actual <= i64::try_from(after).expect("day count should fit in i64")
        );
    }

    #[test]
    fn percent_encoding_uses_uppercase_utf8_octets() {
        assert_eq!(
            percent_encode("Corriger l’été"),
            "Corriger%20l%E2%80%99%C3%A9t%C3%A9"
        );
    }

    #[test]
    fn percent_encoding_preserves_only_safe_ascii_characters() {
        assert_eq!(
            percent_encode("ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789-_.~"),
            "ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789-_.~"
        );
    }

    #[test]
    fn truncation_counts_characters_without_splitting_utf8() {
        assert_eq!(truncate_chars("éclair", 2), "éc");
        assert_eq!(truncate_chars("été", 3), "été");
        assert_eq!(truncate_chars("été", 8), "été");
        assert_eq!(truncate_chars("été", 0), "");
    }

    #[test]
    fn linear_id_prefers_title_then_branch_then_body() {
        let keys = linear_keys();

        assert_eq!(
            find_linear_id("Ship APP-12", "fix/OPS-34", "Related to APP-56", &keys),
            Some("APP-12".to_owned())
        );
        assert_eq!(
            find_linear_id("Ship change", "fix/OPS-34", "Related to APP-56", &keys),
            Some("OPS-34".to_owned())
        );
        assert_eq!(
            find_linear_id("Ship change", "fix/change", "Related to APP-56", &keys),
            Some("APP-56".to_owned())
        );
        assert_eq!(
            find_linear_id("Ship change", "fix/change", "No issue", &keys),
            None
        );
    }

    #[test]
    fn linear_id_matching_is_case_insensitive_and_normalized() {
        assert_eq!(
            find_linear_id("Track ops-42", "", "", &linear_keys()),
            Some("OPS-42".to_owned())
        );
    }

    #[test]
    fn linear_id_requires_ascii_word_boundaries() {
        let keys = linear_keys();

        for title in ["xAPP-123", "1APP-123", "_APP-123", "APP-123x", "APP-123_"] {
            assert_eq!(
                find_linear_id(title, "", "", &keys),
                None,
                "matched {title}"
            );
        }
        assert_eq!(
            find_linear_id("(APP-123)", "", "", &keys),
            Some("APP-123".to_owned())
        );
        assert_eq!(
            find_linear_id("APP-1231", "", "", &keys),
            Some("APP-1231".to_owned())
        );
    }

    #[test]
    fn linear_id_requires_at_least_one_digit() {
        assert_eq!(find_linear_id("APP-", "", "", &linear_keys()), None);
    }

    fn linear_keys() -> Vec<String> {
        vec!["APP".to_owned(), "OPS".to_owned()]
    }
}
