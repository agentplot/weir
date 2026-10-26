//! A schedule, read as five-field cron and nothing more (C.5).
//!
//! The search for the times a schedule names is croner's. croner reads more
//! than five-field cron, with no switch to refuse the rest, so the text is
//! read here first: what five-field cron does not have is refused, and what
//! it has is handed to croner as numbers alone.

use std::fmt;

use croner::parser::{CronParser, Seconds};
use croner::Cron;
use jiff::tz::TimeZone;
use jiff::{Timestamp, Zoned};

/// A schedule (59), five-field cron read in the time zone of the now it is
/// asked against (C.5).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Schedule {
    cron: Cron,
}

impl Schedule {
    /// Reads `text` as five-field cron (59; C.5), or refuses it naming the
    /// field and the token that is not.
    pub fn parse(text: &str) -> Result<Schedule, ScheduleError> {
        let text = text.trim();
        if text.starts_with('@') {
            return Err(ScheduleError::Nickname(text.to_string()));
        }
        let fields: Vec<&str> = text.split_whitespace().collect();
        if fields.len() != Field::ALL.len() {
            return Err(ScheduleError::FieldCount(fields.len()));
        }
        let pattern = Field::ALL
            .iter()
            .zip(&fields)
            .map(|(field, text)| field.read(text))
            .collect::<Result<Vec<_>, _>>()?
            .join(" ");
        // Five fields read with the seconds at 0; the day fields combine as
        // cron's do, either one when both are restricted.
        let cron = CronParser::builder()
            .seconds(Seconds::Optional)
            .build()
            .parse(&pattern)
            .map_err(|error| ScheduleError::Unsearchable(error.to_string()))?;
        Ok(Schedule { cron })
    }

    /// The first time the schedule names strictly after `instant`, read in
    /// `zone`, or none when it names no later time.
    pub(crate) fn first_after(&self, instant: Timestamp, zone: &TimeZone) -> Option<Zoned> {
        // Searched from the instant's whole second, that second included, so
        // a fraction past a named time never decides whether it was passed.
        let start = Timestamp::from_second(instant.as_second())
            .ok()?
            .to_zoned(zone.clone());
        let found = self.cron.find_next_occurrence(&start, true).ok()?;
        if found.timestamp() > instant {
            return Some(found);
        }
        self.cron.find_next_occurrence(&found, false).ok()
    }
}

/// Why a schedule was refused (59): the part of it that is not five-field
/// cron (C.5).
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ScheduleError {
    /// Other than five fields: the count found.
    FieldCount(usize),
    /// A nickname, such as `@hourly`, in place of the five fields.
    Nickname(String),
    /// A token of one field that five-field cron does not read.
    Field {
        /// The field the token is in.
        field: Field,
        /// The token refused: one element of the field's list.
        token: String,
        /// Why it is refused.
        reason: Reason,
    },
    /// Five-field cron that croner still refused, with its reason.
    Unsearchable(String),
}

impl fmt::Display for ScheduleError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ScheduleError::FieldCount(count) => write!(
                f,
                "a schedule is five fields (minute, hour, day of month, month, day of week), \
                 and this one has {count}"
            ),
            ScheduleError::Nickname(token) => {
                write!(f, "`{token}` is a nickname; a schedule is five fields")
            }
            ScheduleError::Field {
                field,
                token,
                reason,
            } => {
                let (min, max) = field.bounds();
                write!(f, "{field} `{token}`: ")?;
                match reason {
                    Reason::Extension(c) => write!(f, "`{c}` is not five-field cron"),
                    Reason::UnknownName => write!(f, "not a month's or a weekday's name"),
                    Reason::NameOutsideField => write!(f, "a name the {field} field does not take"),
                    Reason::OutOfRange => write!(f, "a {field} is {min} to {max}"),
                    Reason::Step => write!(f, "a step is 1 to {max}"),
                    Reason::Backward => write!(f, "a range runs from its start up to its end"),
                    Reason::Malformed => {
                        write!(
                            f,
                            "not `*`, a number, a range or a step over `*` or a range"
                        )
                    }
                }
            }
            ScheduleError::Unsearchable(reason) => {
                write!(f, "the schedule's times cannot be searched: {reason}")
            }
        }
    }
}

impl std::error::Error for ScheduleError {}

/// One of the five fields of a schedule (59; C.5).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Field {
    /// The minute, 0 to 59.
    Minute,
    /// The hour, 0 to 23.
    Hour,
    /// The day of the month, 1 to 31.
    DayOfMonth,
    /// The month, 1 to 12 or its name.
    Month,
    /// The day of the week, 0 to 7, Sunday as both 0 and 7, or its name.
    DayOfWeek,
}

impl fmt::Display for Field {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(match self {
            Field::Minute => "minute",
            Field::Hour => "hour",
            Field::DayOfMonth => "day of month",
            Field::Month => "month",
            Field::DayOfWeek => "day of week",
        })
    }
}

