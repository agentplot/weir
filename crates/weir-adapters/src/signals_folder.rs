//! The signals-folder import: crew's folders read into import lines, one
//! per capture, each with the pointer, signals and moves it was read with
//! (37 to 39, 63; C.6; S8, S9). It is no adapter, and writes nothing: a
//! host's import operation writes its lines (64, 75a). This is the one
//! module of the crate that names crew (core.2).
//!
//! A body is a root holding `signals/`: a partition's flywheel branch, or a
//! blueprints repository's meeting corpus. Each folder under `signals/`
//! holding a `capture.md` is one capture, read in the order of the folders'
//! names; anything else there is passed over. Its moves are those of one
//! `moves.rec`: the file the caller names, the flywheel branch's for a
//! blueprints corpus, or else the one at the root, where a flywheel branch
//! keeps them; a `signals/moves.rec` is never read, crew having carried its
//! moves onto the flywheel branch (C.6). The reading is a function of
//! the body's files alone, sorted at every step, so one body read twice
//! gives the same lines (38; S8; core.14), and no file a pointer names is
//! ever opened (37; core.11, core.25).

use std::collections::BTreeMap;
use std::fmt;
use std::fs;
use std::io;
use std::path::Path;

use time::format_description::well_known::Rfc3339;
use time::OffsetDateTime;
use weir_capture::{
    rec, CameIn, ImportLine, ImportLineCapture, ImportLineMove, ImportLineSignal, Key, Nature,
    Record,
};

use crate::event_time::day_start;

/// The body crew's folders are, as each line's `came_in` names it (22).
pub const BODY: &str = "crew-folders";

/// What `weir import signals <root>` does, for the command's help.
pub const HELP: &str = "print one import line per capture of crew's signals folders under \
                        <root>/signals/, with its pointer, signals and moves, for a host's \
                        import operation to write";

/// What a body's reading gives: one line per capture read whole, each
/// capture not read whole with why, and each move not carried with why.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Reading {
    pub lines: Vec<ImportLine>,
    pub not_read: Vec<NotRead>,
    pub not_carried: Vec<NotCarried>,
}

/// A capture folder not read whole: the folder, the file and why (15, 28).
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct NotRead {
    pub capture: String,
    pub file: String,
    pub why: String,
}

impl fmt::Display for NotRead {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "capture signals/{}/ not read: {}: {}",
            self.capture, self.file, self.why
        )
    }
}

/// A move of `moves.rec` carried in no line: the signal it names as
/// written, its word and why (15).
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct NotCarried {
    pub signal: String,
    pub word: String,
    pub why: String,
}

impl fmt::Display for NotCarried {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "move `{}` of signal {} not carried: {}",
            self.word, self.signal, self.why
        )
    }
}

/// The body at `root` read into import lines, its moves those of the file
/// `moves` names or else of `<root>/moves.rec`; or why it cannot be read: a
/// root with no `signals/` folder, a moves file named that cannot be read,
/// or a `moves.rec` that cannot be read as records.
pub fn read(root: &Path, moves: Option<&Path>) -> Result<Reading, String> {
    let signals = root.join("signals");
    if !signals.is_dir() {
        return Err(format!(
            "{} holds no signals/ folder of crew's",
            root.display()
        ));
    }
    let records = match moves {
        Some(file) => moves_named(file)?,
        None => moves_at_root(root)?,
    };
    let mut reading = Reading::default();
    // Each capture folder, with its line's place and its signal files when
    // it was read whole.
    let mut captures: BTreeMap<String, Option<(usize, Vec<String>)>> = BTreeMap::new();
    for capture in capture_folders(&signals)? {
        match read_capture(&signals.join(&capture), &capture) {
            Ok((line, files)) => {
                captures.insert(capture, Some((reading.lines.len(), files)));
                reading.lines.push(line);
            }
            Err((file, why)) => {
                captures.insert(capture.clone(), None);
                reading.not_read.push(NotRead { capture, file, why });
            }
        }
    }
    for record in records {
        let Some(signal) = record.get("Signal") else {
            continue;
        };
        let word = record.get("Move").unwrap_or_default();
        let not_carried = |why: String| NotCarried {
            signal: signal.to_string(),
            word: word.to_string(),
            why,
        };
        if word.is_empty() {
            reading
                .not_carried
                .push(not_carried("it names no move".to_string()));
            continue;
        }
        match signal_number(signal, &captures) {
            Ok((line, n)) => reading.lines[line].moves.push(carried(&record, word, n)),
            Err(why) => reading.not_carried.push(not_carried(why)),
        }
    }
    Ok(reading)
}

/// The records of the moves file the caller named, which must be there.
fn moves_named(file: &Path) -> Result<Vec<Record>, String> {
    let text = fs::read_to_string(file)
        .map_err(|error| format!("{} cannot be read: {error}", file.display()))?;
    rec::read(&text).map_err(|error| format!("{} cannot be read: {error}", file.display()))
}

/// The records of `<root>/moves.rec`; none when there is no such file.
fn moves_at_root(root: &Path) -> Result<Vec<Record>, String> {
    match fs::read_to_string(root.join("moves.rec")) {
        Ok(text) => rec::read(&text).map_err(|error| format!("moves.rec cannot be read: {error}")),
        Err(error) if error.kind() == io::ErrorKind::NotFound => Ok(Vec::new()),
        Err(error) => Err(format!("moves.rec cannot be read: {error}")),
    }
}

/// The place of the line of the capture a move names its signal in, and
/// the signal's number there, by its capture's folder name and the `NN` its
/// signal part, `<NN>-<slug>` or `<NN>`, begins with: the one signal file
/// of that capture whose name begins `NN`, whatever slug follows (C.6); or
/// why the body holds no one such signal it read.
fn signal_number(
    id: &str,
    captures: &BTreeMap<String, Option<(usize, Vec<String>)>>,
) -> Result<(usize, u64), String> {
    let Some((capture, signal)) = id.rsplit_once('/') else {
        return Err("it is no signal's id, <capture>/<NN>-<slug>".to_string());
    };
    let bytes = signal.as_bytes();
    let numbered = bytes.len() >= 2
        && bytes[..2].iter().all(u8::is_ascii_digit)
        && (bytes.len() == 2 || bytes[2] == b'-');
    if !numbered {
        return Err(format!(
            "`{signal}` names no signal's number, <NN>-<slug> or <NN>"
        ));
    }
    let number = &signal[..2];
    let (line, files) = match captures.get(capture) {
        None => return Err(format!("no capture of this body is `{capture}`")),
        Some(None) => return Err(format!("its capture `{capture}` was not read")),
        Some(Some((line, files))) => (*line, files),
    };
    let matched: Vec<(usize, &String)> = files
        .iter()
        .enumerate()
        .filter(|(_, file)| file.starts_with(number))
        .collect();
    match matched.as_slice() {
        [(at, _)] => Ok((line, *at as u64 + 1)),
        [] => Err(format!(
            "its capture `{capture}` holds no signal `{number}`"
        )),
        _ => Err(format!(
            "its capture `{capture}` holds more than one signal numbered `{number}`: {}",
            matched
                .iter()
                .map(|(_, file)| file.as_str())
                .collect::<Vec<_>>()
                .join(", ")
        )),
    }
}

