//! An adapter's declaration: how it is started, the shape of its
//! configuration, and the secrets it needs, by name (11, 72; C.1). The
//! shape is a JSON Schema, draft 2020-12 (section 10, question 8).

use std::ffi::OsString;
use std::fmt;
use std::path::Path;

use toml::{Table, Value};

/// The environment variable that names the file holding a run's
/// configuration (C.1).
pub const CAPTURE_CONFIG: &str = "CAPTURE_CONFIG";

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

/// How a run is started: the command line, and the names of the
/// environment variables the host places, holding no value (11, 73;
/// core.3).
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Invocation {
    pub args: Vec<OsString>,
    /// [`CAPTURE_CONFIG`], then each secret's name, as declared.
    pub env: Vec<String>,
}

impl Declaration {
    /// The run of this adapter with the raw place `raw_dir` and, when one
    /// is kept, the cursor of its last complete run (8; C.1). It starts
    /// nothing.
    pub fn invocation(&self, raw_dir: &Path, cursor: Option<&str>) -> Invocation {
        let mut args: Vec<OsString> = self.command.iter().map(OsString::from).collect();
        args.push("--raw-dir".into());
        args.push(raw_dir.into());
        if let Some(cursor) = cursor {
            args.push("--cursor".into());
            args.push(cursor.into());
        }
        let env = std::iter::once(CAPTURE_CONFIG.to_string())
            .chain(self.secrets.iter().cloned())
            .collect();
        Invocation { args, env }
    }
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

/// Why a configuration was refused.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ConfigError {
    /// The schema is not a draft 2020-12 schema, or refers to something it
    /// does not hold: nothing it refers to is fetched (core.1).
    Schema(String),
    /// The configuration fails the schema, at each place named.
    Fails(Vec<ConfigFailure>),
}

/// One place a configuration fails its schema: the JSON Pointer of the
/// instance, empty for the whole, and what fails there.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ConfigFailure {
    pub path: String,
    pub message: String,
}

impl fmt::Display for ConfigFailure {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if self.path.is_empty() {
            write!(f, "the configuration: {}", self.message)
        } else {
            write!(f, "`{}`: {}", self.path, self.message)
        }
    }
}

impl fmt::Display for ConfigError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ConfigError::Schema(reason) => write!(f, "the schema cannot be used: {reason}"),
            ConfigError::Fails(failures) => {
                let failures: Vec<String> = failures.iter().map(ToString::to_string).collect();
                write!(
                    f,
                    "the configuration fails its schema: {}",
                    failures.join("; ")
                )
            }
        }
    }
}

impl std::error::Error for ConfigError {}

/// Checks `config` against `schema` as draft 2020-12, naming every place
/// it fails. The crate's `jsonschema` is built with none of its features
/// that resolve a reference by fetching it, by network or from a file.
pub fn check_config(
    schema: &serde_json::Value,
    config: &serde_json::Value,
) -> Result<(), ConfigError> {
    let validator = jsonschema::draft202012::new(schema)
        .map_err(|error| ConfigError::Schema(error.to_string()))?;
    let failures: Vec<ConfigFailure> = validator
        .iter_errors(config)
        .map(|error| ConfigFailure {
            path: error.instance_path().to_string(),
            message: error.to_string(),
        })
        .collect();
    if failures.is_empty() {
        Ok(())
    } else {
        Err(ConfigError::Fails(failures))
    }
}

#[cfg(test)]
mod tests {
    use serde_json::json;

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

    fn folder_schema() -> serde_json::Value {
        json!({
            "$schema": "https://json-schema.org/draft/2020-12/schema",
            "type": "object",
            "properties": {"folder": {"type": "string"}, "depth": {"type": "integer"}},
            "required": ["folder"]
        })
    }

    #[test]
    fn a_configuration_missing_a_required_property_is_refused() {
        let Err(ConfigError::Fails(failures)) = check_config(&folder_schema(), &json!({})) else {
            panic!("a configuration with no folder was taken");
        };
        assert_eq!(failures.len(), 1);
        assert_eq!(failures[0].path, "");
        assert!(failures[0].message.contains("folder"), "{}", failures[0]);
    }

    #[test]
    fn a_configuration_that_fits_is_taken() {
        assert_eq!(
            check_config(&folder_schema(), &json!({"folder": "/tmp/drop"})),
            Ok(())
        );
    }

    #[test]
    fn every_place_a_configuration_fails_is_named() {
        let Err(ConfigError::Fails(failures)) =
            check_config(&folder_schema(), &json!({"folder": 3, "depth": "deep"}))
        else {
            panic!("a configuration of the wrong shape was taken");
        };
        let mut paths: Vec<&str> = failures.iter().map(|f| f.path.as_str()).collect();
        paths.sort();
        assert_eq!(paths, ["/depth", "/folder"]);
    }

    #[test]
    fn a_reference_is_never_fetched() {
        // A schema that, fetched, would take any object: the check fails
        // because nothing is fetched.
        let schema = json!({"$ref": "https://json.schemastore.org/package.json"});
        let Err(ConfigError::Schema(reason)) = check_config(&schema, &json!({})) else {
            panic!("a remote $ref was resolved");
        };
        assert!(reason.contains("json.schemastore.org"), "{reason}");
    }

    fn meetings() -> Declaration {
        read_declarations(
            r#"
            [adapter.meetings]
            command = ["meetings", "capture"]
            config = "config.schema.json"
            secrets = ["MEETINGS_TOKEN"]
            "#,
        )
        .unwrap()
        .remove(0)
    }

    fn words(args: &[OsString]) -> Vec<&str> {
        args.iter().map(|arg| arg.to_str().unwrap()).collect()
    }

    #[test]
    fn a_run_with_a_cursor() {
        let run = meetings().invocation(Path::new("/r/1"), Some("c9"));
        assert_eq!(
            words(&run.args),
            ["meetings", "capture", "--raw-dir", "/r/1", "--cursor", "c9"]
        );
        assert_eq!(run.env, ["CAPTURE_CONFIG", "MEETINGS_TOKEN"]);
    }

    #[test]
    fn a_first_run_has_no_cursor() {
        let run = meetings().invocation(Path::new("/r/1"), None);
        assert_eq!(
            words(&run.args),
            ["meetings", "capture", "--raw-dir", "/r/1"]
        );
    }
}