/// Why a token of a field of a schedule is refused (59; C.5).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Reason {
    /// A specifier five-field cron does not have: `L`, `W`, `#`, `?` or `+`.
    Extension(char),
    /// Letters that are no month's or weekday's three-letter name.
    UnknownName,
    /// A month's name outside the month field, or a weekday's outside the
    /// day-of-week field.
    NameOutsideField,
    /// A value outside the field's range.
    OutOfRange,
    /// A step that is not a number from 1 to the field's highest value.
    Step,
    /// A range whose start is after its end.
    Backward,
    /// Not `*`, a number, a range, or a step over `*` or a range.
    Malformed,
}

const MONTHS: [&str; 12] = [
    "JAN", "FEB", "MAR", "APR", "MAY", "JUN", "JUL", "AUG", "SEP", "OCT", "NOV", "DEC",
];
const WEEKDAYS: [&str; 7] = ["SUN", "MON", "TUE", "WED", "THU", "FRI", "SAT"];

impl Field {
    const ALL: [Field; 5] = [
        Field::Minute,
        Field::Hour,
        Field::DayOfMonth,
        Field::Month,
        Field::DayOfWeek,
    ];

    fn bounds(self) -> (u32, u32) {
        match self {
            Field::Minute => (0, 59),
            Field::Hour => (0, 23),
            Field::DayOfMonth => (1, 31),
            Field::Month => (1, 12),
            Field::DayOfWeek => (0, 7),
        }
    }

    /// Reads one field into the same field in numbers alone.
    fn read(self, text: &str) -> Result<String, ScheduleError> {
        if text == "*" {
            return Ok(text.to_string());
        }
        text.split(',')
            .map(|element| {
                self.element(element)
                    .map_err(|reason| ScheduleError::Field {
                        field: self,
                        token: element.to_string(),
                        reason,
                    })
            })
            .collect::<Result<Vec<_>, _>>()
            .map(|elements| elements.join(","))
    }

    /// One element of a field's list: `*/n`, a value, a range, or a range
    /// with a step.
    fn element(self, element: &str) -> Result<String, Reason> {
        self.extensions(element)?;
        let (base, step) = match element.split_once('/') {
            Some((base, step)) => (base, Some(self.step(step)?)),
            None => (element, None),
        };
        let base = match base.split_once('-') {
            _ if base == "*" && step.is_some() => base.to_string(),
            Some((start, end)) => {
                let start = self.value(start, false)?;
                let end = self.value(end, start > 0)?;
                if start > end {
                    return Err(Reason::Backward);
                }
                format!("{start}-{end}")
            }
            // A single value with a step is not five-field cron.
            None if step.is_none() => self.value(base, false)?.to_string(),
            None => return Err(Reason::Malformed),
        };
        Ok(match step {
            Some(step) => format!("{base}/{step}"),
            None => base,
        })
    }

    /// Refuses a specifier of the cron dialects five-field cron lacks.
    fn extensions(self, element: &str) -> Result<(), Reason> {
        if let Some(c) = element.chars().find(|c| matches!(c, '#' | '?' | '+')) {
            return Err(Reason::Extension(c));
        }
        for word in element
            .split(|c: char| !c.is_ascii_alphabetic())
            .filter(|word| !word.is_empty())
        {
            let upper = word.to_ascii_uppercase();
            if matches!(upper.as_str(), "L" | "W" | "LW") {
                return Err(Reason::Extension(char::from(upper.as_bytes()[0])));
            }
        }
        Ok(())
    }

    fn step(self, text: &str) -> Result<u32, Reason> {
        let (_, max) = self.bounds();
        match number(text) {
            Some(Some(step)) if (1..=max).contains(&step) => Ok(step),
            Some(_) => Err(Reason::Step),
            None => Err(Reason::Malformed),
        }
    }

    /// A number in the field's range, or a name the field takes. Sunday's
    /// name reads 7 when `sunday_last`, at the end of a range that starts
    /// after Sunday, so `FRI-SUN` runs to Sunday.
    fn value(self, text: &str, sunday_last: bool) -> Result<u32, Reason> {
        let (min, max) = self.bounds();
        if text.chars().all(|c| c.is_ascii_alphabetic()) && !text.is_empty() {
            let upper = text.to_ascii_uppercase();
            let month = MONTHS.iter().position(|name| *name == upper);
            let weekday = WEEKDAYS.iter().position(|name| *name == upper);
            return match (self, month, weekday) {
                (Field::Month, Some(index), _) => Ok(index as u32 + 1),
                (Field::DayOfWeek, _, Some(0)) if sunday_last => Ok(7),
                (Field::DayOfWeek, _, Some(index)) => Ok(index as u32),
                (_, None, None) => Err(Reason::UnknownName),
                _ => Err(Reason::NameOutsideField),
            };
        }
        match number(text) {
            Some(Some(value)) if (min..=max).contains(&value) => Ok(value),
            Some(_) => Err(Reason::OutOfRange),
            None => Err(Reason::Malformed),
        }
    }
}

/// Digits read as a number: none when `text` is not digits alone, and
/// `Some(None)` when they are too many to hold.
fn number(text: &str) -> Option<Option<u32>> {
    if text.is_empty() || !text.chars().all(|c| c.is_ascii_digit()) {
        return None;
    }
    Some(text.parse().ok())
}
