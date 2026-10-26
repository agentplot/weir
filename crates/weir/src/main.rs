//! The `weir` command line, a caller of the crates (section 9, core.1).
//! It holds one command for now: `weir check <declaration>`.

use std::path::{Path, PathBuf};
use std::process::ExitCode;

use clap::{Parser, Subcommand};
use weir_kinds::{read_elaboration, read_kinds, End, Problem};

#[derive(Parser)]
#[command(name = "weir", version, about = "The weir command line")]
struct Cli {
    #[command(subcommand)]
    command: Command,
}

#[derive(Subcommand)]
enum Command {
    /// Read a declaration of kinds, its elaboration sections included, and
    /// list its kinds, or name every field that fails it (51 to 58).
    Check {
        /// The declaration's file, which is read and never written (58).
        declaration: PathBuf,
    },
}

fn main() -> ExitCode {
    match Cli::parse().command {
        Command::Check { declaration } => check(&declaration),
    }
}

/// `weir check`: the kinds listed on standard output and exit 0; each
/// problem as `<field>: <reason>` on standard error and exit 1; a file it
/// cannot read named on standard error and exit 2.
fn check(path: &Path) -> ExitCode {
    let text = match std::fs::read_to_string(path) {
        Ok(text) => text,
        Err(error) => {
            eprintln!("weir check: {}: {error}", path.display());
            return ExitCode::from(2);
        }
    };
    match listing(&text) {
        Ok(lines) => {
            for line in lines {
                println!("{line}");
            }
            ExitCode::SUCCESS
        }
        Err(problems) => {
            for problem in problems {
                eprintln!("{problem}");
            }
            ExitCode::from(1)
        }
    }
}

/// A declaration's kinds in name order, each `<name>@<version>` with a line
/// per stage and one for an elaboration kind's ending; or every problem of
/// the kinds and of their elaboration sections, gathered into one refusal
/// (57, 81).
fn listing(text: &str) -> Result<Vec<String>, Vec<Problem>> {
    let declaration = read_kinds(text).map_err(|refusal| refusal.problems)?;
    let mut lines = Vec::new();
    let mut problems = Vec::new();
    for kind in declaration.kinds.values() {
        lines.push(format!("{}@{}", kind.name, kind.version));
        for stage in &kind.stages {
            lines.push(match &stage.end {
                End::Deliverable(deliverable) => format!("  {}: {deliverable}", stage.name),
                End::Conversation => format!("  {}: conversation", stage.name),
            });
        }
        match read_elaboration(kind) {
            Ok(None) => {}
            Ok(Some(elaboration)) => lines.push(match elaboration.ending {
                Some(ending) => format!("  elaboration: {ending}"),
                None => "  elaboration: no ending".to_owned(),
            }),
            Err(refusal) => problems.extend(refusal.problems),
        }
    }
    if problems.is_empty() {
        Ok(lines)
    } else {
        Err(problems)
    }
}
