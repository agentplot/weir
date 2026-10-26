//! A host built on the crates, for the pack's own tests: `weir conform` runs
//! it against every host case, and it keeps the contract (65, 87; C.7). It
//! takes the seam's flags, `--inbox <repository> --branch <name> --limit
//! <bytes>`, then `--raw-dir <dir>` for an intake, `--run <declaration>`
//! for a run or `--import`; takes lines through an inbox that writes the
//! branch with `weir-branch` and a bank under the folder `PACK_HOST_BANK`
//! names; imports with `InboxAt::import`; and prints `weir-report/1` last.
//!
//! Each fault a host can make is one environment variable, set by a test
//! that holds `weir conform` to failing it: `PACK_HOST_RAW_IN_RECORD` keeps
//! a capture's raw material in its record (S15), `PACK_HOST_RAW_BESIDE`
//! copies a raw file beside its record, `PACK_HOST_CURSOR_FROM_INCOMPLETE`
//! keeps the cursor of an incomplete run, `PACK_HOST_REFUSAL_REASON`
//! reports every refusal with another reason, and `PACK_HOST_NO_REPORT`
//! prints none.

use std::cell::RefCell;
use std::collections::{BTreeMap, BTreeSet};
use std::env;
use std::fs;
use std::io::{self, BufRead};
use std::path::{Path, PathBuf};
use std::process::{Command, ExitCode, Stdio};
use std::rc::Rc;
use std::time::Duration;

use time::OffsetDateTime;
use weir_branch::{Ask, Branch, Files, Tip};
use weir_capture::declaration::CAPTURE_CONFIG;
use weir_capture::rec;
use weir_capture::{
    check_config, content_hash, read_declarations, read_import_line, take, Bank, CameIn, Inbox,
    Key, RawFile, Refusal, Refused, Report, Take, Taken,
};
use weir_signals::{ImportedCapture, InboxAt};

/// The folder the host banks raw material and keeps its cursors under.
const BANK: &str = "PACK_HOST_BANK";

/// Who asks for each write the host makes (50).
const WHO: &str = "pack-host";

/// The faults the host makes, each when its variable is set.
struct Faults {
    raw_in_record: bool,
    raw_beside: bool,
    cursor_from_incomplete: bool,
    refusal_reason: bool,
    no_report: bool,
}

impl Faults {
    fn from_env() -> Faults {
        let set = |name: &str| env::var_os(name).is_some();
        Faults {
            raw_in_record: set("PACK_HOST_RAW_IN_RECORD"),
            raw_beside: set("PACK_HOST_RAW_BESIDE"),
            cursor_from_incomplete: set("PACK_HOST_CURSOR_FROM_INCOMPLETE"),
            refusal_reason: set("PACK_HOST_REFUSAL_REASON"),
            no_report: set("PACK_HOST_NO_REPORT"),
        }
    }

    /// The files of a capture's write, as a faulty host changes them.
    fn on_capture(&self, key: &str, raw: &[RawFile], files: &mut Files) {
        let record = format!("inbox/captures/{key}/capture.rec");
        if self.raw_in_record {
            if let (Some(text), Some(file)) = (files.get_mut(&record), raw.first()) {
                if let Ok(mut records) = rec::read(text) {
                    records[0].push("raw", String::from_utf8_lossy(&file.bytes));
                    *text = rec::write(&records);
                }
            }
        }
        if self.raw_beside {
            for file in raw {
                let name = file.path.rsplit('/').next().unwrap_or(&file.path);
                files.insert(
                    format!("inbox/captures/{key}/{name}"),
                    String::from_utf8_lossy(&file.bytes).into_owned(),
                );
            }
        }
    }
}

/// How the host was asked to take lines.
enum Mode {
    Intake(Option<PathBuf>),
    Run(PathBuf),
    Import,
}

struct Args {
    inbox: PathBuf,
    branch: String,
    limit: usize,
    mode: Mode,
}

