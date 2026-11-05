//! A lock of kinds: every identity it records with the content hash of
//! its package, read by identity (91, 93, C.4; core.30, core.45).

use std::collections::BTreeMap;

use toml::{Table, Value};

use crate::fields::{field, unnamed};
use crate::hash::ContentHash;
use crate::identity::Identity;
use crate::refusal::{not_toml, Problems, Refusal};

/// The fields of a lock's entry the format names; every other key is
/// returned unread.
const ENTRY_FIELDS: [&str; 2] = ["kind", "hash"];

/// A lock read: every identity it records, each with its content hash,
/// and every key beside them the format does not name (91, C.4). It holds
/// an identity at most once.
#[derive(Debug, Clone, Default, PartialEq)]
pub struct Lock {
    /// Every identity its `kinds` tables record, in identity order, each
    /// with its hash; none when there is no `kinds` table (91, 93).
    pub entries: BTreeMap<Identity, LockEntry>,
    /// Every top-level key but `kinds`, unread and as written (C.4).
    pub sections: Table,
}

/// What a lock records of one identity (91, C.4).
#[derive(Debug, Clone, PartialEq)]
pub struct LockEntry {
    /// The content hash of the identity's package.
    pub hash: ContentHash,
    /// Every key of its `kinds` table but `kind` and `hash`, unread and as
    /// written (C.4).
    pub sections: Table,
}

/// `read-lock` (93): every identity a lock's text records with its hash,
/// or the lock refused whole, naming every field that fails it (91).
///
/// A lock is refused where its text is not TOML, `kinds` is not an array
/// of tables, an entry has no `kind` or no `hash`, a `kind` is no whole
/// identity, a `hash` is no content hash, or an identity is recorded
/// again, which is named at its second entry with its first (91, C.4;
/// core.45). An identity of any source is read, [`REPO`](crate::REPO) and
/// [`LAUNCH`](crate::LAUNCH) among them: whose kinds a lock may record is
/// its reader's to say. The same text always gives the same result, and
/// reading changes nothing (93).
pub fn read_lock(text: &str) -> Result<Lock, Refusal> {
    let root: Table = text.parse().map_err(|error| not_toml(text, &error))?;
    let mut problems = Problems::default();
    let mut entries = BTreeMap::new();
    let tables = match root.get("kinds") {
        None => Some(Vec::new()),
        Some(Value::Array(items)) => items.iter().map(Value::as_table).collect(),
        Some(_) => None,
    };
    match tables {
        Some(tables) => {
            // The entry each identity was first recorded at.
            let mut first: BTreeMap<Identity, usize> = BTreeMap::new();
            for (n, table) in tables.into_iter().enumerate() {
                let at = format!("kinds[{n}]");
                let identity = read_kind(table, &at, &mut problems);
                if let Some(identity) = &identity {
                    match first.get(identity) {
                        Some(earlier) => problems.refuse(
                            &field(&at, "kind"),
                            format!(
                                "`{identity}` is recorded at kinds[{earlier}] too, and a lock records an identity at most once"
                            ),
                        ),
                        None => {
                            first.insert(identity.clone(), n);
                        }
                    }
                }
                let hash = read_hash(table, &at, &mut problems);
                if let (Some(identity), Some(hash)) = (identity, hash) {
                    entries.insert(
                        identity,
                        LockEntry {
                            hash,
                            sections: unnamed(table, &ENTRY_FIELDS),
                        },
                    );
                }
            }
        }
        None => problems.refuse(
            "kinds",
            "must be an array of tables, one per identity recorded",
        ),
    }
    problems.finish(Lock {
        entries,
        sections: unnamed(&root, &["kinds"]),
    })
}

/// An entry's `kind`, a whole identity, or `None` where it is refused.
fn read_kind(table: &Table, at: &str, problems: &mut Problems) -> Option<Identity> {
    let text = required(table, "kind", "an identity", at, problems)?;
    match text.parse() {
        Ok(identity) => Some(identity),
        Err(refusal) => {
            problems.refuse(&field(at, "kind"), refusal.to_string());
            None
        }
    }
}

/// An entry's `hash`, a content hash, or `None` where it is refused.
fn read_hash(table: &Table, at: &str, problems: &mut Problems) -> Option<ContentHash> {
    let text = required(table, "hash", "a content hash", at, problems)?;
    match text.parse() {
        Ok(hash) => Some(hash),
        Err(refusal) => {
            problems.refuse(&field(at, "hash"), refusal.to_string());
            None
        }
    }
}

/// A string an entry must hold, `what` it names, or `None` where it is
/// missing or not a string.
fn required<'a>(
    table: &'a Table,
    name: &str,
    what: &str,
    at: &str,
    problems: &mut Problems,
) -> Option<&'a str> {
    match table.get(name) {
        Some(Value::String(text)) => Some(text.as_str()),
        None => {
            problems.refuse(
                &field(at, name),
                format!("is missing, and every entry of a lock has a {name}, {what}"),
            );
            None
        }
        Some(_) => {
            problems.refuse(&field(at, name), format!("must be a string, {what}"));
            None
        }
    }
}
