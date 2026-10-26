# Design

## Context

weir has one crate, `weir-capture`, which holds the contract's lines,
the key rule, the sha256 hash, the capture record and `take`. This
change adds the built-in adapters and the command that runs them. The
statement (`agentplot/blueprints:design/weir/requirements.md` at main)
fixes what they do: the folder drop (61), the meeting file (62, and
section 10, question 4), the import, which is no adapter (63, 64, 75a,
as madswan-design amended them, blueprints main f4aabd9), imports
generally (37 to 39), the import line (C.1), the three commands (C.6;
section 9) and scenarios S8, S9 and S14. crew's folders are read as
crew writes them: crew's `README.md` (Signals), its `signals.md` rules
and `plugin/lib/plan.py` (`capture_text`, `signal_md`, `MOVES`), the
blueprints' `signals/README.md`, and swancloud's release adapter
(`packages/release-signals/release-signals.sh`), checked against a
flywheel branch's tree (`swancloud/main` of `afterthought/crew-state`).

The rules that bind are `docs/architecture/core.md`, the only page of
`docs/architecture/`; this change touches no area page, since none
exists. It rests on core.1, core.2, core.5, core.6, core.7, core.9,
core.11, core.14, core.25, core.34 and core.39; where weir has no rule
of its own, the moon projects follow kit:testing.1 and kit:testing.3.

The inbox's records and layout, a unit of this bolt being written
beside this one, owns the import operation in `weir-signals`, which
turns import lines into records (75a). This change owns everything
before it: the line's type and the reader of crew's folders.

## Goals / Non-Goals

**Goals:**

- The import line, read and written in `weir-capture` beside the three
  others, so the import operation reads what the import prints.
- The folder drop and the meeting file as adapters whose lines any host
  takes, and the reader of crew's folders printing import lines.
- `weir capture folder`, `weir capture meeting` and `weir import
  signals`.

**Non-Goals:**

- Writing an import into an inbox: the import operation, in
  `weir-signals` (75a). That a second import writes nothing is proved
  there; this change proves the second reading prints the same lines.
- The adapters' declarations: a built-in adapter's is built into the
  host (C.1), so millpond holds `[adapter.folder]` and
  `[adapter.meeting]`.
- The pack's adapter checks and meeting checks (65, 66): the pack's
  unit, which runs these adapters' output.
- `weir check` and `weir conform`: the kind format's and the pack's
  units add them to the command this change starts.

## Decisions

### Two crates: the adapters, and the command

`crates/weir-adapters` (moon project `adapters`) holds three modules,
`folder`, `meeting` and `signals_folder`, each a function from paths to
lines, and depends on `weir-capture` and `time`. `crates/weir` (package
`weir`, binary `weir`, moon project `weir`) holds the command line and
depends on both. Alternative: the binary inside `weir-adapters`.
Rejected: section 9 has the command run the pack and check kinds too,
and the adapters crate would then depend on the kinds crate and the
pack; the command is a caller of the crates, as core.1 says of it.

The arguments are read by hand from `std::env::args`: three commands,
two options. Alternative: `clap`. Rejected for now: one more dependency
tree for six words; the command's later units can bring it if `check`
and `conform` need it.

### The import line lives in `weir-capture`

`weir-capture` gains a module `import` with `ImportLine` (the capture
as `ImportLineCapture`, `pointer`, `came_in`, `signals` and `moves`),
`ImportLineSignal`, `ImportLineMove`, `read_import_line(&str) ->
Result<ImportLine, Malformed>` and `ImportLine::to_json`, reusing
`line`'s field readers, `Key::parse`, `Nature` and `CameIn`, so a
refusal names its field the same way (`import.signals[1].kind`). The
madswan-design ruling places it there, beside the three other lines.
`read_line` refuses a line holding `import` with a reason naming the
import operation, so `take` names it among a run's refusals (64; core.5).

An imported signal is `ImportLineSignal`, not the capture crate's
`OwnSignal`: an own signal's text is cut and its number is always 1,
while an imported signal's number is read and its excerpt never cut or
checked (37). A move's word is a `String`, not an enum of the six, so a
word of another body reaches the import operation, which leaves its
signal unmoved (39; core.25). Fields are read through `serde_json::Value`
as `line` reads them, and written by `serde` in C.1's order with absent
options left out.

