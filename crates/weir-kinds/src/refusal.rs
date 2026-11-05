use std::fmt;

/// One field a declaration fails on, by its path from the declaration's
/// root, with its reason (57, 81).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Problem {
    /// The field's path, stages indexed from 0:
    /// `kinds.bump.stages[1].prompt`. For text that is not TOML, the line
    /// and column it fails at: `line 3, column 7`. For a package, a file
    /// of it is named by its path within the package, `prompts/code.md`,
    /// and a folder that could not be read by its path as given (92).
    pub field: String,
    /// Why the field fails.
    pub reason: String,
}

impl fmt::Display for Problem {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}: {}", self.field, self.reason)
    }
}

/// A declaration refused whole: every field that fails it, in the order
/// the reader walks the declaration, and no kind (57, 81).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Refusal {
    /// Every problem found; never empty.
    pub problems: Vec<Problem>,
}

impl fmt::Display for Refusal {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        for (n, problem) in self.problems.iter().enumerate() {
            if n > 0 {
                writeln!(f)?;
            }
            write!(f, "{problem}")?;
        }
        Ok(())
    }
}

impl std::error::Error for Refusal {}

/// The problems a read gathers as it walks, so that one read names every
/// field that fails (57).
#[derive(Debug, Default)]
pub(crate) struct Problems(Vec<Problem>);

impl Problems {
    pub(crate) fn refuse(&mut self, field: &str, reason: impl Into<String>) {
        self.0.push(Problem {
            field: field.to_owned(),
            reason: reason.into(),
        });
    }

    /// Every problem of a refusal made by another read, as this read's
    /// own, a package's `kind.toml` refused by `read-kinds` among them.
    pub(crate) fn absorb(&mut self, refusal: Refusal) {
        self.0.extend(refusal.problems);
    }

    /// How many problems the read has gathered so far.
    pub(crate) fn len(&self) -> usize {
        self.0.len()
    }

    /// The value read when no field failed, and otherwise the refusal and
    /// nothing of the value.
    pub(crate) fn finish<T>(self, value: T) -> Result<T, Refusal> {
        if self.0.is_empty() {
            Ok(value)
        } else {
            Err(Refusal { problems: self.0 })
        }
    }
}

/// The refusal of text that does not parse as TOML: one problem, placed by
/// the line and column the parser's span starts at.
pub(crate) fn not_toml(text: &str, error: &toml::de::Error) -> Refusal {
    let field = match error.span().and_then(|span| text.get(..span.start)) {
        Some(before) => {
            let line = before.matches('\n').count() + 1;
            let column = before.rsplit('\n').next().map_or(0, |l| l.chars().count()) + 1;
            format!("line {line}, column {column}")
        }
        None => "the text".to_owned(),
    };
    let reason = error
        .message()
        .lines()
        .map(str::trim)
        .filter(|l| !l.is_empty())
        .collect::<Vec<_>>()
        .join("; ");
    Refusal {
        problems: vec![Problem { field, reason }],
    }
}