fn parse(args: &[String]) -> Result<Args, String> {
    let mut inbox = None;
    let mut branch = None;
    let mut limit = None;
    let mut raw_dir = None;
    let mut run = None;
    let mut import = false;
    let mut args = args.iter();
    while let Some(arg) = args.next() {
        let mut value = || {
            args.next()
                .cloned()
                .ok_or_else(|| format!("{arg} takes a value"))
        };
        match arg.as_str() {
            "--inbox" => inbox = Some(PathBuf::from(value()?)),
            "--branch" => branch = Some(value()?),
            "--limit" => {
                let text = value()?;
                limit = Some(
                    text.parse()
                        .map_err(|_| format!("--limit is `{text}`, not a count of bytes"))?,
                );
            }
            "--raw-dir" => raw_dir = Some(PathBuf::from(value()?)),
            "--run" => run = Some(PathBuf::from(value()?)),
            "--import" => import = true,
            other => return Err(format!("`{other}` is no flag of the host")),
        }
    }
    let mode = match (raw_dir, run, import) {
        (raw_dir, None, false) => Mode::Intake(raw_dir),
        (None, Some(declaration), false) => Mode::Run(declaration),
        (None, None, true) => Mode::Import,
        _ => return Err("--raw-dir, --run and --import are each a way in of their own".into()),
    };
    Ok(Args {
        inbox: inbox.ok_or("--inbox <repository> is required")?,
        branch: branch.ok_or("--branch <name> is required")?,
        limit: limit.ok_or("--limit <bytes> is required")?,
        mode,
    })
}

fn main() -> ExitCode {
    let faults = Faults::from_env();
    match host(&faults) {
        Ok(_) if faults.no_report => {
            eprintln!("pack-host: no report, as PACK_HOST_NO_REPORT asks");
            ExitCode::from(1)
        }
        Ok(mut report) => {
            if faults.refusal_reason {
                for refusal in &mut report.refused {
                    refusal.reason = "refused".to_string();
                }
            }
            println!("{}", report.to_json());
            ExitCode::SUCCESS
        }
        Err(why) => {
            eprintln!("pack-host: {}", why.replace('\n', " "));
            ExitCode::from(1)
        }
    }
}

fn host(faults: &Faults) -> Result<Report, String> {
    let args: Vec<String> = env::args().skip(1).collect();
    let args = parse(&args)?;
    let bank = env::var_os(BANK).ok_or_else(|| format!("{BANK} names no folder"))?;
    // One folder of the bank per inbox: its raw material and its cursors.
    let state = PathBuf::from(bank).join(content_hash(args.inbox.to_string_lossy().as_bytes()));
    let machine = short_hostname()?;
    let branch = Branch::local(&args.branch, &args.inbox);
    let host = Host {
        branch: &branch,
        state: &state,
        machine: &machine,
        limit: args.limit,
        faults,
    };
    match args.mode {
        Mode::Intake(raw_dir) => {
            let lines = stdin_lines()?;
            Ok(host
                .take_lines(&lines, raw_dir.as_deref(), CameIn::Intake)?
                .finish(true))
        }
        Mode::Run(declaration) => host.run(&declaration),
        Mode::Import => host.import(&stdin_lines()?),
    }
}

fn stdin_lines() -> Result<Vec<String>, String> {
    io::stdin()
        .lock()
        .lines()
        .collect::<Result<_, _>>()
        .map_err(|error| format!("standard input: {error}"))
}

/// This machine's short hostname (17).
fn short_hostname() -> Result<String, String> {
    let output = Command::new("hostname")
        .stdin(Stdio::null())
        .output()
        .map_err(|error| format!("hostname could not be run: {error}"))?;
    let name = String::from_utf8_lossy(&output.stdout);
    let short = name.trim().split('.').next().unwrap_or_default();
    if short.is_empty() {
        return Err("hostname named no machine".into());
    }
    Ok(short.to_string())
}

fn ask(what: &str) -> Ask<'_> {
    Ask {
        what,
        who: WHO,
        bound: None,
        stale_after: Duration::from_secs(60),
        clock: &OffsetDateTime::now_utc,
    }
}