/// A move as its line carries it: crew's words as the six, `drop` with its
/// `Reason` as its consequence, `new-territory` as a join into a proposed
/// item named for its target, and any other word as it stands, each with
/// its `Reason` as its note (30, 31, 39; C.2; section 10, question 3;
/// core.25).
fn carried(record: &Record, word: &str, signal: u64) -> ImportLineMove {
    let field = |name: &str| {
        record
            .get(name)
            .filter(|value| !value.is_empty())
            .map(str::to_string)
    };
    let (word, target) = match word {
        "drop" => (
            "drop".to_string(),
            field("Reason").or_else(|| field("Target")),
        ),
        "new-territory" => (
            "join".to_string(),
            field("Target").and_then(|target| item_name(&target)),
        ),
        word => (word.to_string(), field("Target")),
    };
    ImportLineMove {
        signal,
        word,
        target,
        note: field("Reason"),
        by: field("By"),
        at: field("Date").map(|date| day_start(&date).unwrap_or(date)),
    }
}

/// The name of a proposed item made from `target`: lowercase ASCII letters
/// and digits kept, every other run of characters one `-`, trimmed of `-`,
/// so it holds no `/` and lies as one folder (35); none when nothing is
/// left.
fn item_name(target: &str) -> Option<String> {
    let mut name = String::new();
    for c in target.chars() {
        if c.is_ascii_alphanumeric() {
            name.push(c.to_ascii_lowercase());
        } else if !name.ends_with('-') {
            name.push('-');
        }
    }
    let name = name.trim_matches('-');
    (!name.is_empty()).then(|| name.to_string())
}

/// The names of the folders under `signals/` holding a `capture.md`, in
/// byte order.
fn capture_folders(signals: &Path) -> Result<Vec<String>, String> {
    let mut names: Vec<String> = fs::read_dir(signals)
        .map_err(|error| format!("{} cannot be read: {error}", signals.display()))?
        .filter_map(|entry| entry.ok())
        .map(|entry| entry.file_name().to_string_lossy().into_owned())
        .filter(|name| signals.join(name).join("capture.md").is_file())
        .collect();
    names.sort();
    Ok(names)
}

/// A capture's line, with its moves still to be carried, and its signal
/// files in their order; or the file it could not be read whole from and
/// why.
fn read_capture(folder: &Path, name: &str) -> Result<(ImportLine, Vec<String>), (String, String)> {
    let not_read = |why: String| ("capture.md".to_string(), why);
    let text = fs::read_to_string(folder.join("capture.md"))
        .map_err(|error| not_read(format!("cannot be read: {error}")))?;
    let (fields, body) = front_matter(&text);
    let field = |name: &str| {
        fields
            .iter()
            .find(|(field, _)| field == name)
            .map(|(_, value)| value.as_str())
            .filter(|value| !value.is_empty())
    };
    let source = field("source")
        .ok_or_else(|| not_read("names no `source`".to_string()))?
        .to_string();
    // A release capture keeps the key it was first held under; every other
    // takes its folder's name (38; C.6; core.25).
    let key_text = match (source.as_str(), field("key")) {
        ("release", Some(key)) => key,
        _ => name,
    };
    let key = Key::parse(key_text)
        .map_err(|error| not_read(format!("the key `{key_text}` is refused: {error}")))?;
    let event_at = field("at")
        .filter(|at| OffsetDateTime::parse(at, &Rfc3339).is_ok())
        .map(str::to_string)
        .or_else(|| field("event_date").and_then(day_start))
        .ok_or_else(|| {
            not_read("names neither `at` as an RFC 3339 time nor `event_date` as a day".to_string())
        })?;
    let title = body
        .lines()
        .find_map(|line| line.strip_prefix("# "))
        .map(|title| title.trim_end().to_string());
    let files = signal_files(folder).map_err(|why| (".".to_string(), why))?;
    let signals = files
        .iter()
        .zip(1..)
        .map(|(file, n)| read_signal(folder, name, file, n))
        .collect::<Result<_, _>>()?;
    let line = ImportLine {
        capture: ImportLineCapture {
            key,
            source,
            event_at,
            title,
            url: field("url").map(str::to_string),
        },
        pointer: field("raw").map(str::to_string),
        came_in: CameIn::Import(BODY.to_string()),
        signals,
        moves: Vec::new(),
    };
    Ok((line, files))
}

/// Whether `name` is a signal's file as crew names one, `NN-<slug>.md`, the
/// slug lowercase letters, digits and dashes.
fn is_signal_file(name: &str) -> bool {
    let Some(stem) = name.strip_suffix(".md") else {
        return false;
    };
    let bytes = stem.as_bytes();
    bytes.len() > 3
        && bytes[..2].iter().all(u8::is_ascii_digit)
        && bytes[2] == b'-'
        && bytes[3] != b'-'
        && bytes[3..]
            .iter()
            .all(|byte| byte.is_ascii_lowercase() || byte.is_ascii_digit() || *byte == b'-')
}

/// A capture's signal files, in the order of their `NN` and then of their
/// names, which byte order of the names is, `NN` being two digits (27).
fn signal_files(folder: &Path) -> Result<Vec<String>, String> {
    let mut names: Vec<String> = fs::read_dir(folder)
        .map_err(|error| format!("the capture's folder cannot be read: {error}"))?
        .filter_map(|entry| entry.ok())
        .map(|entry| entry.file_name().to_string_lossy().into_owned())
        .filter(|name| is_signal_file(name))
        .collect();
    names.sort();
    Ok(names)
}

/// A list as crew writes one, `[a, b]`, read as its entries; an absent one
/// is empty.
fn list(value: Option<&str>) -> Vec<String> {
    let Some(value) = value.map(str::trim) else {
        return Vec::new();
    };
    let inner = value
        .strip_prefix('[')
        .and_then(|inner| inner.strip_suffix(']'))
        .unwrap_or(value);
    inner
        .split(", ")
        .map(str::trim)
        .filter(|entry| !entry.is_empty())
        .map(str::to_string)
        .collect()
}

/// Whether `line` is a Markdown heading: one or more `#`, then a space or
/// nothing.
fn is_heading(line: &str) -> bool {
    let rest = line.trim_start_matches('#');
    rest.len() < line.len() && (rest.is_empty() || rest.starts_with(' '))
}

