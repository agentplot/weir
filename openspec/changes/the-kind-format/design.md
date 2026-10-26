# Design

## Context

The workspace holds no crate yet and no Rust toolchain: `Cargo.toml`
lists `members = []`, and moon runs only the root's `format` and
`prose`. This change brings the first Rust, so it also brings the
toolchain every later crate shares. Two other units of the bolt, the
capture contract and cadence, are planned beside this one and need the
same toolchain; whichever lands first carries it, and the others take
it at their merge.

It satisfies clauses 51 to 58, 81 and 82 of the statement, with C.4 as
the profile and S10 to S12 as its scenarios. flume's C.2 is the reader
this crate serves first: it names the same fields, adds a `[profiles]`
table of its own beside the kinds, and keeps its own defaults for
`quiet`, `on_fail` and `max_send_backs`. sounding's C.3 names the
elaboration section's fields and their meaning.

There are no area pages under `docs/architecture/`; `core.md` binds
everywhere this change goes.

## Goals / Non-Goals

**Goals:**

- One reader of the format that flume, sounding and later Flywheel call
  and none rewrites (core.30).
- A refusal a person can act on: every failing field by its path, with
  its reason.
- `weir check` for whoever writes a declaration, with no tool of the
  family installed.

**Non-Goals:**

- Which declaration a kind's name resolves to, among a tool's own, a
  repository's and a launch's (58, core.32): a caller hands the crate
  one declaration's text and gets its kinds.
- Finding `.config/<tool>.toml` at a revision, or reading a prompt's
  file: the prompt comes back as written, and resolving it against the
  declaration's file is the reader's (C.4).
- A reader's own refusals, such as flume refusing a stage with two
  agents (flume 23): weir reads what the format allows.
- Defaults a reader keeps: flume's 30-minute quiet, its send-back
  default, sounding's ending when none is declared.

## Decisions

### Two crates: the reader and the command line

`crates/weir-kinds` is the library, named in section 9. `crates/weir`
is the `weir` command line, a caller of the crates (core.1 details) and
the binary section 9 names; this change gives it `check` alone, and the
built-in adapters and the pack add their commands to it later. The
command line is its own crate so that a tool depending on
`weir-kinds` pulls in no argument parser.

*Alternative:* a `[[bin]]` inside `weir-kinds`. Rejected: every later
command would then make the kinds crate depend on the capture and
adapter crates.

### Walk a parsed table by hand, not serde derive

The text is parsed into a `toml::Table`, and the reader walks it field
by field. Derive would stop at the first error and drop the fields it
does not know. Walking lets the reader collect every failing field in
one pass (57) and keep each field the format does not name as a section
(56) without a catch-all on every struct.

### What a read returns

- `read_kinds(text) -> Result<Declaration, Refusal>` is `read-kinds`
  (81). A `Declaration` holds `kinds`, a map from name to `Kind`
  ordered by name, and `sections`, every top-level key but `kinds`.
- A `Kind` holds its name, `version` (at least 1; 1 when left out,
  57), `branch` (its name when left out, C.4), `starts_unready` and
  `lands` (false when left out), `stages` in the order written, and
  `sections`: every key the format does not name, `elaboration` among
  them (55, 56).
- A `Stage` holds `name`, `prompt` as written, `end` (one
  `Deliverable` or `Conversation`), `agents` (each a table of its
  settings, read as written, since an agent's settings are its runner's
  (53)), `wait` (false when left out), `quiet` as an
  `Option<Duration>`, `on_fail` and `max_send_backs` as options, and
  `sections`.
- `Deliverable` is `Commits`, `Change`, `Tasks`, `Archived`,
  `Report(path)` or `Committed(path)`, the path a non-empty string
  after the prefix (54, C.4).
- Unknown values are `toml::Value`s, and the crate re-exports `toml`
  so that a caller names the same version. A value type of weir's own
  would copy TOML's for nothing.

Everything derives `PartialEq` and `Debug`, so a test compares two
reads (58).

### The defaults the format applies, and the ones it leaves

The format applies a default only where the statement or C.4 gives
one, or where a flag's absence can mean only false: `version` 1,
`branch` the kind's name, `starts_unready`, `lands`, `wait` and
`conversation` false. `quiet`, `on_fail`, `max_send_backs` and an
elaboration's `ending` come back as `None` when left out, because
their defaults differ by reader (flume C.2; sounding 30).

### The checks

Each check names a field by its path from the declaration's root,
stages by their index from 0: `kinds.bump.stages[1].prompt`. A
`Refusal` holds every `Problem { field, reason }` found, in the order
the reader walks the declaration, and returns no kind. Text that is not
TOML gives one problem whose field is the line and column toml's span
points at.

Refused, each naming the field:

- `kinds`, or a `kinds.<name>`, that is not a table;
- `version` not an integer of at least 1; `branch` not a non-empty
  string; `starts_unready`, `lands`, `wait` or `conversation` not a
  boolean;