/// The inbox at `tip`, read whole (43).
fn inbox_at(tip: Option<&Tip>) -> Result<InboxAt, String> {
    let files = match tip {
        Some(tip) => tip.files("inbox/").map_err(|error| error.to_string())?,
        None => Files::new(),
    };
    InboxAt::read(&files).map_err(|error| error.to_string())
}

struct Host<'a> {
    branch: &'a Branch,
    state: &'a Path,
    machine: &'a str,
    limit: usize,
    faults: &'a Faults,
}

/// What a capture's raw files were, by its key, as the bank was handed
/// them.
type Banked = Rc<RefCell<BTreeMap<String, Vec<RawFile>>>>;

/// The inbox take writes through: each capture one write of the branch,
/// formed and checked at its tip (19, 44 to 46).
struct BranchInbox<'a> {
    branch: &'a Branch,
    held: BTreeSet<Key>,
    banked: Banked,
    faults: &'a Faults,
}

impl Inbox for BranchInbox<'_> {
    fn holds(&self, key: &Key) -> bool {
        self.held.contains(key)
    }

    fn write(&mut self, taken: &Taken) -> Result<(), Refusal> {
        let key = taken.capture.key.to_string();
        let raw = self.banked.borrow().get(&key).cloned().unwrap_or_default();
        let what = format!("take {key}");
        weir_branch::write(self.branch, &ask(&what), |tip| {
            let mut files = inbox_at(tip)?
                .take(taken)
                .map_err(|refused| refused.to_string())?
                .files;
            self.faults.on_capture(&key, &raw, &mut files);
            Ok(files)
        })
        .map(|_| ())
        .map_err(|error| Refusal(error.to_string()))
    }
}

/// The bank: each capture's raw files and the whole text of a cut excerpt
/// under `<root>/<key>/`, byte for byte (17, 18, 21).
struct FolderBank {
    root: PathBuf,
    banked: Banked,
}

impl Bank for FolderBank {
    fn bank(
        &mut self,
        key: &Key,
        files: &[RawFile],
        whole_excerpt: Option<&str>,
    ) -> Result<String, Refusal> {
        let dir = self.root.join(key.as_str());
        let fail = |error: io::Error| Refusal(format!("{}: {error}", dir.display()));
        for file in files {
            let path = dir.join("raw").join(&file.path);
            if let Some(parent) = path.parent() {
                fs::create_dir_all(parent).map_err(fail)?;
            }
            fs::write(&path, &file.bytes).map_err(fail)?;
        }
        if let Some(text) = whole_excerpt {
            fs::create_dir_all(&dir).map_err(fail)?;
            fs::write(dir.join("excerpt.txt"), text).map_err(fail)?;
        }
        self.banked
            .borrow_mut()
            .insert(key.to_string(), files.to_vec());
        Ok(dir.to_string_lossy().into_owned())
    }

    fn discard(&mut self, key: &Key) {
        let _ = fs::remove_dir_all(self.root.join(key.as_str()));
        self.banked.borrow_mut().remove(key.as_str());
    }
}