/// The signal numbered `n` of the capture `capture`, from its file `file`,
/// or the file and why it could not be read whole (25, 28).
fn read_signal(
    folder: &Path,
    capture: &str,
    file: &str,
    n: u64,
) -> Result<ImportLineSignal, (String, String)> {
    let not_read = |why: String| (file.to_string(), why);
    let text = fs::read_to_string(folder.join(file))
        .map_err(|error| not_read(format!("cannot be read: {error}")))?;
    let (fields, body) = front_matter(&text);
    let field = |name: &str| {
        fields
            .iter()
            .find(|(field, _)| field == name)
            .map(|(_, value)| value.as_str())
            .filter(|value| !value.is_empty())
    };
    let kind_text = field("kind").ok_or_else(|| not_read("names no `kind`".to_string()))?;
    let kind = Nature::parse(kind_text).ok_or_else(|| {
        not_read(format!(
            "`kind` is `{kind_text}`, not one of {}",
            Nature::ALL.map(Nature::as_str).join(", ")
        ))
    })?;
    let asserted_by = field("who")
        .ok_or_else(|| not_read("names no `who`".to_string()))?
        .to_string();

    // The quote's lines, an attribution line after it, and what is left,
    // headings aside, is the assertion.
    let mut quote: Vec<&str> = Vec::new();
    let mut attribution = None;
    let mut assertion: Vec<&str> = Vec::new();
    for line in body.lines() {
        if let Some(quoted) = line.strip_prefix('>') {
            quote.push(quoted.strip_prefix(' ').unwrap_or(quoted));
        } else if let Some(at) = line.strip_prefix("— ") {
            if !quote.is_empty() && attribution.is_none() {
                attribution = Some(at.trim());
            }
        } else if !is_heading(line) && !line.trim().is_empty() {
            assertion.push(line.trim());
        }
    }
    let assertion = assertion.join("\n");
    if assertion.is_empty() {
        return Err(not_read("states nothing it asserts".to_string()));
    }
    let quoted = quote.join("\n");
    // crew's own attribution closes its quote: `"<words>" — <where>`.
    let (excerpt, position) = match quoted.rfind("\" — ") {
        Some(at) if quoted.starts_with('"') && at > 0 && !quoted[at..].contains('\n') => (
            quoted[1..at].to_string(),
            quoted[at + "\" — ".len()..].trim().to_string(),
        ),
        _ => (
            quoted,
            attribution
                .map(str::to_string)
                .unwrap_or_else(|| format!("signals/{capture}/{file}")),
        ),
    };
    if excerpt.is_empty() {
        return Err(not_read("quotes no excerpt".to_string()));
    }
    Ok(ImportLineSignal {
        n,
        kind,
        asserted_by,
        assertion,
        subjects: list(field("subject")),
        argues_with: list(field("claims")),
        excerpt,
        position,
    })
}

/// The fields of the front matter `text` opens with, as crew reads them:
/// the lines between an opening `---` and the next, each split at its
/// first `: `, a line without one a field with no value; and the text after
/// it. A text with no front matter has no fields.
fn front_matter(text: &str) -> (Vec<(String, String)>, &str) {
    let Some(rest) = text.strip_prefix("---\n") else {
        return (Vec::new(), text);
    };
    let (block, body) = if let Some(body) = rest.strip_prefix("---\n") {
        ("", body)
    } else if let Some(at) = rest.find("\n---\n") {
        (&rest[..at], &rest[at + "\n---\n".len()..])
    } else {
        return (Vec::new(), text);
    };
    let fields = block
        .lines()
        .filter(|line| !line.trim().is_empty())
        .map(|line| match line.split_once(": ") {
            Some((name, value)) => (name.to_string(), value.to_string()),
            None => (line.trim_end_matches(':').to_string(), String::new()),
        })
        .collect();
    (fields, body)
}

#[cfg(test)]
mod tests {
    use std::os::unix::fs::PermissionsExt;
    use std::path::PathBuf;

    use super::*;

    const RELEASE: &str = "2026-10-08-release-pendingyou-0-32-0";
    const SESSION: &str = "2026-10-09-swancloud-design-6d65f646";

    const RELEASE_CAPTURE: &str = "\
---
capture: 2026-10-08-release-pendingyou-0-32-0
source: release
key: release/pendingyou/0.32.0
event_date: 2026-10-08
captured_by: release-signals
host: mac-studio
at: 2026-10-08T18:53:39Z
url: https://www.npmjs.com/package/pendingyou/v/0.32.0
raw: mac-studio:~/.local/state/swancloud/release-signals/raw/2026-10-08-release-pendingyou-0.32.0/
imported: 2026-10-08
status: read
signals: 1
---

# pendingyou 0.32.0

pendingyou's release 0.32.0, published 2026-10-08, captured by the fleet's release adapter on mac-studio.
";

    const RELEASE_SIGNAL: &str = "\
---
signal: 2026-10-08-release-pendingyou-0-32-0/01-released
kind: ask
who: npm:pendingyou
subject: [release, pendingyou]
claims: [packages/pendingyou/default.nix, flake.nix inputs.herdr-pendingyou-src]
---

pendingyou 0.32.0 is released; the fleet pins 0.31.2.

> ## 0.32.0
>
> - `pendingyou hold` waits on a card.

— https://www.npmjs.com/package/pendingyou/v/0.32.0, published 2026-10-08T18:53:39Z
";

    const SESSION_CAPTURE: &str = "\
---
capture: 2026-10-09-swancloud-design-6d65f646
source: crew-session
key: session/c433e3c0/6d65f646
captured_by: swancloud-design
host: mac-studio
session: c433e3c0
record: 6d65f646
at: 2026-10-09T22:21:18Z
excerpt: verified
asserted_by: tool
where: main level
raw: mac-studio:~/.local/state/crew/swancloud/raw/2026-10-09-swancloud-design-6d65f646.jsonl
event_date: 2026-10-09
imported: 2026-10-09
status: read
signals: 1
---

# swancloud-design, 2026-10-09 22:21Z

One record of swancloud-design's session, captured with crew signal.
";

    const SESSION_SIGNAL: &str = "\
---
signal: 2026-10-09-swancloud-design-6d65f646/01-no-refetch
kind: constraint
who: swancloud-design
---

Claude Code does not fetch the tools again.

> \"HTTP reconnection successful after 16216ms (attempt 1)\" — 22:21:18Z
";

    const MOVES_DESCRIPTOR: &str = "\
# The flywheel's curation moves: one per signal, appended through crew and
# never merged.

%rec: Move
%doc: A curation move over one signal, stored with its inputs.
%key: Signal
%mandatory: Signal Move Date By
%allowed: Signal Move Target Reason Date By
%type: Move enum attach challenge new-territory answered drop route
%type: Date date
";

    /// A body in a temporary directory, built file by file.
    struct Body {
        dir: tempfile::TempDir,
    }

    impl Body {
        fn new() -> Body {
            let body = Body {
                dir: tempfile::tempdir().unwrap(),
            };
            fs::create_dir(body.root().join("signals")).unwrap();
            body
        }

        fn root(&self) -> PathBuf {
            self.dir.path().to_path_buf()
        }

        fn file(&self, path: &str, text: &str) -> &Body {
            let path = self.root().join(path);
            fs::create_dir_all(path.parent().unwrap()).unwrap();
            fs::write(path, text).unwrap();
            self
        }

