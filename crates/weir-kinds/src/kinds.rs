use std::collections::BTreeMap;
use std::time::Duration;

use toml::{Table, Value};

use crate::deliverable::{Deliverable, End};
use crate::fields::{boolean, field, key, unnamed};
use crate::quiet;
use crate::refusal::{not_toml, Problems, Refusal};

/// The fields of a kind the format names; every other key is a section.
const KIND_FIELDS: [&str; 5] = ["version", "branch", "starts_unready", "lands", "stages"];

/// The fields of a stage the format names; every other key is a section.
const STAGE_FIELDS: [&str; 9] = [
    "name",
    "prompt",
    "deliverables",
    "conversation",
    "agents",
    "wait",
    "quiet",
    "on_fail",
    "max_send_backs",
];

/// A declaration read: every kind it declares, and every section beside
/// them (81).
#[derive(Debug, Clone, PartialEq)]
pub struct Declaration {
    /// Every kind under the `kinds` table, by name, in name order; none
    /// when there is no `kinds` table (51, 52).
    pub kinds: BTreeMap<String, Kind>,
    /// Every top-level key but `kinds`, unread and as written (56).
    pub sections: Table,
}

/// A kind: its version and its stages in order, with how its places are
/// run (51, 52, C.4).
#[derive(Debug, Clone, PartialEq)]
pub struct Kind {
    /// The kind's name, its key under `kinds`.
    pub name: String,
    /// Its version, at least 1; 1 when left out (51, 57).
    pub version: u64,
    /// The prefix of its places' branches; the kind's name when left out
    /// (52, C.4).
    pub branch: String,
    /// Whether its first stage starts in a place that could not be made
    /// ready; false when left out (52).
    pub starts_unready: bool,
    /// Whether its work is submitted after the last stage; false when left
    /// out (52).
    pub lands: bool,
    /// Its stages, at least one, in the order written (51).
    pub stages: Vec<Stage>,
    /// Every key of the kind the format does not name, unread and as
    /// written, an elaboration kind's `elaboration` section among them
    /// (55, 56).
    pub sections: Table,
}

/// A stage of a kind (53, C.4).
#[derive(Debug, Clone, PartialEq)]
pub struct Stage {
    /// Its name, unique in its kind (52, 53).
    pub name: String,
    /// Its prompt as written; resolving it against the declaration's file
    /// is the reader's (53, C.4).
    pub prompt: String,
    /// Its one deliverable, or that it is a conversation (51, 54).
    pub end: End,
    /// Its agents, each a table of its settings as written, since an
    /// agent's settings are its runner's; none when left out (53).
    pub agents: Vec<Table>,
    /// Whether it waits on its caller's word; false when left out (53).
    pub wait: bool,
    /// How long its agent may be quiet before the stage is stuck; `None`
    /// when left out, since each reader keeps its own default (53).
    pub quiet: Option<Duration>,
    /// The stage of its kind a failed submission sends a place back to;
    /// `None` when left out (52).
    pub on_fail: Option<String>,
    /// How many times a place may go back before it waits on its caller;
    /// `None` when left out (52).
    pub max_send_backs: Option<u64>,
    /// Every key of the stage the format does not name, unread and as
    /// written (56).
    pub sections: Table,
}

/// `read-kinds` (81): every kind a declaration's text declares, each with
/// its stages and its sections, or the declaration refused whole, naming
/// every field that fails it (57).
///
/// A table or field the format does not name is returned unread and never
/// fails the read (56). The same text always gives the same result, and
/// nothing is read but the text (58, 82).
pub fn read_kinds(text: &str) -> Result<Declaration, Refusal> {
    let root: Table = text.parse().map_err(|error| not_toml(text, &error))?;
    let mut problems = Problems::default();
    let mut kinds = BTreeMap::new();
    match root.get("kinds") {
        None => {}
        Some(Value::Table(table)) => {
            for (name, kind) in table {
                let at = format!("kinds.{}", key(name));
                match kind {
                    Value::Table(kind) => {
                        kinds.insert(name.clone(), read_kind(name, kind, &at, &mut problems));
                    }
                    _ => problems.refuse(&at, "must be a table"),
                }
            }
        }
        Some(_) => problems.refuse("kinds", "must be a table of kinds"),
    }
    problems.finish(Declaration {
        kinds,
        sections: unnamed(&root, &["kinds"]),
    })
}

fn read_kind(name: &str, kind: &Table, at: &str, problems: &mut Problems) -> Kind {
    let version = match kind.get("version") {
        None => 1,
        Some(Value::Integer(version)) if *version >= 1 => version.unsigned_abs(),
        Some(_) => {
            problems.refuse(&field(at, "version"), "must be an integer of at least 1");
            1
        }
    };
    let branch = match kind.get("branch") {
        None => name.to_owned(),
        Some(Value::String(branch)) if !branch.is_empty() => branch.clone(),
        Some(_) => {
            problems.refuse(&field(at, "branch"), "must be a non-empty string");
            name.to_owned()
        }
    };
    Kind {
        name: name.to_owned(),
        version,
        branch,
        starts_unready: boolean(kind, "starts_unready", at, problems),
        lands: boolean(kind, "lands", at, problems),
        stages: read_stages(kind, at, problems),
        sections: unnamed(kind, &KIND_FIELDS),
    }
}

