//! The `weir` command line: the caller that runs the crates, holding no
//! rule of its own (section 9; core.1). `weir check <declaration>` reads a
//! declaration of kinds (51 to 58). `weir capture folder` and `weir
//! capture meeting` run the built-in adapters as a host runs any adapter,
//! with the arguments a host appends, `--raw-dir <dir>` and `--cursor <c>`,
//! and print their lines (C.1; core.34). `weir import signals` prints the
//! import lines of a body read before its inbox existed, for a host's
//! import operation, and names on standard error what it could not carry
//! (63, 75a). What it says of the body comes from the crate that reads it
//! (core.2). `weir conform <host command>` runs a host against every host
//! case of the pack and says which it passed (65, 87; C.7).

mod conform;

use std::io::{self, Write};
use std::path::{Path, PathBuf};
use std::process::ExitCode;

use clap::{Parser, Subcommand};
use weir_adapters::{folder, meeting, signals_folder};
use weir_capture::{ImportLine, Line};
use weir_kinds::{read_elaboration, read_kinds, End, Problem};
use weir_pack::Pack;

fn usage() -> String {
    format!(
        "usage:\n  \
         weir check <declaration>\n      \
         list the kinds a declaration holds, or name every field that fails it\n  \
         weir capture folder <dir> --raw-dir <dir> [--cursor <c>]\n      \
         offer one capture per file in <dir>, keyed by the folder and the file's content\n  \
         weir capture meeting <file> --raw-dir <dir> [--cursor <c>]\n      \
         offer one capture for the meeting file <file>\n  \
         weir import signals <root> [--moves <file>]\n      \
         {}\n  \
         weir conform [--pack <dir>] <host command> [<arg>...]\n      \
         run a host against every host case of the pack, one line a case\n",
        signals_folder::HELP
    )
}

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

/// How a command ended: its exit status.
enum Exit {
    Done,
    /// A run that could not begin, or did not finish, said why on standard
    /// error.
    Failed,
    /// The command line asked for no command weir has.
    Usage,
}

impl From<Exit> for ExitCode {
    fn from(exit: Exit) -> ExitCode {
        match exit {
            Exit::Done => ExitCode::SUCCESS,
            Exit::Failed => ExitCode::from(1),
            Exit::Usage => {
                eprint!("{}", usage());
                ExitCode::from(2)
            }
        }
    }
}

/// `weir check` is read by its parser; a built-in adapter's arguments and
/// the import's are read by hand, the few a host appends (C.1).
fn main() -> ExitCode {
    let args: Vec<String> = std::env::args().skip(1).collect();
    let args: Vec<&str> = args.iter().map(String::as_str).collect();
    match args.as_slice() {
        ["check", ..] => match Cli::parse().command {
            Command::Check { declaration } => check(&declaration),
        },
        ["capture", adapter @ ("folder" | "meeting"), rest @ ..] => capture(adapter, rest).into(),
        ["import", "signals", rest @ ..] => import_signals(rest).into(),
        ["conform", rest @ ..] => conform(rest).into(),
        _ => Exit::Usage.into(),
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

/// A built-in adapter's run: its path, then the options a host appends.
/// The cursor is taken and never read, since neither adapter keeps one (8).
fn capture(adapter: &str, args: &[&str]) -> Exit {
    let command = format!("capture {adapter}");
    let mut path = None;
    let mut raw_dir = None;
    let mut args = args.iter();
    while let Some(arg) = args.next() {
        match *arg {
            "--raw-dir" => match args.next() {
                Some(dir) => raw_dir = Some(PathBuf::from(dir)),
                None => return Exit::Usage,
            },
            "--cursor" => {
                if args.next().is_none() {
                    return Exit::Usage;
                }
            }
            option if option.starts_with("--") => return Exit::Usage,
            given if path.is_none() => path = Some(PathBuf::from(given)),
            _ => return Exit::Usage,
        }
    }
    let Some(path) = path else {
        return Exit::Usage;
    };
    let Some(raw_dir) = raw_dir else {
        return failed(
            &command,
            "--raw-dir <dir> is required: the raw place a host hands the run",
        );
    };
    let run: fn(&Path, &Path) -> Result<Vec<Line>, String> = match adapter {
        "folder" => folder::run,
        _ => meeting::run,
    };
    match run(&path, &raw_dir) {
        Ok(lines) => print(&command, lines.iter().map(Line::to_json)),
        Err(why) => failed(&command, &why),
    }
}

/// The import lines of the body at its root, its moves those of the file
/// `--moves` names when it names one, and on standard error each capture
/// not read whole and each move not carried, one a line; it fails when any
/// capture was not read, once the rest is printed (15).
fn import_signals(args: &[&str]) -> Exit {
    let command = "import signals";
    let mut root = None;
    let mut moves = None;
    let mut args = args.iter();
    while let Some(arg) = args.next() {
        match *arg {
            "--moves" => match args.next() {
                Some(file) => moves = Some(PathBuf::from(file)),
                None => return Exit::Usage,
            },
            option if option.starts_with("--") => return Exit::Usage,
            given if root.is_none() => root = Some(PathBuf::from(given)),
            _ => return Exit::Usage,
        }
    }
    let Some(root) = root else {
        return Exit::Usage;
    };
    let reading = match signals_folder::read(&root, moves.as_deref()) {
        Ok(reading) => reading,
        Err(why) => return failed(command, &why),
    };
    let printed = print(command, reading.lines.iter().map(ImportLine::to_json));
    for not_read in &reading.not_read {
        eprintln!("weir {command}: {not_read}");
    }
    for not_carried in &reading.not_carried {
        eprintln!("weir {command}: {not_carried}");
    }
    match printed {
        Exit::Done if reading.not_read.is_empty() => Exit::Done,
        _ => Exit::Failed,
    }
}

/// `weir conform [--pack <dir>] <program> [<arg>...]`: the options first,
/// and the first word that is none of them begins the host command, run
/// with no shell (C.7). Exits 0 when every case passed, 1 when any failed,
/// and 2 when no host command is given (65, 87).
fn conform(args: &[&str]) -> Exit {
    let command = "conform";
    let mut pack_dir = None;
    let mut args = args;
    loop {
        match args {
            ["--pack", dir, rest @ ..] => {
                pack_dir = Some(PathBuf::from(dir));
                args = rest;
            }
            ["--pack"] => return Exit::Usage,
            _ => break,
        }
    }
    let [program, host_args @ ..] = args else {
        return Exit::Usage;
    };
    let pack = match pack_dir {
        Some(dir) => match Pack::read(&dir) {
            Ok(pack) => pack,
            Err(why) => return failed(command, &why),
        },
        None => Pack::embedded(),
    };
    match conform::conform(&pack, program, host_args) {
        Ok(true) => Exit::Done,
        Ok(false) => Exit::Failed,
        Err(why) => failed(command, &why),
    }
}

/// Says on standard error, in one line, why the command did not finish.
fn failed(command: &str, why: &str) -> Exit {
    eprintln!("weir {command}: {}", why.replace('\n', " "));
    Exit::Failed
}

/// Prints `lines` to standard output, one a line.
fn print(command: &str, mut lines: impl Iterator<Item = String>) -> Exit {
    let mut out = io::BufWriter::new(io::stdout().lock());
    let written = lines
        .try_for_each(|line| writeln!(out, "{line}"))
        .and_then(|()| out.flush());
    match written {
        Ok(()) => Exit::Done,
        Err(error) => failed(command, &format!("standard output: {error}")),
    }
}
