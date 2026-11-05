//! A lock of kinds: every identity it records with the content hash of
//! its package, read by identity, spelled in identity order, and a
//! package checked against it (91, 93, C.4; S18; core.30, core.45).

use std::collections::BTreeMap;
use std::fmt;

use toml::{Table, Value};

use crate::fields::{field, unnamed};
use crate::hash::ContentHash;
use crate::identity::Identity;
use crate::name::fault;
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

impl Lock {
    /// What the lock says of a package held at `identity` with the content
    /// hash `hash`: recorded at that hash, or not recorded, or refused, where
    /// the lock records the identity at another hash, naming the kind and
    /// both hashes (91, S18; core.45). Whether a kind not recorded runs,
    /// and which locks are checked, are the reader's (91; core.32).
    pub fn check(&self, identity: &Identity, hash: &ContentHash) -> Result<Checked, Mismatch> {
        match self.entries.get(identity) {
            None => Ok(Checked::NotRecorded),
            Some(entry) if entry.hash == *hash => Ok(Checked::Recorded),
            Some(entry) => Err(Mismatch {
                identity: identity.clone(),
                locked: entry.hash.clone(),
                given: hash.clone(),
            }),
        }
    }
}

/// What a lock says of a package it does not refuse (91).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Checked {
    /// The lock records the package's identity at its content hash.
    Recorded,
    /// The lock records no entry for the package's identity.
    NotRecorded,
}

/// A package a lock refuses: the lock records its identity at a content
/// hash other than the package's (91, S18; core.45). Its
/// [`Display`](fmt::Display) names the kind and both hashes.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Mismatch {
    /// The identity the package is held at.
    pub identity: Identity,
    /// The content hash the lock records for it.
    pub locked: ContentHash,
    /// The package's own content hash.
    pub given: ContentHash,
}

impl fmt::Display for Mismatch {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "the lock records `{}` at `{}`, but its package's content hash is `{}`",
            self.identity, self.locked, self.given
        )
    }
}

impl std::error::Error for Mismatch {}

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

/// A lock's TOML in C.4's form, which [`read_lock`] reads back as `lock`:
/// its own unread keys, then one `[[kinds]]` table per entry in identity
/// order, `kind` then `hash`, then the entry's unread keys (C.4 as amended
/// at blueprints 361a599; core.30). Each key is a line of its own, its
/// value as TOML spells one, a table inline; a key the format names, put
/// among the unread keys, is not spelled there, since the field it names
/// is spelled in its place.
///
/// One lock always spells the same text, whatever order its entries were
/// read or added in. Writing the file is the reader's (91; core.1).
pub fn spell_lock(lock: &Lock) -> String {
    let mut tables = Vec::with_capacity(lock.entries.len() + 1);
    let own = lines(&lock.sections, &["kinds"]);
    if !own.is_empty() {
        tables.push(own);
    }
    for (identity, entry) in &lock.entries {
        tables.push(format!(
            "[[kinds]]\nkind = {}\nhash = {}\n{}",
            Value::from(identity.to_string()),
            Value::from(entry.hash.to_string()),
            lines(&entry.sections, &ENTRY_FIELDS),
        ));
    }
    tables.join("\n")
}

/// Every key of `table` but those `named`, one line each, `<key> =
/// <value>`.
fn lines(table: &Table, named: &[&str]) -> String {
    unnamed(table, named)
        .iter()
        .map(|(name, value)| format!("{} = {value}\n", spelled_key(name)))
        .collect()
}

/// A key as TOML spells it: bare where it is a name, else a basic string
/// with every `"`, `\` and control character escaped.
fn spelled_key(name: &str) -> String {
    if fault(name).is_none() {
        return name.to_owned();
    }
    let mut spelled = String::from('"');
    for c in name.chars() {
        match c {
            '"' => spelled.push_str("\\\""),
            '\\' => spelled.push_str("\\\\"),
            c if c.is_control() => spelled.push_str(&format!("\\u{:04X}", u32::from(c))),
            c => spelled.push(c),
        }
    }
    spelled.push('"');
    spelled
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