The types are named for the line, `ImportLine*`, because the inbox's
records and layout unit names its own value `ImportedCapture`: the
capture, pointer, signals and moves its import operation writes, with
the key, source and event time as text so that operation refuses a
line naming no key whatever reader built it (75a). The line is handed
to the import operation through one conversion, `impl
From<weir_capture::ImportLine> for weir_signals::ImportedCapture`. It
lives in `weir-signals`, since `weir-signals` depends on `weir-capture`
and never the other way, and it is written by whichever of the two
units is built second: by this one when `crates/weir-signals` stands on
the branch it is built on, and otherwise by that unit, against this
one's types.

### The folder drop

`folder::run(dir, raw_dir) -> Vec<Line>` lists the folder's entries in
byte order of their names, passing over names beginning with `.` and
anything that is not a file once a symbolic link is followed. For each
file it copies the bytes into `<raw_dir>/<sha256>/<file name>`, hashing
them as they are copied with `content_hash_reader`, so the bytes banked
are the bytes keyed (6, 61; core.34). The copy goes to a temporary name
in the raw place and is renamed once its hash is known. A content seen
earlier in the run is not offered again: its copy is removed and the
file still counts as listed.

The folder's own name is its last path component once the path is
resolved; `Key::parse` checks `folder/<name>/<hash>` before any file is
read, and a refused name fails the run (4; core.7).

Hidden names are passed over because a folder a person drops files in
holds the system's own (`.DS_Store`, an editor's swap file), and a
capture of one is noise no reader should read. Subfolders are passed
over because 61 offers the files in a folder, and a key by the folder's
name alone could not tell two subfolders' files apart.

### Event times read a day as its start in UTC

A name beginning `YYYY-MM-DD` that names a day of the calendar gives
that day's start in UTC, `2026-10-02T00:00:00Z`; otherwise the file's
modification time is written in UTC, truncated to the second (5, 61).
The same rule reads crew's `event_date` and a move's `Date`. UTC, not
the machine's zone, so two machines listing one folder print the same
offers (3, 24; core.14). Alternative: the machine's local midnight.
Rejected: the event time would move with the machine that listed it.

### The meeting file

`meeting::run(file, raw_dir) -> Vec<Line>` offers the one file as the
folder drop offers a file, keyed `meeting/file/<sha256>` with source
`meeting` (62; C.6). Its start is read from a block the file opens with:
a first line `---`, lines `<name>: <value>`, and a closing `---`, as
crew's and Markdown's front matter are written; a `start` field there
that parses as RFC 3339 with an offset is kept as written. Otherwise the
event time is the folder drop's. A file that states its start some
other way, a WebVTT header or a `.docx` property, is read by its date or
last write until the design names that way.

### Neither adapter keeps a cursor

Both print no cursor and accept `--cursor` unread (8). Every run offers
every file again, and the host holds each key once (14). Alternative: a
cursor of the latest modification time. Rejected: a file copied in
keeps its old modification time and would never be offered.

### The reader of crew's folders

`signals_folder::read(root) -> Reading` returns the import lines, each
capture folder not read whole with why, and each move not carried with
why. It is the one module that names crew (core.2): its body name,
`crew-folders`, the folder layout, and its help text for the command.

- **The body.** `root` is a body's top: a flywheel branch's tree or a
  blueprints repository. Captures are the folders under `signals/`
  holding a `capture.md`; moves are `moves.rec` at the root, where the
  flywheel branch keeps them, and `signals/moves.rec`, where a
  blueprints repository did. `came_in` is `import:crew-folders`.
- **The front matter** is read as crew's `frontmatter` reads it: the
  lines between the opening `---` and the next, each split at its first
  `: `. A list, `[a, b]`, is its entries split at `, `.
- **The key.** `source: release` with a `key` keeps that key; any other
  capture's key is its folder's name, crew's lowercase words, digits and
  dashes, which the key rule takes (38; C.6).
- **The capture.** `source`; `event_at` from `at` (RFC 3339) or else
  `event_date`; `url`; the title from the first `# ` heading; the
  pointer from `raw`, byte for byte. The raw material is never opened
  (37; core.11).
