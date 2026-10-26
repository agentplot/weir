//! A stage's quiet time (53, C.4): one to three parts, hours, minutes and
//! seconds in that order, each a whole number followed by `h`, `m` or `s`,
//! as `45m`, `1h30m` or `90s`. The format reads it into a [`Duration`] so
//! that no two readers read one declaration's quiet time two ways (58).

use std::time::Duration;

const UNITS: [(u8, u64); 3] = [(b'h', 3600), (b'm', 60), (b's', 1)];

/// The duration a quiet time names, or `None` when the text is not one.
pub(crate) fn parse(text: &str) -> Option<Duration> {
    let mut rest = text.as_bytes();
    if rest.is_empty() {
        return None;
    }
    let mut next = 0;
    let mut seconds: u64 = 0;
    while !rest.is_empty() {
        let digits = rest.iter().take_while(|b| b.is_ascii_digit()).count();
        let (&unit, after) = rest.get(digits..)?.split_first()?;
        if digits == 0 {
            return None;
        }
        let number: u64 = std::str::from_utf8(&rest[..digits]).ok()?.parse().ok()?;
        let place = next + UNITS[next..].iter().position(|(u, _)| *u == unit)?;
        seconds = seconds.checked_add(number.checked_mul(UNITS[place].1)?)?;
        next = place + 1;
        rest = after;
    }
    Some(Duration::from_secs(seconds))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn minutes(n: u64) -> Option<Duration> {
        Some(Duration::from_secs(n * 60))
    }

    #[test]
    fn each_part_alone_and_together() {
        assert_eq!(parse("45m"), minutes(45));
        assert_eq!(parse("1h30m"), minutes(90));
        assert_eq!(parse("90s"), Some(Duration::from_secs(90)));
        assert_eq!(parse("2h"), minutes(120));
        assert_eq!(parse("1h2m3s"), Some(Duration::from_secs(3723)));
        assert_eq!(parse("0m"), minutes(0));
    }

    #[test]
    fn anything_else_is_no_duration() {
        for text in [
            "", "45", "m", "h30m", "30m1h", "1h1h", "1.5h", "45 m", " 45m", "45m ", "1d", "-5m",
            "45M", "１m",
        ] {
            assert_eq!(parse(text), None, "{text:?}");
        }
    }

    #[test]
    fn a_time_too_long_to_hold_is_no_duration() {
        assert_eq!(parse("99999999999999999999s"), None);
        assert_eq!(parse(&format!("{}h", u64::MAX)), None);
    }
}
