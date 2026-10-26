//! The signals-folder import: crew's folders read into import lines, one
//! per capture, each with the pointer, signals and moves it was read with
//! (37 to 39, 63; C.6; S8, S9). It is no adapter, and writes nothing: a
//! host's import operation writes its lines (64, 75a). This is the one
//! module of the crate that names crew (core.2).
//!
//! A body is a root holding `signals/`: a partition's flywheel branch, or a
//! blueprints repository's meeting corpus. Each folder under `signals/`
//! holding a `capture.md` is one capture, read in the order of the folders'
//! names; anything else there is passed over. The reading is a function of
//! the body's files alone, sorted at every step, so one body read twice
//! gives the same lines (38; S8; core.14), and no file a pointer names is
//! ever opened (37; core.11, core.25).

use std::fmt;
use std::fs;
use std::path::Path;

use time::format_description::well_known::Rfc3339;
use time::OffsetDateTime;
use weir_capture::{CameIn, ImportLine, ImportLineCapture, Key};

use crate::event_time::day_start;

/// The body crew's folders are, as each line's `came_in` names it (22).
pub const BODY: &str = "crew-folders";

/// What `weir import signals <root>` does, for the command's help.
pub const HELP: &str = "print one import line per capture of crew's signals folders under \
                        <root>/signals/, with its pointer, signals and moves, for a host's \
                        import operation to write";

/// What a body's reading gives: one line per capture read whole, each
/// capture not read whole with why.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Reading {
    pub lines: Vec<ImportLine>,
    pub not_read: Vec<NotRead>,
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

/// The body at `root` read into import lines, or why it cannot be read: a
/// root with no `signals/` folder.
pub fn read(root: &Path) -> Result<Reading, String> {
    let signals = root.join("signals");
    if !signals.is_dir() {
        return Err(format!(
            "{} holds no signals/ folder of crew's",
            root.display()
        ));
    }
    let mut reading = Reading::default();
    for capture in capture_folders(&signals)? {
        match read_capture(&signals.join(&capture), &capture) {
            Ok(line) => reading.lines.push(line),
            Err((file, why)) => reading.not_read.push(NotRead { capture, file, why }),
        }
    }
    Ok(reading)
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

/// A capture's line, or the file it could not be read whole from and why.
fn read_capture(folder: &Path, name: &str) -> Result<ImportLine, (String, String)> {
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
    Ok(ImportLine {
        capture: ImportLineCapture {
            key,
            source,
            event_at,
            title,
            url: field("url").map(str::to_string),
        },
        pointer: field("raw").map(str::to_string),
        came_in: CameIn::Import(BODY.to_string()),
        signals: Vec::new(),
        moves: Vec::new(),
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
            read(&self.root()).unwrap()
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
        let why = read(dir.path()).unwrap_err();
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
