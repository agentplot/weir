//! A stage's quiet time (57, C.4): one whole number followed by one unit,
//! `s`, `m` or `h`, as `45m`, `90s` or `2h`. Two parts, such as `1h30m`,
//! are no quiet time. The format reads it into a [`Duration`] so that no
//! two readers read one declaration's quiet time two ways (58).

use std::time::Duration;

const UNITS: [(char, u64); 3] = [('s', 1), ('m', 60), ('h', 3600)];

/// The duration a quiet time names, or `None` when the text is not one.
pub(crate) fn parse(text: &str) -> Option<Duration> {
    let (number, seconds) = UNITS
        .iter()
        .find_map(|&(unit, seconds)| Some((text.strip_suffix(unit)?, seconds)))?;
    if number.is_empty() || !number.bytes().all(|b| b.is_ascii_digit()) {
        return None;
    }
    let number: u64 = number.parse().ok()?;
    Some(Duration::from_secs(number.checked_mul(seconds)?))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn minutes(n: u64) -> Option<Duration> {
        Some(Duration::from_secs(n * 60))
    }

    #[test]
    fn one_number_and_one_unit() {
        assert_eq!(parse("45m"), minutes(45));
        assert_eq!(parse("90s"), Some(Duration::from_secs(90)));
        assert_eq!(parse("2h"), minutes(120));
        assert_eq!(parse("0m"), minutes(0));
        assert_eq!(parse("007s"), Some(Duration::from_secs(7)));
    }

    #[test]
    fn two_parts_are_no_duration() {
        for text in ["1h30m", "1h2m3s", "30m1h", "1h1h", "90s0m"] {
            assert_eq!(parse(text), None, "{text:?}");
        }
    }

    #[test]
    fn anything_else_is_no_duration() {
        for text in [
            "", "45", "m", "h30m", "1.5h", "45 m", " 45m", "45m ", "1d", "-5m", "+5m", "45M",
            "１m", "5é",
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