impl Host<'_> {
    /// Takes `lines` through the branch's inbox and the bank, the keys the
    /// tip holds before the take held (13 to 22).
    fn take_lines(
        &self,
        lines: &[String],
        raw_dir: Option<&Path>,
        came_in: CameIn,
    ) -> Result<Report, String> {
        let tip = weir_branch::read(self.branch).map_err(|failure| failure.to_string())?;
        let held = inbox_at(tip.as_ref())?.keys().cloned().collect();
        let banked = Banked::default();
        let mut inbox = BranchInbox {
            branch: self.branch,
            held,
            banked: banked.clone(),
            faults: self.faults,
        };
        let mut bank = FolderBank {
            root: self.state.join("bank"),
            banked,
        };
        let how = Take::new(self.machine, OffsetDateTime::now_utc(), self.limit, came_in)
            .map_err(|error| error.to_string())?;
        Ok(take(lines, raw_dir, &how, &mut inbox, &mut bank))
    }

    /// A run of the adapter the declaration names, from its declaration
    /// and nothing else (11), with a fresh raw place and the cursor of its
    /// last complete run (8); its lines taken, and its cursor kept only
    /// when it was complete (9, 23).
    fn run(&self, declared: &Path) -> Result<Report, String> {
        let text = fs::read_to_string(declared)
            .map_err(|error| format!("{}: {error}", declared.display()))?;
        let declarations = read_declarations(&text).map_err(|error| error.to_string())?;
        let [declaration] = declarations.as_slice() else {
            return Err(format!(
                "{} declares {} adapters, and a run names one",
                declared.display(),
                declarations.len()
            ));
        };
        let beside = declared.parent().unwrap_or(Path::new("."));
        let schema = fs::read_to_string(beside.join(&declaration.config))
            .map_err(|error| format!("{}: {error}", declaration.config))?;
        let schema: serde_json::Value =
            serde_json::from_str(&schema).map_err(|error| error.to_string())?;
        let config = serde_json::json!({});
        check_config(&schema, &config).map_err(|error| error.to_string())?;
        fs::create_dir_all(self.state).map_err(|error| error.to_string())?;
        let config_file = self.state.join("config.json");
        fs::write(&config_file, config.to_string()).map_err(|error| error.to_string())?;
        let cursor_file = self.state.join("cursors").join(&declaration.name);
        let cursor = fs::read_to_string(&cursor_file).ok();
        let raw_dir = tempfile::tempdir().map_err(|error| error.to_string())?;
        let invocation = declaration.invocation(raw_dir.path(), cursor.as_deref());
        let output = Command::new(&invocation.args[0])
            .args(&invocation.args[1..])
            .env(CAPTURE_CONFIG, &config_file)
            .stdin(Stdio::null())
            .stderr(Stdio::inherit())
            .output()
            .map_err(|error| format!("the adapter could not be run: {error}"))?;
        let lines: Vec<String> = String::from_utf8_lossy(&output.stdout)
            .lines()
            .map(str::to_string)
            .collect();
        let mut report = self
            .take_lines(
                &lines,
                Some(raw_dir.path()),
                CameIn::Run(declaration.name.clone()),
            )?
            .finish(output.status.success());
        if self.faults.cursor_from_incomplete && !report.complete {
            report.cursor = report.last_cursor.clone();
        }
        if let Some(kept) = &report.cursor {
            fs::create_dir_all(self.state.join("cursors")).map_err(|error| error.to_string())?;
            fs::write(&cursor_file, kept).map_err(|error| error.to_string())?;
        }
        Ok(report)
    }

    /// The import of `lines` as one write, formed and checked at the tip
    /// (37 to 39, 75a): a line that is no import line refused by its
    /// number, and each capture the import refuses by its line.
    fn import(&self, lines: &[String]) -> Result<Report, String> {
        let mut refused = Vec::new();
        let mut captures: Vec<ImportedCapture> = Vec::new();
        let mut line_of = Vec::new();
        for (n, line) in (1..).zip(lines) {
            match read_import_line(line) {
                Ok(read) => {
                    captures.push(read.into());
                    line_of.push(n);
                }
                Err(malformed) => refused.push(Refused {
                    line: n,
                    key: malformed.key.clone(),
                    reason: malformed.to_string(),
                }),
            }
        }
        let now = OffsetDateTime::now_utc();
        let mut imported = None;
        weir_branch::write(self.branch, &ask("import"), |tip| {
            let done = inbox_at(tip)?
                .import(&captures, self.machine, now)
                .map_err(|refused| refused.to_string())?;
            imported = Some(done.report);
            Ok(done.write.map(|write| write.files).unwrap_or_default())
        })
        .map_err(|error| error.to_string())?;
        let report = imported.ok_or("the import formed no write")?;
        for refusal in report.refused {
            let line = captures
                .iter()
                .position(|capture| capture.key == refusal.key)
                .map_or(0, |at| line_of[at]);
            refused.push(Refused {
                line,
                key: Some(refusal.key),
                reason: format!("`{}` {}", refusal.field, refusal.why),
            });
        }
        refused.sort_by_key(|refusal| refusal.line);
        Ok(Report {
            taken: report.written,
            held: report.held,
            refused,
            ..Report::default()
        }
        .finish(true))
    }
}