        fn capture(&self, name: &str, capture: &str, signals: &[(&str, &str)]) -> &Body {
            self.file(&format!("signals/{name}/capture.md"), capture);
            for (file, text) in signals {
                self.file(&format!("signals/{name}/{file}"), text);
            }
            self
        }

        /// A flywheel branch's tree: the release and the session capture,
        /// crew's README beside them and the moves at the root.
        fn flywheel() -> Body {
            let body = Body::new();
            body.capture(
                RELEASE,
                RELEASE_CAPTURE,
                &[("01-released.md", RELEASE_SIGNAL)],
            )
            .capture(
                SESSION,
                SESSION_CAPTURE,
                &[("01-no-refetch.md", SESSION_SIGNAL)],
            )
            .file("signals/README.md", "# Signals\n")
            .file("moves.rec", MOVES_DESCRIPTOR);
            body
        }

        fn read(&self) -> Reading {
            read(&self.root(), None).unwrap()
        }

        fn read_with(&self, moves: &Path) -> Reading {
            read(&self.root(), Some(moves)).unwrap()
        }
    }

    /// A meeting capture as a blueprints repository's corpus holds one: a
    /// day and no time.
    fn meeting_capture(name: &str) -> String {
        format!(
            "---\ncapture: {name}\nsource: wispr\nevent_key: m-1\nevent_date: 2026-10-02\n\
             captured_by: madswan-design\nimported: 2026-10-02\nstatus: read\nsignals: 1\n---\n\n\
             The weekly planning call.\n"
        )
    }

    fn keys(reading: &Reading) -> Vec<&str> {
        reading
            .lines
            .iter()
            .map(|line| line.capture.key.as_str())
            .collect()
    }

    #[test]
    fn a_flywheel_branchs_tree() {
        let reading = Body::flywheel().read();
        assert_eq!(
            keys(&reading),
            ["release/pendingyou/0.32.0", SESSION],
            "the release capture's first"
        );
        assert!(reading.not_read.is_empty(), "{:?}", reading.not_read);
        for line in &reading.lines {
            assert_eq!(line.came_in, CameIn::Import("crew-folders".into()));
        }
    }

    #[test]
    fn a_blueprints_repositorys_corpus() {
        let body = Body::new();
        body.file("signals/README.md", "# Signals\n")
            .capture(
                "2026-10-02-weekly-planning",
                &meeting_capture("2026-10-02-weekly-planning"),
                &[],
            )
            .file("signals/moves.rec", MOVES_DESCRIPTOR);
        let reading = body.read();
        assert_eq!(keys(&reading), ["2026-10-02-weekly-planning"]);
        let capture = &reading.lines[0].capture;
        assert_eq!(capture.source, "wispr");
        assert_eq!(capture.event_at, "2026-10-02T00:00:00Z");
        assert_eq!(capture.title, None);
        assert_eq!(reading.lines[0].pointer, None);
    }

    #[test]
    fn a_folder_with_no_capture_md_beside_one_that_has_one() {
        let body = Body::flywheel();
        body.file("signals/2026-10-07-half-written/01-a.md", SESSION_SIGNAL)
            .file("signals/loose.md", "not a capture\n");
        let reading = body.read();
        assert_eq!(keys(&reading).len(), 2);
        assert!(reading.not_read.is_empty(), "{:?}", reading.not_read);
    }

    #[test]
    fn a_root_with_no_signals_folder_is_refused() {
        let dir = tempfile::tempdir().unwrap();
        let why = read(dir.path(), None).unwrap_err();
        assert!(why.contains("signals/"), "{why}");
    }

    #[test]
    fn one_capture_of_three_not_read() {
        let body = Body::flywheel();
        let no_source = RELEASE_CAPTURE
            .replace("source: release\n", "")
            .replace(RELEASE, "2026-10-08-x");
        body.capture("2026-10-08-x", &no_source, &[]);
        let reading = body.read();
        assert_eq!(keys(&reading), ["release/pendingyou/0.32.0", SESSION]);
        assert_eq!(
            reading.not_read,
            [NotRead {
                capture: "2026-10-08-x".into(),
                file: "capture.md".into(),
                why: "names no `source`".into(),
            }]
        );
        assert_eq!(
            reading.not_read[0].to_string(),
            "capture signals/2026-10-08-x/ not read: capture.md: names no `source`"
        );
    }

    #[test]
    fn a_release_capture() {
        let reading = Body::flywheel().read();
        let release = &reading.lines[0].capture;
        assert_eq!(release.key.as_str(), "release/pendingyou/0.32.0");
        assert_eq!(release.source, "release");
    }

    #[test]
    fn a_sessions_capture() {
        let reading = Body::flywheel().read();
        let session = &reading.lines[1].capture;
        assert_eq!(session.key.as_str(), SESSION);
        assert_eq!(session.source, "crew-session");
    }

    #[test]
    fn a_key_the_rule_refuses_leaves_the_capture_unread() {
        let body = Body::new();
        body.capture(
            RELEASE,
            &RELEASE_CAPTURE.replace("key: release/pendingyou/0.32.0", "key: moves/x"),
            &[],
        )
        .capture("capture.rec", &meeting_capture("capture.rec"), &[]);
        let reading = body.read();
        assert!(reading.lines.is_empty());
        assert_eq!(reading.not_read.len(), 2);
        assert_eq!(reading.not_read[0].capture, RELEASE);
        assert!(
            reading.not_read[0].why.contains("`moves`"),
            "{:?}",
            reading.not_read
        );
        assert!(
            reading.not_read[1].why.contains("`capture.rec`"),
            "{:?}",
            reading.not_read
        );
    }

    #[test]
    fn a_release_captures_fields() {
        let reading = Body::flywheel().read();
        let release = &reading.lines[0].capture;
        assert_eq!(release.event_at, "2026-10-08T18:53:39Z");
        assert_eq!(
            release.url.as_deref(),
            Some("https://www.npmjs.com/package/pendingyou/v/0.32.0")
        );
        assert_eq!(release.title.as_deref(), Some("pendingyou 0.32.0"));
        let session = &reading.lines[1].capture;
        assert_eq!(
            session.title.as_deref(),
            Some("swancloud-design, 2026-10-09 22:21Z")
        );
        assert_eq!(session.url, None);
    }

    #[test]
    fn a_day_and_no_time() {
        let body = Body::new();
        body.capture(
            SESSION,
            &SESSION_CAPTURE
                .replace("at: 2026-10-09T22:21:18Z\n", "")
                .replace("event_date: 2026-10-09", "event_date: 2026-10-02"),
            &[],
        );
        assert_eq!(
            body.read().lines[0].capture.event_at,
            "2026-10-02T00:00:00Z"
        );
        let not_a_time = Body::new();
        not_a_time.capture(
            SESSION,
            &SESSION_CAPTURE.replace("at: 2026-10-09T22:21:18Z", "at: late"),
            &[],
        );
        assert_eq!(
            not_a_time.read().lines[0].capture.event_at,
            "2026-10-09T00:00:00Z"
        );
    }