- `stages` missing, empty, or not an array of tables (51, 57);
- a stage's `name` or `prompt` missing, empty or not a string (57);
- `deliverables` not an array of strings, with other than one entry, or
  with a spelling not of the six; a stage with neither one deliverable
  nor `conversation = true`; a conversation with `deliverables` (51,
  54);
- `agents` not an array of tables; `quiet` not a duration;
  `max_send_backs` not an integer of at least 0;
- `on_fail` not a string, or naming no stage of its kind (52);
- a stage name used twice in one kind.

The last two are this design's reading of 52: a send-back "names the
stage", which a name shared by two stages does not do.

### quiet's grammar

`quiet` is a string of one to three parts, hours, minutes and seconds
in that order, each a whole number followed by `h`, `m` or `s`:
`"45m"`, `"1h30m"`, `"90s"`. Any other string, or a value not a
string, is refused naming the field. The format reads it into a
`Duration` so that two readers never read one declaration's quiet time
two ways (58). C.4's example, `"45m"`, is the only form the statement
shows.

### The elaboration section

`read_elaboration(&Kind) -> Result<Option<Elaboration>, Refusal>` is
the elaboration reader's read (55). It reads the kind's
`sections["elaboration"]`, returns `None` when there is none, and
checks the section whole the way `read_kinds` checks the rest:
`inputs`, `conclusion` and `lands` arrays of strings (empty when left
out), `ending` one of `self-closing`, `standing`, `with-operator` (an
`Ending`, `None` when left out), `transcripts` a boolean (false when
left out), and any other key kept in the section's own `sections`.
`read_kinds` never looks inside it, so a reader of places never fails
on it (56, core.30).

### weir check

`weir check <path>` reads the file, calls `read_kinds`, then
`read_elaboration` for each kind, and gathers every problem of both
into one refusal. On success it prints, per kind in name order, a line
`<name>@<version>` and one indented line per stage,
`<stage>: <deliverable spelling>` or `<stage>: conversation`, and a
line `elaboration: <ending or "no ending">` for an elaboration kind,
then exits 0. On refusal it prints one line per problem,
`<field>: <reason>`, on standard error and exits 1. A file it cannot
read is named with the system's reason on standard error, exit 2. It
opens the file read-only and writes nothing (58).

### The toolchain

Rust is pinned in `.prototools` at 1.98.1, the version tailrace runs,
and read by `.moon/toolchains.yml` with `versionFromPrototools`
(kit:toolchain.2). `rust-toolchain.toml` names the same channel with
`clippy` and `rustfmt`, so rustup runs that toolchain anywhere in the
tree. `check-toolchain.sh`, run by a root `toolchain` task tagged
`check-fast` and never cached, fails when the two pins differ or when
`rustc`, `cargo`, `clippy` or `rustfmt` on the path is another version
(core.40). treefmt gains `rustfmt`, through a devenv wrapper that
activates the pinned toolchain, and `shellcheck` with `shfmt` for the
script (core.39). tailrace's setup is the precedent for each piece.

Each crate is a moon project, `weir-kinds` and `weir`, with `format`
(`cargo fmt -p <crate> --check`), `lint` (`cargo clippy -p <crate>
--all-targets -- -D warnings`), `typecheck` (`cargo check -p <crate>
--all-targets`) and `test` (`cargo test -p <crate>`), each tagged
`check-fast` and declaring the crate's files, `/Cargo.toml`,
`/Cargo.lock`, `/rust-toolchain.toml`, `/devenv.nix`, `/devenv.lock`
and `/.prototools` as inputs (core.39, kit:testing.1, kit:testing.3).
`weir`'s inputs take `weir-kinds`'s files too.

### Tests and their fixtures

The crate's tests read fixture declarations from
`crates/weir-kinds/tests/fixtures/`, each valid TOML so taplo formats
it, and keep text that is not TOML inline in the test. The fixture for
S10 is named for its kinds, `fix-and-change.toml`, and S11's
`elaboration.toml`: neither file nor test names a tool of the family
(core.2). `weir`'s tests run the built binary
(`CARGO_BIN_EXE_weir`) against the same fixtures, and compare a
fixture's bytes before and after.

### Rules this design relies on

core.1 (the crate answers from its inputs; the command line is the
caller that reads the file), core.2, core.3 (a declaration names a
secret only by name; nothing here reads one), core.30, core.31,
core.32, core.37, core.39, core.40; kit:testing.1, kit:testing.3,
kit:testing.5, kit:toolchain.2. It departs from none.

## Risks / Trade-offs

- [The other two units of the bolt add the same toolchain files] →
  each names them in its touches, so crew names the overlap; the second
  to merge takes the first's files and drops its own copy.
- [quiet's grammar, the duplicate-name check and the elaboration
  defaults are this design's, not the statement's] → each is small and
  stated here; if the statement fixes them otherwise, the change is to
  one check and its test.
- [Re-exporting `toml` ties callers to its version] → a breaking bump
  of `toml` is a breaking bump of `weir-kinds`; the family's tools
  depend on weir at a tag.
