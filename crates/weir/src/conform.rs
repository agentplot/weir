//! `weir conform`: a host run against every host case of the pack through
//! the one seam the pack fixes (65, 87; C.7). For each case, a bare git
//! repository with no remote in a temporary folder, its branch `inbox`
//! seeded with the case's starting records; the host run once per step
//! with the case's flags and input, and the `weir-report/1` it prints last
//! compared with the step's; then the records at the branch's tip
//! compared with the case's, through `weir-pack`. Nothing is fetched or
//! pushed anywhere else, and the temporary folder goes with the case (68).

use std::collections::BTreeMap;
use std::fs::{self, File};
use std::io;
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};
use std::time::Duration;

use time::OffsetDateTime;
use weir_branch::{write, Ask, Branch, Files};
use weir_pack::{
    compare_report, compare_tip, read_host_case, verdict, Case, HostCase, Invocation, Machine,
    Pack, Shape,
};

/// The branch every case seeds and every host writes (C.7).
const BRANCH: &str = "inbox";

/// Who seeds a case's branch, as its commit names (50).
const SEEDER: &str = "weir conform";

/// The variables that would point git at another repository than the one
/// named.
const REDIRECTS: [&str; 7] = [
    "GIT_DIR",
    "GIT_WORK_TREE",
    "GIT_INDEX_FILE",
    "GIT_OBJECT_DIRECTORY",
    "GIT_ALTERNATE_OBJECT_DIRECTORIES",
    "GIT_COMMON_DIR",
    "GIT_NAMESPACE",
];

/// Every host case of `pack` run against `program` with `args`, a line
/// printed for each; whether every case passed, or why the cases could not
/// be run at all.
pub fn conform(pack: &Pack, program: &str, args: &[&str]) -> Result<bool, String> {
    let hostname = short_hostname()?;
    let mut passed = true;
    for case in pack.cases(Shape::Host) {
        let line = match read_host_case(&case) {
            Ok(host_case) => {
                let differences = run_case(&host_case, program, args, &hostname);
                passed &= differences.is_empty();
                verdict(&host_case, &differences)
            }
            Err(why) => {
                passed = false;
                format!("{}: failed: the case does not read: {why}", case.name)
            }
        };
        println!("{line}");
    }
    Ok(passed)
}

/// This machine's short hostname, the name a pointer gives it (17).
fn short_hostname() -> Result<String, String> {
    let output = Command::new("hostname")
        .stdin(Stdio::null())
        .output()
        .map_err(|error| format!("hostname could not be run: {error}"))?;
    let name = String::from_utf8_lossy(&output.stdout);
    let short = name
        .trim()
        .split('.')
        .next()
        .unwrap_or_default()
        .to_string();
    if !output.status.success() || short.is_empty() {
        return Err("hostname named no machine".to_string());
    }
    Ok(short)
}

/// git in the repository `repo`, with nothing redirecting it, `input` on
/// its standard input; what it printed, or its words when it failed.
fn git(repo: &Path, args: &[&str], input: Option<&Path>) -> Result<String, String> {
    let mut command = Command::new("git");
    command.arg("--git-dir").arg(repo).args(args);
    for name in REDIRECTS {
        command.env_remove(name);
    }
    command
        .env("GIT_AUTHOR_NAME", SEEDER)
        .env("GIT_AUTHOR_EMAIL", "")
        .env("GIT_COMMITTER_NAME", SEEDER)
        .env("GIT_COMMITTER_EMAIL", "");
    command.stdin(match input {
        Some(path) => Stdio::from(File::open(path).map_err(|error| error.to_string())?),
        None => Stdio::null(),
    });
    let output = command
        .output()
        .map_err(|error| format!("git could not be run: {error}"))?;
    if output.status.success() {
        Ok(String::from_utf8_lossy(&output.stdout).trim().to_string())
    } else {
        Err(String::from_utf8_lossy(&output.stderr).trim().to_string())
    }
}

/// Seeds the branch with the case's starting records in one write of
/// `weir-branch`, or with an empty tree when it has none, the branch made
/// from the expected old value of none (core.27).
fn seed(
    branch: &Branch,
    repo: &Path,
    scratch: &Path,
    start: &BTreeMap<String, Vec<u8>>,
) -> Result<(), String> {
    if start.is_empty() {
        let empty = scratch.join("empty");
        fs::write(&empty, "").map_err(|error| error.to_string())?;
        let tree = git(repo, &["mktree"], Some(&empty))?;
        let commit = git(
            repo,
            &[
                "commit-tree",
                "--no-gpg-sign",
                &tree,
                "-m",
                "seed an empty inbox",
            ],
            None,
        )?;
        git(
            repo,
            &["update-ref", &format!("refs/heads/{BRANCH}"), &commit, ""],
            None,
        )?;
        return Ok(());
    }
    let files: Files = start
        .iter()
        .map(|(path, bytes)| {
            String::from_utf8(bytes.clone())
                .map(|text| (path.clone(), text))
                .map_err(|_| format!("start/{path} is not UTF-8"))
        })
        .collect::<Result<_, _>>()?;
    let ask = Ask {
        what: "seed the case's starting records",
        who: SEEDER,
        bound: None,
        stale_after: Duration::from_secs(60),
        clock: &OffsetDateTime::now_utc,
    };
    write(branch, &ask, |_| Ok(files.clone()))
        .map(|_| ())
        .map_err(|error| error.to_string())
}

/// A TOML basic string holding `text`.
fn toml_string(text: &str) -> String {
    format!("\"{}\"", text.replace('\\', "\\\\").replace('"', "\\\""))
}