    #[test]
    fn neither_a_time_nor_a_day() {
        let body = Body::new();
        body.capture(
            SESSION,
            &SESSION_CAPTURE
                .replace("at: 2026-10-09T22:21:18Z\n", "")
                .replace("event_date: 2026-10-09", "event_date: Tuesday"),
            &[],
        );
        let reading = body.read();
        assert!(reading.lines.is_empty());
        assert_eq!(reading.not_read[0].file, "capture.md");
        assert!(reading.not_read[0].why.contains("`event_date`"));
    }

    #[test]
    fn a_pointer_kept() {
        let body = Body::flywheel();
        // A pointer at a file here that cannot be read: the reading never
        // opens it.
        let raw = body.root().join("raw.jsonl");
        fs::write(&raw, "{}\n").unwrap();
        fs::set_permissions(&raw, fs::Permissions::from_mode(0o000)).unwrap();
        let pointer = format!("mac-studio:{}", raw.display());
        body.capture(
            SESSION,
            &SESSION_CAPTURE.replace(
                "raw: mac-studio:~/.local/state/crew/swancloud/raw/2026-10-09-swancloud-design-6d65f646.jsonl",
                &format!("raw: {pointer}"),
            ),
            &[],
        );
        let reading = body.read();
        fs::set_permissions(&raw, fs::Permissions::from_mode(0o644)).unwrap();
        assert_eq!(
            reading.lines[0].pointer.as_deref(),
            Some(
                "mac-studio:~/.local/state/swancloud/release-signals/raw/2026-10-08-release-pendingyou-0.32.0/"
            )
        );
        assert_eq!(reading.lines[1].pointer.as_deref(), Some(pointer.as_str()));
    }

    #[test]
    fn no_pointer() {
        let body = Body::new();
        body.capture(
            SESSION,
            &SESSION_CAPTURE.replace(
                "raw: mac-studio:~/.local/state/crew/swancloud/raw/2026-10-09-swancloud-design-6d65f646.jsonl\n",
                "",
            ),
            &[],
        );
        assert_eq!(body.read().lines[0].pointer, None);
    }

    fn signal(kind: &str, extra: &str, text: &str) -> String {
        format!("---\nsignal: x\nkind: {kind}\nwho: swancloud-design\n{extra}---\n\n{text}")
    }

    const SAID: &str = "It says so.\n\n> \"the words\" — 10:00:00Z\n";

    #[test]
    fn signals_numbered_with_a_gap() {
        let body = Body::new();
        body.capture(
            SESSION,
            SESSION_CAPTURE,
            &[
                ("03-b.md", &signal("ask", "", "B.\n\n> \"b\" — 2\n")),
                ("01-a.md", &signal("ask", "", "A.\n\n> \"a\" — 1\n")),
            ],
        );
        let signals = &body.read().lines[0].signals;
        assert_eq!(
            signals
                .iter()
                .map(|signal| (signal.n, signal.excerpt.as_str()))
                .collect::<Vec<_>>(),
            [(1, "a"), (2, "b")]
        );
    }

    #[test]
    fn signals_are_ordered_by_their_number_and_then_their_name() {
        let body = Body::new();
        let files: Vec<(&str, String)> = ["02-a.md", "01-z.md", "01-b.md"]
            .into_iter()
            .map(|file| (file, signal("ask", "", &format!("{file}.\n\n> {file}\n"))))
            .collect();
        let files: Vec<(&str, &str)> = files.iter().map(|(f, t)| (*f, t.as_str())).collect();
        body.capture(SESSION, SESSION_CAPTURE, &files)
            .file(&format!("signals/{SESSION}/notes.txt"), "not a signal")
            .file(&format!("signals/{SESSION}/1-a.md"), "not a signal")
            .file(&format!("signals/{SESSION}/01-A.md"), "not a signal");
        let signals = &body.read().lines[0].signals;
        assert_eq!(
            signals
                .iter()
                .map(|signal| signal.excerpt.as_str())
                .collect::<Vec<_>>(),
            ["01-b.md", "01-z.md", "02-a.md"]
        );
    }

    #[test]
    fn a_signal_of_no_nature() {
        let body = Body::flywheel();
        body.file(
            &format!("signals/{SESSION}/02-finding.md"),
            &signal("finding", "", SAID),
        );
        let reading = body.read();
        assert_eq!(keys(&reading), ["release/pendingyou/0.32.0"]);
        assert_eq!(reading.not_read.len(), 1);
        let not_read = &reading.not_read[0];
        assert_eq!(not_read.capture, SESSION);
        assert_eq!(not_read.file, "02-finding.md");
        assert!(not_read.why.contains("`kind`"), "{not_read}");
        assert!(not_read.why.contains("`finding`"), "{not_read}");
    }

    #[test]
    fn a_signal_lacking_a_field_leaves_its_capture_unread() {
        for (text, why) in [
            (
                "---\nkind: ask\n---\n\nIt says so.\n\n> words\n".to_string(),
                "`who`",
            ),
            (
                "---\nwho: x\n---\n\nIt says so.\n\n> words\n".to_string(),
                "`kind`",
            ),
            (signal("ask", "", "> \"the words\" — 1\n"), "asserts"),
            (
                signal("ask", "", "It says so, and quotes nothing.\n"),
                "excerpt",
            ),
            (signal("ask", "", "It says so.\n\n> \"\" — 1\n"), "excerpt"),
        ] {
            let body = Body::new();
            body.capture(SESSION, SESSION_CAPTURE, &[("01-a.md", &text)]);
            let reading = body.read();
            assert!(reading.lines.is_empty(), "{text}");
            assert_eq!(reading.not_read[0].file, "01-a.md");
            assert!(
                reading.not_read[0].why.contains(why),
                "{:?}",
                reading.not_read
            );
        }
    }

    #[test]
    fn a_sessions_signal() {
        let reading = Body::flywheel().read();
        assert_eq!(
            reading.lines[1].signals,
            [ImportLineSignal {
                n: 1,
                kind: Nature::Constraint,
                asserted_by: "swancloud-design".into(),
                assertion: "Claude Code does not fetch the tools again.".into(),
                subjects: vec![],
                argues_with: vec![],
                excerpt: "HTTP reconnection successful after 16216ms (attempt 1)".into(),
                position: "22:21:18Z".into(),
            }]
        );
    }

    #[test]
    fn a_releases_signal() {
        let reading = Body::flywheel().read();
        assert_eq!(
            reading.lines[0].signals,
            [ImportLineSignal {
                n: 1,
                kind: Nature::Ask,
                asserted_by: "npm:pendingyou".into(),
                assertion: "pendingyou 0.32.0 is released; the fleet pins 0.31.2.".into(),
                subjects: vec!["release".into(), "pendingyou".into()],
                argues_with: vec![
                    "packages/pendingyou/default.nix".into(),
                    "flake.nix inputs.herdr-pendingyou-src".into()
                ],
                excerpt: "## 0.32.0\n\n- `pendingyou hold` waits on a card.".into(),
                position: "https://www.npmjs.com/package/pendingyou/v/0.32.0, published \
                           2026-10-08T18:53:39Z"
                    .into(),
            }]
        );
    }