- **A signal** is a file `NN-<slug>.md`, ordered by `NN` and then by
  name and numbered from 1. `kind` and `who` are fields; the excerpt is
  the `>` lines; the position is crew's attribution, the `" — <where>`
  ending a one-line quote, as `crew signal` writes it, or the `— <where>`
  line after a quote, as the release adapter writes it, and otherwise
  the file's path in the body; the assertion is the rest of the text,
  headings aside. A capture with a signal lacking any field is not read
  (28).
- **The moves** are read with `weir_capture::rec::read`, which skips
  comments; records without `Signal` (crew's `%rec` descriptor) are
  passed over. A signal is matched by its full id or its short one.
  crew's words map to the six by name, `drop` taking its `Reason` as its
  consequence. crew's `new-territory` becomes a join into a proposed
  item named for its target (section 10, question 3; ADR 0002; core.25):
  the name is the target lowercased with every run of characters other
  than ASCII letters and digits made one `-`, so it holds no `/` and
  lies as one folder (35). Any other word is carried as it stands (39).
  A record whose signal no capture of the body holds is reported, not
  carried.

The reading is a function of the body's files alone, sorted at every
step, so one body read twice gives the same bytes (S8; core.14).

### The command

`weir capture folder <dir> --raw-dir <raw> [--cursor <c>]`, `weir
capture meeting <file> --raw-dir <raw> [--cursor <c>]` and `weir import
signals <root>`. The adapters print each line with `Line::to_json` and
exit 0 after the completion; a run that cannot start prints one line on
standard error and exits 1, with no completion, so the host records it
incomplete (9; C.1). The import prints each line with
`ImportLine::to_json`, writes each capture not read and each move not
carried as one line on standard error, and exits 1 when any capture was
not read. A usage error exits 2.

### Tests build their bodies in code

Each crate's tests make folders, files, crew captures and `moves.rec`
in temporary directories (`tempfile`), as `weir-capture`'s do, so no
fixture file type enters the tree for treefmt to claim. A test that
needs a file it cannot read makes one with no read permission. The
command's tests run the built binary (`CARGO_BIN_EXE_weir`), and the
test of S14 takes its lines with `weir_capture::take` into an in-memory
inbox and bank kept in the test.

### The moon projects

`adapters` (`crates/weir-adapters`) and `weir` (`crates/weir`), each
listed in `.moon/workspace.yml` before `root`, with `language: 'rust'`
and four tasks tagged `check-fast` and run from the workspace root:
`format` (`cargo fmt -p <crate> --check`), `lint` (`cargo clippy -p
<crate> --all-targets -- -D warnings`), `typecheck` (`cargo check -p
<crate> --all-targets`) and `test` (`cargo test -p <crate>`), each
declaring as inputs the crate, the crates it depends on,
`/Cargo.toml`, `/Cargo.lock`, `/devenv.nix`, `/devenv.lock` and
`/.prototools`, as `capture`'s do (core.39; kit:testing.1,
kit:testing.3).

## Risks / Trade-offs

- [The import operation is written in another unit, beside this one]
  → The line's shape is C.1's, read in one place, `read_import_line`,
  and handed over by one conversion, written by whichever unit is built
  second. When this one is built first, no test here needs
  `weir-signals`; when second, the conversion's test runs in that
  crate's project.
- [S9 says crew's `new-territory` arrives unmoved, while section 10's
  ruling on question 3, ADR 0002 and core.25 make it a join] → This
  change follows the ruling and the rule, the later word; S9's case is
  proved with a word of no body's, `promote`.
- [A blueprints corpus's moves lie in the flywheel branch's
  `moves.rec`] → Importing the corpus alone reads only the moves its
  own `signals/moves.rec` holds, and importing the branch names the
  corpus's moves as not carried. Nothing is lost silently: each is named
  on standard error.
- [crew's moves carry a `Reason` beside a target, and `weir-move/1`
  has no field for it] → Only a drop carries it, as its consequence;
  every other move's reason is not carried.
- [Copying every file of a folder on every run costs time on a large
  folder] → The host banks only what it takes and discards the rest
  (18); a cursor that skipped files would lose some (8).

## Open Questions

- Where a meeting file states its start beyond an opening block's
  `start:`, for madswan-design.
- How a blueprints corpus's signals receive the moves recorded on the
  flywheel branch, for madswan-design.