/// The declaration a run step names: the case's adapter by the case's
/// name, its command the case's script, its configuration's schema one
/// that takes any configuration, and no secret (11; C.1, C.7).
fn declaration(scratch: &Path, name: &str, script: &Path) -> io::Result<PathBuf> {
    fs::write(scratch.join("config.schema.json"), "{}\n")?;
    let path = scratch.join("adapter.toml");
    let script = script.to_string_lossy();
    fs::write(
        &path,
        format!(
            "[adapter.{}]\ncommand = [{}]\nconfig = \"config.schema.json\"\nsecrets = []\n",
            toml_string(name),
            toml_string(&script)
        ),
    )?;
    Ok(path)
}

/// Every file under `path` on this machine, each with its bytes: the
/// pointer's place, as the comparison reads it.
fn read_place(path: &str) -> Result<Vec<Vec<u8>>, String> {
    fn walk(path: &Path, found: &mut Vec<Vec<u8>>) -> io::Result<()> {
        if path.is_dir() {
            for entry in fs::read_dir(path)? {
                walk(&entry?.path(), found)?;
            }
        } else {
            found.push(fs::read(path)?);
        }
        Ok(())
    }
    let mut found = Vec::new();
    walk(Path::new(path), &mut found).map_err(|error| error.to_string())?;
    Ok(found)
}

/// Runs one case, each difference from what it expects named; none when
/// the host passed it.
fn run_case(case: &HostCase, program: &str, args: &[&str], hostname: &str) -> Vec<String> {
    let scratch = match tempfile::tempdir() {
        Ok(scratch) => scratch,
        Err(error) => return vec![format!("no temporary folder for the case: {error}")],
    };
    let repo = scratch.path().join("inbox.git");
    let case_dir = scratch.path().join("case");
    let repo_text = repo.to_string_lossy().into_owned();
    if let Err(why) = git(
        scratch.path(),
        &["init", "--bare", "--quiet", &repo_text],
        None,
    ) {
        return vec![format!("the bare repository could not be made: {why}")];
    }
    let branch = Branch::local(BRANCH, &repo);
    if let Err(why) = seed(&branch, &repo, scratch.path(), &case.start) {
        return vec![format!("the branch could not be seeded: {why}")];
    }
    let materialized = Case {
        shape: Shape::Host,
        name: case.name.clone(),
        files: case.files.clone(),
    };
    if let Err(error) = materialized.write_to(&case_dir) {
        return vec![format!("the case's files could not be written: {error}")];
    }
    let mut said = Vec::new();
    let began = OffsetDateTime::now_utc();
    for (n, step) in (1..).zip(&case.steps) {
        match run_step(
            case,
            &step.invocation,
            program,
            args,
            &repo,
            &case_dir,
            scratch.path(),
        ) {
            Err(why) => {
                said.push(format!("step {n}: {why}"));
                break;
            }
            Ok(printed) => said.extend(
                compare_report(&step.expected, &printed)
                    .into_iter()
                    .map(|difference| format!("step {n}: {difference}")),
            ),
        }
    }
    let ended = OffsetDateTime::now_utc();
    let tip = match weir_branch::read(&branch) {
        Ok(None) => BTreeMap::new(),
        Ok(Some(tip)) => match tip.files("") {
            Ok(files) => files
                .into_iter()
                .map(|(path, text)| (path, text.into_bytes()))
                .collect(),
            Err(error) => {
                said.push(format!("the tip could not be read: {error}"));
                return said;
            }
        },
        Err(failure) => {
            said.push(format!("the tip could not be read: {failure}"));
            return said;
        }
    };
    said.extend(compare_tip(
        case,
        &tip,
        &Machine {
            hostname,
            began,
            ended,
            read: &read_place,
        },
    ));
    said
}

/// Runs the host for one step, with no shell: its words, then
/// `--inbox <repo> --branch inbox --limit <bytes>`, then the step's own
/// flags, and its input on standard input (C.7). The last line it printed
/// on standard output, or why there is none.
fn run_step(
    case: &HostCase,
    invocation: &Invocation,
    program: &str,
    args: &[&str],
    repo: &Path,
    case_dir: &Path,
    scratch: &Path,
) -> Result<String, String> {
    let mut command = Command::new(program);
    command.args(args).arg("--inbox").arg(repo).args([
        "--branch",
        BRANCH,
        "--limit",
        &case.limit.to_string(),
    ]);
    let input = match invocation {
        Invocation::Intake { lines, raw } => {
            if let Some(raw) = raw {
                command.arg("--raw-dir").arg(case_dir.join(raw));
            }
            Some(case_dir.join(lines))
        }
        Invocation::Run { adapter, .. } => {
            let declared = declaration(scratch, &case.name, &case_dir.join(adapter))
                .map_err(|error| format!("the declaration could not be written: {error}"))?;
            command.arg("--run").arg(declared);
            None
        }
        Invocation::Import { lines } => {
            command.arg("--import");
            Some(case_dir.join(lines))
        }
    };
    command.stdin(match input {
        Some(path) => Stdio::from(File::open(&path).map_err(|error| error.to_string())?),
        None => Stdio::null(),
    });
    let output = command
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .output()
        .map_err(|error| format!("the host could not be run: {error}"))?;
    let stdout = String::from_utf8_lossy(&output.stdout);
    if let Some(last) = stdout.lines().rev().find(|line| !line.trim().is_empty()) {
        return Ok(last.to_string());
    }
    let stderr = String::from_utf8_lossy(&output.stderr);
    let said = stderr
        .lines()
        .rev()
        .find(|line| !line.trim().is_empty())
        .map_or_else(
            || "saying nothing".to_string(),
            |line| format!("saying `{}`", line.trim()),
        );
    let exit = match output.status.code() {
        Some(code) => format!("exited {code}"),
        None => "was ended by a signal".to_string(),
    };
    Err(format!("the host {exit} and printed no report, {said}"))
}