    #[test]
    fn a_quote_of_several_lines_crew_attributes() {
        let body = Body::new();
        body.capture(
            SESSION,
            SESSION_CAPTURE,
            &[(
                "01-a.md",
                &signal(
                    "reaction",
                    "subject: [mcp]\n",
                    "# A heading\nIt says so,\nover two lines.\n\n> \"first line\n>\n> last line\" — 22:21:18Z\n",
                ),
            )],
        );
        let signal = &body.read().lines[0].signals[0];
        assert_eq!(signal.excerpt, "first line\n\nlast line");
        assert_eq!(signal.position, "22:21:18Z");
        assert_eq!(signal.assertion, "It says so,\nover two lines.");
        assert_eq!(signal.subjects, ["mcp"]);
    }

    #[test]
    fn a_signal_with_no_attribution_is_placed_by_its_file() {
        let body = Body::new();
        body.capture(
            SESSION,
            SESSION_CAPTURE,
            &[(
                "01-a.md",
                &signal("question", "", "Is it so?\n\n> the words\n"),
            )],
        );
        let signal = &body.read().lines[0].signals[0];
        assert_eq!(signal.excerpt, "the words");
        assert_eq!(signal.position, format!("signals/{SESSION}/01-a.md"));
    }

    /// A capture of two signals, `01-a` and `02-b`, beside the release.
    fn two_signals() -> Body {
        let body = Body::flywheel();
        body.capture(
            "2026-10-05-planner-0a1b2c3d",
            &SESSION_CAPTURE.replace(SESSION, "2026-10-05-planner-0a1b2c3d"),
            &[
                ("01-a.md", &signal("ask", "", SAID)),
                ("02-b.md", &signal("ask", "", SAID)),
            ],
        );
        body
    }

    fn move_record(signal: &str, word: &str, fields: &str) -> String {
        format!("\nSignal: {signal}\nMove: {word}\n{fields}")
    }

    fn with_moves(body: &Body, records: &[String]) {
        body.file(
            "moves.rec",
            &format!("{MOVES_DESCRIPTOR}{}", records.concat()),
        );
    }

