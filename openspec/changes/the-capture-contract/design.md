# Design

## Context

weir is a Cargo workspace with no members, no Rust toolchain and one moon
project, `root`, whose `check-fast` tasks run treefmt and Vale. This is
the first crate. The statement (`agentplot/blueprints:design/weir/requirements.md`
at main) fixes what it must do: the lines and the offer (1 to 12), what a
host does with them (13 to 24), the adapter's and host's operations
(72 to 77), the JSON Lines and declaration profile and the seven steps
(C.1), the capture record's fields (C.2), and scenarios S1 to S4. Section
9 places the contract's types and key rules, JSON in flight, recutils at
rest, the sha256 hash and the cut in `weir-capture`, and the signal,
move, batch, item, brief and send records and the inbox's layout in
`weir-signals`, a later unit of this bolt. The capture design's table of
who owns what (`design/capture/design.md`, "What the shared crates
hold") says the same.

The rules that bind are `docs/architecture/core.md`, the only page of
`docs/architecture/`; this change touches no area page, since none
exists. Where weir has no rule of its own (the toolchain and the moon
tasks), the kit standard's rules bind and are cited as `kit:<id>`.

## Goals / Non-Goals

**Goals:**

- One crate, `weir-capture`, that reads and writes an adapter's lines,
  checks keys, reads a declaration and checks a configuration against
  its schema, hashes content, cuts an own excerpt, writes the capture
  record, and takes lines into a host's inbox and bank in the seven
  steps of C.1, identically for every host.
- Rust in the workspace, pinned once and held to its pin, with the
  crate as a moon project whose checks run in `check-fast`.

**Non-Goals:**

- The own signal's record, `weir-signal/1`, and where any record lies in
  `inbox/`: `weir-signals`, a later unit, which renders the own signal
  this crate hands it.
- Writing a shared branch: `weir-branch`, which an inbox implementation
  calls; this crate never touches git (47).
- Starting an adapter's process, reading a host's settings or the
  machine's name, choosing a limit or a bank's place: the host's, which
  hands each in (core.1).
- The built-in adapters and the conformance pack: later units, which
  build on this crate's line writer, hash and `take`.
- A run record format. C.2 names none; the report is a value the host
  keeps or prints as JSON (23, 76).

## Decisions

### The crate is a library of pure functions and one driver over two seams

Modules: `key`, `line` (offer, excerpt, cursor, completion; reading and
writing), `declaration`, `hash`, `cut`, `rec`, `capture` (the
`weir-capture/1` record) and `take`. Every function answers from its
arguments (core.1). The machine's name, the taken-at time, the limit,
how the capture came in and the raw place are arguments of `take`,
never read from the environment (22; core.1, core.14).

`take` drives the seven steps itself, calling two traits the host
implements:

- `Inbox`: `holds(&Key) -> bool`, and `write(&Taken) -> Result<(), Refusal>`,
  where `Taken` is the capture record with its own signal when it has
  one, handed together so they land in one write (19; core.13).
- `Bank`: `bank(&Key, files, whole_excerpt) -> Result<String, Refusal>`,
  returning the path on the machine where the capture's material now
  lies, and `discard(&Key)`.

Alternative: `take` returns a plan and the host carries it out. Rejected:
the order bank-then-write and held-means-nothing-banked would then be
each host's to keep, and 24 says what a host does is fixed by the
statement, not the host. A trait call is a write the caller asked for,
so core.1 holds.

Within one call, `take` remembers the keys it has taken, so a key offered
twice in one set of lines is held the second time even if the host's
`Inbox` answers `holds` from a tip read before the call (14).

### The own signal is a value, not a record

`take` hands the inbox an `OwnSignal`: identity `<key>#1`, every excerpt
field as offered, the excerpt text as cut, and the lines and bytes left
out when cut. Rendering it as `weir-signal/1` is `weir-signals`', which
depends on this crate. Alternative: render `weir-signal/1` here. Rejected:
it would give the signal format two homes, against section 9's split
and core.22's one reader of a format.

### recutils is written by the crate, checked by `recfix`

The `rec` module writes and reads records as GNU recutils: one
`name: value` line per field in a fixed order, a value's later lines as
`+ ` continuation lines, a field with no value left out, records
separated by a blank line. `weir-signals` reuses it for its records.
`Capture::to_rec` and `Capture::from_rec` use it, naming
`format: weir-capture/1` first (40; core.22).

The field order is C.2's: `format`, `key`, `source`, `event_at`, `title`,
`url`, `pointer`, `came_in`, `machine`, `taken_at`, `unread`, with
`unread` spelled `yes` or `no`, recutils' boolean, and `taken_at` in RFC
3339 in UTC.

Tests hold every record the crate writes to `recfix` (GNU recutils,
added to `devenv.nix`'s packages), so a record a later write would
refuse (core.24) is caught here. Alternative: bind librec or shell out
from the crate. Rejected: the crate would start a process or link a
system library at run time (core.1); only the tests run `recfix`.

### Lines are read through `serde_json::Value`

A line is parsed to a JSON value first, then checked for exactly one of
`capture`, `cursor` and `done` (1; core.5), then each field checked by
name, so a refusal names the field (`event_at`, `excerpt.text`,
`done.failed[0].why`) and an unknown field is ignored (core.6). A
cursor must be a string, since it is handed back as `--cursor <c>`
(C.1), and is kept byte for byte (core.9). `event_at` is checked with the
`time` crate's RFC 3339 parser, which requires an offset, and kept as
the offered string (5, 16; core.8).

Writing a line uses `serde_json` with no pretty-printing, so a newline
inside a value is escaped and the line stays one line.

### The key rule

`Key::parse` refuses an empty key, a leading `/`, a `..` segment and a
first segment of `captures`, `moves` or `items` (4; C.1; core.7). It also
refuses an empty segment (`a//b`, a trailing `/`) and a `.` segment, since
each would name the same folder as another key once segments become
folders (42), and two events never share a place.

### Raw paths are checked against the raw place on disk

Each `raw` entry must be relative with no `..` segment (6). When taking,
`take` joins it to the raw place, resolves it, and refuses the offer when
the resolved file is not inside the resolved raw place (a symbolic link
out) or cannot be read (18). Taking with no raw place refuses any offer
that names raw material, as the capture design's intake does.

### The cut is one function with a fixed closing line

`cut(text, limit) -> Cut` is the only cut (core.12). Lines end at `\n`.
Within the limit, the text is returned whole and `Cut::left_out` is
`None`. Otherwise it keeps the longest prefix ending just after a `\n`
such that the prefix plus the closing line is within the limit, and
appends the closing line:

```
[cut: <n> lines and <b> bytes left out, banked whole with the capture]
```

`<n>` counts the lines of the remainder (a last line with no `\n`
counts), `<b>` its bytes, singular `line`/`byte` at 1. When no line
fits, the closing line alone is the excerpt (20). `Take::new` refuses a
limit shorter than the closing line with both numbers at their widest,
so the closing line always fits. Cutting at a `\n` never splits a UTF-8
character.

The statement fixes what the closing line says, not its words; this
crate fixes them once so every host writes the same (24).

### The cut text is banked beside the raw files

When an excerpt is cut, `take` passes the whole text to `Bank::bank` as
`whole_excerpt`, in the same call as the raw files, before `Inbox::write`
(21). The bank chooses where it lies under the capture's place. A capture
with only an uncut excerpt and no raw files banks nothing and holds no
pointer (17).

### A failed write unbanks

When `Inbox::write` refuses, `take` calls `Bank::discard(&key)` and
reports the offer refused with the inbox's reason, so only what a taken
capture names is banked (18).

### The run's report

`take` returns a `Report`: the keys taken, the held count, each refusal
(line number, key where one was read, reason), the completion's listed
count and failures, and the last cursor read. `Report::finish(exited_ok)`
says complete only when a completion was read and the adapter exited
reporting success, and keeps the cursor only then (9, 23; core.15). An
intake calls `finish(true)` and keeps no cursor, since no adapter ran
(design: "only a run keeps a cursor"). The report serializes to JSON for
the host (C.2: JSON only in flight).

### The declaration

`read_declarations(toml) -> Result<Vec<Declaration>, DeclarationError>`
reads every `[adapter.<name>]` table with the `toml` crate, naming the
first field missing or of the wrong shape (`adapter.x.command`).
`check_config(schema, config)` validates with `jsonschema` 0.60's
`draft202012` validator built with `default-features = false`, so the
`resolve-http` and `resolve-file` features are off and no `$ref` is
fetched (68; core.1), and names each failing instance path.
`Declaration::invocation(raw_dir, cursor)` returns the argument list and
the environment variable names (`CAPTURE_CONFIG`, then each secret's
name); it starts nothing and holds no secret's value (11, 73; core.3).

### The hash

`content_hash(bytes)` and `content_hash_reader(reader)` return sha256 as
lowercase hex, through the `sha2` crate (C.6).

### Dependencies

`serde` with `derive`, `serde_json`, `toml`, `time` with `parsing` and
`formatting`, `sha2`, and `jsonschema` with default features off, each
in `[workspace.dependencies]`; `tempfile` for tests. Versions are the
current releases the coder resolves into `Cargo.lock`.

### The toolchain and the moon project

As tailrace does, the family's Rust precedent:

- `.prototools` pins `rust = "1.98.1"`; `.moon/toolchains.yml` reads it
  with `versionFromPrototools` (kit:toolchain.2).
- `rust-toolchain.toml` names the same channel with `clippy` and
  `rustfmt`, `profile = "minimal"`, so rustup never reaches a host's
  default toolchain; `check-toolchain.sh`, run by a root `toolchain`
  task tagged `check-fast` and never cached, fails when the two pins
  differ or the tools on `PATH` are not the pinned build (core.40).
- `treefmt.nix` enables `rustfmt` (edition 2021) for `.rs` and `shfmt`
  for `.sh` (core.39; kit:agents.4); `Cargo.toml` and
  `rust-toolchain.toml` are already claimed by taplo's `*.toml`.
- The crate is `crates/weir-capture`, in the workspace's `members`, and
  the moon project `capture` in `.moon/workspace.yml`, with
  `language: rust` and four tasks, each tagged `check-fast` and run from
  the workspace root: `format` (`cargo fmt -p weir-capture --check`),
  `lint` (`cargo clippy -p weir-capture --all-targets -- -D warnings`),
  `typecheck` (`cargo check -p weir-capture --all-targets`) and `test`
  (`cargo test -p weir-capture`). Each declares its inputs: the crate,
  `/Cargo.toml`, `/Cargo.lock`, `/rust-toolchain.toml`, `/devenv.nix`,
  `/devenv.lock` and `/.prototools` (core.39; kit:testing.1,
  kit:testing.3).
- Tests build their inputs in code and in temporary directories, so the
  change adds no fixture file type for treefmt to claim.

## Risks / Trade-offs

- [Two other units of this bolt, the kind format and cadence, are
  planned beside this one and each needs the Rust toolchain] → This
  change brings the toolchain whole and names its shared files in
  `## Touches`; crew orders the units on the overlap, and the later
  one's code builds on this toolchain instead of adding its own.
- [The closing line's words are this change's, not the statement's] →
  They are fixed in one function and one test, so a later change of
  words reaches every host at once through the shared crate; excerpts
  already written keep the words they were cut with (41).
- [A pointer's path is the bank's, which is the host's] → Two hosts
  with different banks write different pointer paths. The scenarios of
  24 hold the bank fixed; the pack's later cases do the same.
- [`jsonschema` brings many transitive crates] → Default features off
  keeps out networking and TLS; it is the maintained draft 2020-12
  implementation, which the ruling of section 10, question 8 requires.

## Open Questions

- Whether a key segment may equal a record's file name (`x/capture.rec`
  lies where capture `x`'s record does). The layout is fixed by the
  inbox's records unit; this crate's key rule follows C.1 as written.
