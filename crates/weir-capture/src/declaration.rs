//! An adapter's declaration: how it is started, the shape of its
//! configuration, and the secrets it needs, by name (11, 72; C.1).

use std::fmt;

use toml::{Table, Value};

/// One `[adapter.<name>]` table.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Declaration {
    pub name: String,
    /// The program and its arguments, never empty.
    pub command: Vec<String>,
    /// The path of the configuration's JSON Schema, beside the declaration.
    pub config: String,
    /// The secrets' names; a declaration holds no secret's value (core.3).
    pub secrets: Vec<String>,
}

/// A declaration refused, naming the field where there is one
/// (`adapter.x.command`).
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct DeclarationError {
    pub field: Option<String>,
    pub reason: String,
}

impl fmt::Display for DeclarationError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match &self.field {
            Some(field) => write!(f, "`{field}` {}", self.reason),
            None => f.write_str(&self.reason),
        }
    }
}

impl std::error::Error for DeclarationError {}

fn refused(field: impl Into<String>, reason: impl Into<String>) -> DeclarationError {
    DeclarationError {
        field: Some(field.into()),
        reason: reason.into(),
    }
}

/// Every `[adapter.<name>]` table of `text`, in name order, each read
/// whole; a table missing a field, or holding one of the wrong shape, is
/// refused naming it. Every other table and field is ignored.
pub fn read_declarations(text: &str) -> Result<Vec<Declaration>, DeclarationError> {
    let table: Table = text.parse().map_err(|error| DeclarationError {
        field: None,
        reason: format!("the declaration is not TOML: {error}"),
    })?;
    let adapters = match table.get("adapter") {
        None => return Ok(Vec::new()),
        Some(Value::Table(adapters)) => adapters,
        Some(_) => return Err(refused("adapter", "is not a table")),
    };
    adapters
        .iter()
        .map(|(name, value)| {
            let path = format!("adapter.{name}");
            let Value::Table(adapter) = value else {
                return Err(refused(path, "is not a table"));
            };
            let command = strings(adapter, "command", &path)?;
            if command.is_empty() {
                return Err(refused(format!("{path}.command"), "is an empty list"));
            }
            let config = match adapter.get("config") {
                None => return Err(refused(format!("{path}.config"), "is missing")),
                Some(Value::String(config)) if !config.is_empty() => config.clone(),
                Some(Value::String(_)) => {
                    return Err(refused(format!("{path}.config"), "is empty"))
                }
                Some(_) => return Err(refused(format!("{path}.config"), "is not a path")),
            };
            let secrets = strings(adapter, "secrets", &path)?;
            if let Some(n) = secrets.iter().position(String::is_empty) {
                return Err(refused(format!("{path}.secrets[{n}]"), "is empty"));
            }
            Ok(Declaration {
                name: name.clone(),
                command,
                config,
                secrets,
            })
        })
        .collect()
}

fn strings(adapter: &Table, name: &str, path: &str) -> Result<Vec<String>, DeclarationError> {
    let field = format!("{path}.{name}");
    let Some(value) = adapter.get(name) else {
        return Err(refused(field, "is missing"));
    };
    let Value::Array(items) = value else {
        return Err(refused(field, "is not a list of strings"));
    };
    items
        .iter()
        .enumerate()
        .map(|(n, item)| match item {
            Value::String(text) => Ok(text.clone()),
            _ => Err(refused(format!("{field}[{n}]"), "is not a string")),
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_declaration_is_read() {
        let declarations = read_declarations(
            r#"
            [adapter.meetings]
            command = ["meetings", "capture"]
            config = "config.schema.json"
            secrets = ["MEETINGS_TOKEN"]
            "#,
        )
        .unwrap();
        assert_eq!(
            declarations,
            [Declaration {
                name: "meetings".into(),
                command: vec!["meetings".into(), "capture".into()],
                config: "config.schema.json".into(),
                secrets: vec!["MEETINGS_TOKEN".into()],
            }]
        );
    }

    #[test]
    fn every_adapter_table_is_read_and_every_other_ignored() {
        let declarations = read_declarations(
            r#"
            [plugin]
            name = "eddy"

            [adapter.wispr]
            command = ["eddy", "capture", "wispr"]
            config = "wispr.schema.json"
            secrets = ["WISPR_TOKEN"]
            notes = "ignored"

            [adapter.files]
            command = ["eddy", "capture", "files"]
            config = "files.schema.json"
            secrets = []
            "#,
        )
        .unwrap();
        let names: Vec<&str> = declarations.iter().map(|d| d.name.as_str()).collect();
        assert_eq!(names, ["files", "wispr"]);
        assert!(declarations[0].secrets.is_empty());
        assert_eq!(read_declarations("[plugin]\nname = \"x\"\n"), Ok(vec![]));
    }

    #[test]
    fn a_declaration_with_no_command_is_refused() {
        let error = read_declarations(
            r#"
            [adapter.x]
            config = "config.schema.json"
            secrets = []
            "#,
        )
        .unwrap_err();
        assert_eq!(error.field.as_deref(), Some("adapter.x.command"));
        assert_eq!(error.to_string(), "`adapter.x.command` is missing");
    }

    #[test]
    fn a_field_of_the_wrong_shape_is_refused_naming_it() {
        for (table, field) in [
            (
                "command = []\nconfig = \"c.json\"\nsecrets = []",
                "adapter.x.command",
            ),
            (
                "command = \"x capture\"\nconfig = \"c.json\"\nsecrets = []",
                "adapter.x.command",
            ),
            (
                "command = [\"x\", 2]\nconfig = \"c.json\"\nsecrets = []",
                "adapter.x.command[1]",
            ),
            ("command = [\"x\"]\nsecrets = []", "adapter.x.config"),
            (
                "command = [\"x\"]\nconfig = 3\nsecrets = []",
                "adapter.x.config",
            ),
            (
                "command = [\"x\"]\nconfig = \"c.json\"",
                "adapter.x.secrets",
            ),
            (
                "command = [\"x\"]\nconfig = \"c.json\"\nsecrets = \"TOKEN\"",
                "adapter.x.secrets",
            ),
            (
                "command = [\"x\"]\nconfig = \"c.json\"\nsecrets = [\"\"]",
                "adapter.x.secrets[0]",
            ),
        ] {
            let error = read_declarations(&format!("[adapter.x]\n{table}\n")).unwrap_err();
            assert_eq!(error.field.as_deref(), Some(field), "{table}");
        }
        assert_eq!(
            read_declarations("adapter = 1")
                .unwrap_err()
                .field
                .as_deref(),
            Some("adapter")
        );
        assert_eq!(
            read_declarations("[adapter]\nx = 1")
                .unwrap_err()
                .field
                .as_deref(),
            Some("adapter.x")
        );
        assert_eq!(read_declarations("[adapter.x").unwrap_err().field, None);
    }
}