    fn moves_of<'a>(reading: &'a Reading, key: &str) -> &'a [ImportLineMove] {
        &reading
            .lines
            .iter()
            .find(|line| line.capture.key.as_str() == key)
            .expect(key)
            .moves
    }

    const TWO: &str = "2026-10-05-planner-0a1b2c3d";

    #[test]
    fn a_route_and_a_drop() {
        let body = two_signals();
        with_moves(
            &body,
            &[
                move_record(
                    &format!("{TWO}/01-a"),
                    "route",
                    "Target: unit/fix-x\nDate: 2026-10-06\nBy: swancloud-planner\n",
                ),
                move_record(
                    &format!("{TWO}/02-b"),
                    "drop",
                    "Reason: the same defect as another\nDate: 2026-10-06\nBy: swancloud-design\n",
                ),
            ],
        );
        let reading = body.read();
        let moves = moves_of(&reading, TWO);
        assert_eq!(
            moves[0].clone(),
            ImportLineMove {
                signal: 1,
                word: "route".into(),
                target: Some("unit/fix-x".into()),
                note: None,
                by: Some("swancloud-planner".into()),
                at: Some("2026-10-06T00:00:00Z".into()),
            }
        );
        assert_eq!(moves[1].signal, 2);
        assert_eq!(moves[1].word, "drop");
        assert_eq!(
            moves[1].target.as_deref(),
            Some("the same defect as another")
        );
        assert_eq!(moves[1].note.as_deref(), Some("the same defect as another"));
        assert!(reading.not_carried.is_empty(), "{:?}", reading.not_carried);
        assert!(moves_of(&reading, SESSION).is_empty());
    }

    #[test]
    fn a_signal_named_by_its_short_id() {
        let body = two_signals();
        with_moves(
            &body,
            &[move_record(
                &format!("{TWO}/02"),
                "answered",
                "Target: docs/adr/0002\nDate: 2026-10-07\nBy: d\n",
            )],
        );
        let reading = body.read();
        let moves = moves_of(&reading, TWO);
        assert_eq!(moves.len(), 1);
        assert_eq!(moves[0].signal, 2);
        assert_eq!(moves[0].word, "answered");
        assert_eq!(moves[0].target.as_deref(), Some("docs/adr/0002"));
    }

    #[test]
    fn a_signal_named_by_its_number_and_another_slug() {
        let body = two_signals();
        with_moves(
            &body,
            &[move_record(
                &format!("{TWO}/01-old-slug"),
                "route",
                "Target: unit/fix-x\nDate: 2026-10-06\nBy: p\n",
            )],
        );
        let reading = body.read();
        let moves = moves_of(&reading, TWO);
        assert_eq!(moves.len(), 1);
        assert_eq!(moves[0].signal, 1);
        assert!(reading.not_carried.is_empty(), "{:?}", reading.not_carried);
    }

    #[test]
    fn two_signals_of_one_number() {
        let body = Body::new();
        body.capture(
            SESSION,
            SESSION_CAPTURE,
            &[
                ("01-a.md", &signal("ask", "", SAID)),
                ("01-b.md", &signal("ask", "", SAID)),
            ],
        );
        with_moves(
            &body,
            &[move_record(
                &format!("{SESSION}/01"),
                "drop",
                "Reason: r\nDate: 2026-10-06\nBy: p\n",
            )],
        );
        let reading = body.read();
        assert!(moves_of(&reading, SESSION).is_empty());
        assert_eq!(reading.not_carried.len(), 1);
        let why = &reading.not_carried[0].why;
        assert!(why.contains("more than one signal numbered `01`"), "{why}");
        assert!(why.contains("01-a.md, 01-b.md"), "{why}");
    }

    #[test]
    fn a_signal_part_that_names_no_number() {
        let body = two_signals();
        with_moves(
            &body,
            &[
                move_record(
                    &format!("{TWO}/a"),
                    "drop",
                    "Reason: r\nDate: 2026-10-06\nBy: p\n",
                ),
                move_record(
                    &format!("{TWO}/1-a"),
                    "drop",
                    "Reason: r\nDate: 2026-10-06\nBy: p\n",
                ),
                move_record(
                    &format!("{TWO}/012"),
                    "drop",
                    "Reason: r\nDate: 2026-10-06\nBy: p\n",
                ),
            ],
        );
        let reading = body.read();
        assert!(moves_of(&reading, TWO).is_empty());
        assert_eq!(reading.not_carried.len(), 3);
        for not_carried in &reading.not_carried {
            assert!(
                not_carried.why.contains("names no signal's number"),
                "{not_carried}"
            );
        }
    }

    #[test]
    fn crews_words_are_carried_as_the_six() {
        let body = two_signals();
        with_moves(
            &body,
            &[
                move_record(
                    &format!("{TWO}/01-a"),
                    "attach",
                    "Target: intent/x\nReason: fits\nDate: 2026-10-07\nBy: d\n",
                ),
                move_record(
                    &format!("{TWO}/02-b"),
                    "challenge",
                    "Target: book/p.md\nDate: not a day\nBy: d\n",
                ),
                move_record(
                    &format!("{SESSION}/01-no-refetch"),
                    "drop",
                    "Target: noise\nDate: 2026-10-07\nBy: d\n",
                ),
            ],
        );
        let reading = body.read();
        let two = moves_of(&reading, TWO);
        assert_eq!(
            (two[0].word.as_str(), two[0].target.as_deref()),
            ("attach", Some("intent/x"))
        );
        assert_eq!(two[0].note.as_deref(), Some("fits"));
        assert_eq!(
            (two[1].word.as_str(), two[1].target.as_deref()),
            ("challenge", Some("book/p.md"))
        );
        assert_eq!(two[1].note, None);
        assert_eq!(two[1].at.as_deref(), Some("not a day"));
        let session = moves_of(&reading, SESSION);
        assert_eq!(
            (session[0].word.as_str(), session[0].target.as_deref()),
            ("drop", Some("noise"))
        );
        assert_eq!(session[0].note, None);
    }

    #[test]
    fn an_attach_with_its_reason() {
        let body = two_signals();
        with_moves(
            &body,
            &[move_record(
                &format!("{TWO}/01-a"),
                "attach",
                "Target: intent/x\nReason: the same ask as the standup's\nDate: 2026-10-07\nBy: d\n",
            )],
        );
        let reading = body.read();
        assert_eq!(
            moves_of(&reading, TWO),
            [ImportLineMove {
                signal: 1,
                word: "attach".into(),
                target: Some("intent/x".into()),
                note: Some("the same ask as the standup's".into()),
                by: Some("d".into()),
                at: Some("2026-10-07T00:00:00Z".into()),
            }]
        );
    }

    #[test]
    fn a_word_not_of_the_six_is_carried_as_it_stands() {
        let body = two_signals();
        with_moves(
            &body,
            &[move_record(
                &format!("{TWO}/01-a"),
                "promote",
                "Target: x\nDate: 2026-10-12\nBy: curation\n",
            )],
        );
        let reading = body.read();
        let moves = moves_of(&reading, TWO);
        assert_eq!(moves[0].word, "promote");
        assert_eq!(moves[0].target.as_deref(), Some("x"));
        assert_eq!(moves[0].by.as_deref(), Some("curation"));
    }

    #[test]
    fn crews_new_territory() {
        let body = two_signals();
        with_moves(
            &body,
            &[
                move_record(
                    &format!("{TWO}/01-a"),
                    "new-territory",
                    "Target: upstream: Claude Code's HTTP MCP reconnection\nReason: nothing covers it\nDate: 2026-10-10\nBy: d\n",
                ),
                move_record(&format!("{TWO}/02-b"), "new-territory", "Date: 2026-10-10\nBy: d\n"),
                move_record(&format!("{SESSION}/01"), "new-territory", "Target: — !\nDate: 2026-10-10\nBy: d\n"),
            ],
        );
        let reading = body.read();
        let two = moves_of(&reading, TWO);
        assert_eq!(two[0].word, "join");
        assert_eq!(
            two[0].target.as_deref(),
            Some("upstream-claude-code-s-http-mcp-reconnection")
        );
        assert_eq!(two[0].note.as_deref(), Some("nothing covers it"));
        assert_eq!(
            (two[1].word.as_str(), two[1].target.as_deref()),
            ("join", None)
        );
        let session = moves_of(&reading, SESSION);
        assert_eq!(
            (session[0].word.as_str(), session[0].target.as_deref()),
            ("join", None)
        );
    }

    #[test]
    fn a_move_of_a_signal_in_another_body() {
        let body = two_signals();
        with_moves(
            &body,
            &[
                move_record(
                    "2026-10-02-weekly-planning/01-pin-mise",
                    "route",
                    "Target: unit/pin\nDate: 2026-10-03\nBy: p\n",
                ),
                move_record(
                    &format!("{TWO}/03"),
                    "drop",
                    "Reason: r\nDate: 2026-10-03\nBy: p\n",
                ),
                move_record(
                    &format!("{TWO}/01-a"),
                    "drop",
                    "Reason: r\nDate: 2026-10-03\nBy: p\n",
                ),
            ],
        );
        let reading = body.read();
        assert_eq!(reading.lines.len(), 3);
        assert_eq!(moves_of(&reading, TWO).len(), 1);
        assert_eq!(reading.not_carried.len(), 2);
        let other = &reading.not_carried[0];
        assert_eq!(other.signal, "2026-10-02-weekly-planning/01-pin-mise");
        assert_eq!(other.word, "route");
        assert!(other.why.contains("no capture of this body"), "{other}");
        assert_eq!(
            other.to_string(),
            "move `route` of signal 2026-10-02-weekly-planning/01-pin-mise not carried: \
             no capture of this body is `2026-10-02-weekly-planning`"
        );
        assert!(
            reading.not_carried[1].why.contains("no signal `03`"),
            "{:?}",
            reading.not_carried
        );
    }

    #[test]
    fn a_move_of_a_capture_not_read_is_not_carried() {
        let body = two_signals();
        body.file(
            &format!("signals/{TWO}/03-c.md"),
            &signal("finding", "", SAID),
        );
        with_moves(
            &body,
            &[move_record(
                &format!("{TWO}/01-a"),
                "drop",
                "Reason: r\nDate: 2026-10-03\nBy: p\n",
            )],
        );
        let reading = body.read();
        assert_eq!(reading.lines.len(), 2);
        assert_eq!(reading.not_carried.len(), 1);
        assert!(
            reading.not_carried[0].why.contains("was not read"),
            "{:?}",
            reading.not_carried
        );
    }

    const STANDUP: &str = "2026-10-02-standup";

    /// A blueprints repository's corpus: one meeting capture of two
    /// signals, its old `signals/moves.rec` moving the second.
    fn corpus() -> Body {
        let body = Body::new();
        body.capture(
            STANDUP,
            &meeting_capture(STANDUP),
            &[
                ("01-a.md", &signal("ask", "", SAID)),
                ("02-b.md", &signal("ask", "", SAID)),
            ],
        )
        .file(
            "signals/moves.rec",
            &format!(
                "{MOVES_DESCRIPTOR}{}",
                move_record(
                    &format!("{STANDUP}/02-b"),
                    "drop",
                    "Reason: r\nDate: 2026-10-03\nBy: p\n"
                )
            ),
        );
        body
    }

    /// A flywheel branch's `moves.rec`, holding `records`, in a folder of
    /// its own.
    fn branch_moves(records: &[String]) -> (tempfile::TempDir, PathBuf) {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("moves.rec");
        fs::write(&path, format!("{MOVES_DESCRIPTOR}{}", records.concat())).unwrap();
        (dir, path)
    }

    #[test]
    fn a_blueprints_repositorys_corpus_with_the_flywheel_branchs_moves() {
        let body = corpus();
        let (_branch, moves) = branch_moves(&[move_record(
            &format!("{STANDUP}/01-a"),
            "route",
            "Target: unit/pin\nDate: 2026-10-03\nBy: p\n",
        )]);
        let reading = body.read_with(&moves);
        assert_eq!(keys(&reading), [STANDUP]);
        let carried = &reading.lines[0].moves;
        assert_eq!(carried.len(), 1, "{carried:?}");
        assert_eq!((carried[0].signal, carried[0].word.as_str()), (1, "route"));
        assert!(reading.not_carried.is_empty(), "{:?}", reading.not_carried);
    }

    #[test]
    fn a_blueprints_repositorys_corpus_read_alone() {
        let reading = corpus().read();
        assert_eq!(keys(&reading), [STANDUP]);
        assert!(reading.lines[0].moves.is_empty());
        assert!(reading.not_carried.is_empty(), "{:?}", reading.not_carried);
    }

    #[test]
    fn a_moves_file_named_is_read_in_place_of_the_roots() {
        let body = two_signals();
        with_moves(
            &body,
            &[move_record(
                &format!("{TWO}/01-a"),
                "drop",
                "Reason: r\nDate: 2026-10-03\nBy: p\n",
            )],
        );
        let (_branch, moves) = branch_moves(&[move_record(
            &format!("{TWO}/02-b"),
            "route",
            "Target: unit/x\nDate: 2026-10-03\nBy: p\n",
        )]);
        let reading = body.read_with(&moves);
        let carried = moves_of(&reading, TWO);
        assert_eq!(carried.len(), 1, "{carried:?}");
        assert_eq!((carried[0].signal, carried[0].word.as_str()), (2, "route"));
    }

    #[test]
    fn the_flywheel_branchs_own_moves_read_with_a_corpus() {
        let body = corpus();
        let (_branch, moves) = branch_moves(&[
            move_record(
                &format!("{RELEASE}/01-released"),
                "route",
                "Target: unit/bump-pendingyou\nDate: 2026-10-08\nBy: p\n",
            ),
            move_record(
                &format!("{STANDUP}/02-b"),
                "challenge",
                "Target: book/p.md\nDate: 2026-10-03\nBy: p\n",
            ),
        ]);
        let reading = body.read_with(&moves);
        assert_eq!(keys(&reading), [STANDUP]);
        assert_eq!(reading.lines[0].moves.len(), 1);
        assert_eq!(reading.lines[0].moves[0].signal, 2);
        assert_eq!(reading.not_carried.len(), 1);
        assert_eq!(
            reading.not_carried[0].signal,
            format!("{RELEASE}/01-released")
        );
        assert!(
            reading.not_carried[0]
                .why
                .contains("no capture of this body"),
            "{:?}",
            reading.not_carried
        );
    }

    #[test]
    fn a_moves_file_named_that_cannot_be_read() {
        let body = Body::flywheel();
        let dir = tempfile::tempdir().unwrap();
        let missing = dir.path().join("moves.rec");
        let why = read(&body.root(), Some(&missing)).unwrap_err();
        assert!(
            why.starts_with(&format!("{} cannot be read", missing.display())),
            "{why}"
        );
        let unreadable = dir.path().join("unreadable.rec");
        fs::write(&unreadable, MOVES_DESCRIPTOR).unwrap();
        fs::set_permissions(&unreadable, fs::Permissions::from_mode(0o000)).unwrap();
        let why = read(&body.root(), Some(&unreadable)).unwrap_err();
        fs::set_permissions(&unreadable, fs::Permissions::from_mode(0o644)).unwrap();
        assert!(why.contains("unreadable.rec cannot be read"), "{why}");
        let not_records = dir.path().join("not-records.rec");
        fs::write(&not_records, "not a record\n").unwrap();
        let why = read(&body.root(), Some(&not_records)).unwrap_err();
        assert!(why.contains("not-records.rec cannot be read"), "{why}");
    }

    #[test]
    fn a_moves_file_that_cannot_be_read() {
        let body = Body::flywheel();
        body.file("moves.rec", "not a record\n");
        let why = read(&body.root(), None).unwrap_err();
        assert!(why.starts_with("moves.rec cannot be read"), "{why}");
    }

    /// Every file under `root`, by its path within it, with its bytes.
    fn snapshot(root: &Path) -> BTreeMap<PathBuf, Vec<u8>> {
        let mut files = BTreeMap::new();
        let mut folders = vec![root.to_path_buf()];
        while let Some(folder) = folders.pop() {
            for entry in fs::read_dir(&folder).unwrap() {
                let path = entry.unwrap().path();
                if path.is_dir() {
                    folders.push(path);
                } else {
                    let bytes = fs::read(&path).unwrap();
                    files.insert(path.strip_prefix(root).unwrap().to_path_buf(), bytes);
                }
            }
        }
        files
    }

    fn printed(reading: &Reading) -> Vec<String> {
        reading.lines.iter().map(ImportLine::to_json).collect()
    }

    #[test]
    fn one_folder_read_twice() {
        let body = two_signals();
        with_moves(
            &body,
            &[
                move_record(
                    &format!("{TWO}/01-a"),
                    "route",
                    "Target: unit/fix-x\nDate: 2026-10-06\nBy: p\n",
                ),
                move_record(
                    &format!("{SESSION}/01"),
                    "new-territory",
                    "Target: upstream: x\nDate: 2026-10-10\nBy: d\n",
                ),
                move_record(
                    "elsewhere/01",
                    "drop",
                    "Reason: r\nDate: 2026-10-10\nBy: d\n",
                ),
            ],
        );
        let before = snapshot(&body.root());
        let first = body.read();
        let second = body.read();
        assert_eq!(
            snapshot(&body.root()),
            before,
            "the body's files are as they were"
        );
        assert_eq!(first, second);
        assert_eq!(printed(&first), printed(&second));
        assert_eq!(printed(&first).len(), 3);
        for line in printed(&first) {
            let read = weir_capture::read_import_line(&line).unwrap();
            assert_eq!(read.to_json(), line);
        }

        // The same body at another path, as another machine holds it.
        let elsewhere = tempfile::tempdir().unwrap();
        for (path, bytes) in &before {
            let to = elsewhere.path().join(path);
            fs::create_dir_all(to.parent().unwrap()).unwrap();
            fs::write(to, bytes).unwrap();
        }
        assert_eq!(
            printed(&read(elsewhere.path(), None).unwrap()),
            printed(&first)
        );
    }

    #[test]
    fn front_matter_is_read_as_crew_reads_it() {
        let (fields, body) = front_matter("---\na: 1\nb: x: y\nflag:\n\n---\n\n# T\n");
        assert_eq!(
            fields,
            [
                ("a".to_string(), "1".to_string()),
                ("b".to_string(), "x: y".to_string()),
                ("flag".to_string(), String::new()),
            ]
        );
        assert_eq!(body, "\n# T\n");
        assert_eq!(front_matter("# no front matter\n").0, []);
        assert_eq!(front_matter("---\na: 1\n").0, []);
    }
}
