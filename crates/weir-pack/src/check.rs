//! The pack's check (67; S16): every case of every shape cites the clauses
//! it exercises, every number it cites names a clause of the index, and
//! every counted clause, those of A.1 to A.10 and 71, is cited by some
//! case. It reads each case's `satisfies` alone, so a shape's format may
//! grow without the check changing, and it reads nothing but the pack's
//! files (68; core.35).

use std::collections::{BTreeMap, BTreeSet};
use std::fmt;

use toml::{Table, Value};

use crate::clauses::{read_index, Clause, INDEX};
use crate::pack::{Case, Pack, Shape};

/// The file of a case the check reads.
pub const CASE_FILE: &str = "case.toml";

/// A pack that passed: how many cases of each shape it read.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Checked {
    pub read: BTreeMap<Shape, usize>,
}

impl fmt::Display for Checked {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let each: Vec<String> = self
            .read
            .iter()
            .map(|(shape, count)| format!("{count} {shape}"))
            .collect();
        write!(f, "the pack passed, its cases read: {}", each.join(", "))
    }
}

/// One thing the check fails on, each named by its place in the pack.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Fault {
    /// The index is missing or does not read.
    Index(String),
    /// A folder or file of the pack's root that is no shape.
    NoShape(String),
    /// A file lying in a shape's folder, which is no case's folder.
    NoCase(String),
    /// A case's folder with no `case.toml`.
    NoCaseFile(String),
    /// A case whose `case.toml` is not TOML.
    NotToml { case: String, why: String },
    /// A case that cites nothing: no `satisfies`, or an empty one.
    CitesNothing(String),
    /// A citation that is no clause's number.
    NotACitation { case: String, why: String },
    /// A cited number the index does not hold (S16).
    NamesNoClause { case: String, clause: Clause },
    /// A counted clause no case cites (S16).
    Uncited(Clause),
}

impl fmt::Display for Fault {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Fault::Index(why) => write!(f, "pack/{INDEX}: {why}"),
            Fault::NoShape(path) => write!(
                f,
                "pack/{path}: is no shape of the pack, which are {}",
                Shape::ALL.map(Shape::as_str).join(", ")
            ),
            Fault::NoCase(path) => write!(
                f,
                "pack/{path}: lies in its shape's folder, and every case is a folder there"
            ),
            Fault::NoCaseFile(case) => write!(f, "pack/{case}: holds no {CASE_FILE}"),
            Fault::NotToml { case, why } => write!(f, "pack/{case}/{CASE_FILE}: {why}"),
            Fault::CitesNothing(case) => write!(
                f,
                "pack/{case}: cites no clause, and every case names in `satisfies` the clauses \
                 it exercises"
            ),
            Fault::NotACitation { case, why } => write!(f, "pack/{case}: `satisfies`: {why}"),
            Fault::NamesNoClause { case, clause } => write!(
                f,
                "pack/{case}: cites {clause}, which names no clause of the statement"
            ),
            Fault::Uncited(clause) => write!(f, "{clause}: is cited by no case"),
        }
    }
}

/// The clauses `case` cites in its `case.toml`'s `satisfies`, as the check
/// reads them, or why they do not read.
pub fn satisfies(case: &Case) -> Result<Vec<Clause>, String> {
    let id = case.id();
    let table: Table = case
        .text(CASE_FILE)?
        .parse()
        .map_err(|error: toml::de::Error| format!("{id}: {}", error.message()))?;
    match table.get("satisfies") {
        Some(Value::Array(citations)) => citations
            .iter()
            .map(|citation| Clause::cited(citation).map_err(|why| format!("{id}: {why}")))
            .collect(),
        _ => Err(format!("{id}: `satisfies` is no list of clauses")),
    }
}

/// The index's fault, its words naming the file once.
fn index_fault(why: String) -> Fault {
    let prefix = format!("{INDEX}: ");
    Fault::Index(why.strip_prefix(&prefix).map(str::to_string).unwrap_or(why))
}

/// The check of `pack`: how many cases of each shape it read, or every
/// fault found (67).
pub fn check(pack: &Pack) -> Result<Checked, Vec<Fault>> {
    let mut faults = Vec::new();
    let index = match pack.file(INDEX).map(std::str::from_utf8) {
        None => {
            faults.push(Fault::Index("is missing".to_string()));
            None
        }
        Some(Err(_)) => {
            faults.push(Fault::Index("is not UTF-8".to_string()));
            None
        }
        Some(Ok(text)) => match read_index(text) {
            Ok(index) => Some(index),
            Err(why) => {
                faults.push(index_fault(why));
                None
            }
        },
    };
    let mut no_shape = BTreeSet::new();
    for path in pack.files().keys() {
        if path == INDEX {
            continue;
        }
        match path.split_once('/') {
            None => {
                no_shape.insert(path.clone());
            }
            Some((first, rest)) => {
                if Shape::parse(first).is_none() {
                    no_shape.insert(first.to_string());
                } else if !rest.contains('/') {
                    faults.push(Fault::NoCase(path.clone()));
                }
            }
        }
    }
    faults.extend(no_shape.into_iter().map(Fault::NoShape));

    let mut read = BTreeMap::new();
    let mut cited = BTreeSet::new();
    for shape in Shape::ALL {
        let cases = pack.cases(shape);
        read.insert(shape, cases.len());
        for case in cases {
            let id = case.id();
            let Some(bytes) = case.file(CASE_FILE) else {
                faults.push(Fault::NoCaseFile(id));
                continue;
            };
            let table: Table = match std::str::from_utf8(bytes)
                .map_err(|_| "is not UTF-8".to_string())
                .and_then(|text| {
                    text.parse()
                        .map_err(|error: toml::de::Error| error.to_string())
                }) {
                Ok(table) => table,
                Err(why) => {
                    faults.push(Fault::NotToml { case: id, why });
                    continue;
                }
            };
            let citations = match table.get("satisfies") {
                None => {
                    faults.push(Fault::CitesNothing(id));
                    continue;
                }
                Some(Value::Array(citations)) if citations.is_empty() => {
                    faults.push(Fault::CitesNothing(id));
                    continue;
                }
                Some(Value::Array(citations)) => citations,
                Some(_) => {
                    faults.push(Fault::NotACitation {
                        case: id,
                        why: "is not a list of clauses".to_string(),
                    });
                    continue;
                }
            };
            for citation in citations {
                match Clause::cited(citation) {
                    Err(why) => faults.push(Fault::NotACitation {
                        case: id.clone(),
                        why,
                    }),
                    Ok(clause) => match &index {
                        Some(index) if !index.holds(&clause) => faults.push(Fault::NamesNoClause {
                            case: id.clone(),
                            clause,
                        }),
                        _ => {
                            cited.insert(clause);
                        }
                    },
                }
            }
        }
    }
    if let Some(index) = &index {
        match index.counted() {
            Ok(counted) => faults.extend(counted.difference(&cited).cloned().map(Fault::Uncited)),
            Err(why) => faults.push(index_fault(why)),
        }
    }
    if faults.is_empty() {
        Ok(Checked { read })
    } else {
        Err(faults)
    }
}