fn read_stages(kind: &Table, at: &str, problems: &mut Problems) -> Vec<Stage> {
    let here = field(at, "stages");
    let tables = match kind.get("stages") {
        None => {
            problems.refuse(&here, "is missing, and a kind has at least one stage");
            return Vec::new();
        }
        Some(Value::Array(items)) if items.is_empty() => {
            problems.refuse(&here, "is empty, and a kind has at least one stage");
            return Vec::new();
        }
        Some(Value::Array(items)) => items
            .iter()
            .map(Value::as_table)
            .collect::<Option<Vec<_>>>(),
        Some(_) => None,
    };
    let Some(tables) = tables else {
        problems.refuse(&here, "must be an array of tables, one per stage");
        return Vec::new();
    };

    // A send-back names the stage, which a name two stages share does not
    // do (52): the second use of a name is refused.
    let mut stages: Vec<Stage> = Vec::with_capacity(tables.len());
    for (n, table) in tables.into_iter().enumerate() {
        let at = format!("{here}[{n}]");
        let stage = read_stage(table, &at, problems);
        if let Some(first) = stages
            .iter()
            .position(|s| !stage.name.is_empty() && s.name == stage.name)
        {
            problems.refuse(
                &field(&at, "name"),
                format!("`{}` is the name of stages[{first}] too", stage.name),
            );
        }
        stages.push(stage);
    }
    for (n, stage) in stages.iter().enumerate() {
        if let Some(target) = &stage.on_fail {
            if target.is_empty() || !stages.iter().any(|s| &s.name == target) {
                problems.refuse(
                    &field(&format!("{here}[{n}]"), "on_fail"),
                    format!("`{target}` names no stage of this kind"),
                );
            }
        }
    }
    stages
}

fn read_stage(stage: &Table, at: &str, problems: &mut Problems) -> Stage {
    let name = required(stage, "name", at, problems);
    let prompt = required(stage, "prompt", at, problems);
    let end = read_end(stage, at, problems);
    let agents = match stage.get("agents") {
        None => Some(Vec::new()),
        Some(Value::Array(items)) => items
            .iter()
            .map(|item| item.as_table().cloned())
            .collect::<Option<Vec<_>>>(),
        Some(_) => None,
    }
    .unwrap_or_else(|| {
        problems.refuse(
            &field(at, "agents"),
            "must be an array of tables, one per agent",
        );
        Vec::new()
    });
    let wait = boolean(stage, "wait", at, problems);
    let quiet = match stage.get("quiet") {
        None => None,
        Some(Value::String(text)) => {
            let quiet = quiet::parse(text);
            if quiet.is_none() {
                problems.refuse(
                    &field(at, "quiet"),
                    format!("`{text}` is not a duration such as 45m, 1h30m or 90s"),
                );
            }
            quiet
        }
        Some(_) => {
            problems.refuse(
                &field(at, "quiet"),
                "must be a duration written as a string, such as \"45m\"",
            );
            None
        }
    };
    let on_fail = match stage.get("on_fail") {
        None => None,
        Some(Value::String(target)) => Some(target.clone()),
        Some(_) => {
            problems.refuse(
                &field(at, "on_fail"),
                "must be the name of a stage of this kind",
            );
            None
        }
    };
    let max_send_backs = match stage.get("max_send_backs") {
        None => None,
        Some(Value::Integer(times)) if *times >= 0 => Some(times.unsigned_abs()),
        Some(_) => {
            problems.refuse(
                &field(at, "max_send_backs"),
                "must be an integer of at least 0",
            );
            None
        }
    };
    Stage {
        name,
        prompt,
        end,
        agents,
        wait,
        quiet,
        on_fail,
        max_send_backs,
        sections: unnamed(stage, &STAGE_FIELDS),
    }
}

/// A stage's name or prompt: a non-empty string it must hold (57).
fn required(stage: &Table, name: &str, at: &str, problems: &mut Problems) -> String {
    match stage.get(name) {
        Some(Value::String(text)) if !text.is_empty() => text.clone(),
        None => {
            problems.refuse(
                &field(at, name),
                format!("is missing, and every stage has a {name}"),
            );
            String::new()
        }
        Some(_) => {
            problems.refuse(&field(at, name), "must be a non-empty string");
            String::new()
        }
    }
}

/// A stage's end: exactly one deliverable, or a conversation with none
/// (51, 54).
fn read_end(stage: &Table, at: &str, problems: &mut Problems) -> End {
    let here = field(at, "deliverables");
    let conversation = match stage.get("conversation") {
        None => Some(false),
        Some(Value::Boolean(flag)) => Some(*flag),
        Some(_) => {
            problems.refuse(&field(at, "conversation"), "must be true or false");
            None
        }
    };
    match (conversation, stage.get("deliverables")) {
        (Some(true), Some(_)) => {
            problems.refuse(&here, "is declared on a conversation, which has none");
            End::Conversation
        }
        (Some(true), None) => End::Conversation,
        (Some(false), None) => {
            problems.refuse(
                &here,
                "is missing, and a stage that is not a conversation has exactly one deliverable",
            );
            End::Conversation
        }
        (_, Some(deliverables)) => read_deliverables(deliverables, &here, problems),
        // The conversation flag is refused already, and with it the end.
        (None, None) => End::Conversation,
    }
}

fn read_deliverables(deliverables: &Value, here: &str, problems: &mut Problems) -> End {
    let spellings = deliverables
        .as_array()
        .and_then(|items| items.iter().map(Value::as_str).collect::<Option<Vec<_>>>());
    let reason = match spellings.as_deref() {
        None => "must be an array of strings".to_owned(),
        Some([spelling]) => match Deliverable::from_spelling(spelling) {
            Ok(deliverable) => return End::Deliverable(deliverable),
            Err(reason) => reason,
        },
        Some([]) => "is empty, and a stage that is not a conversation has exactly one deliverable"
            .to_owned(),
        Some(many) => format!(
            "names {} deliverables, and a stage has exactly one",
            many.len()
        ),
    };
    problems.refuse(here, reason);
    End::Conversation
}
