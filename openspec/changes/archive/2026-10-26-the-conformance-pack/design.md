# Design

## Context

The pack today is fifteen cadence cases, one TOML file each under
`pack/cadence/`, answered by `weir-cadence`'s `tests/pack.rs`. The
shared-branch unit (`a-shared-branch-takes-one-write`, archived on its
branch, not yet in the bolt) adds `crates/weir-branch` and its cases at
`pack/branch/<case>/case.toml`, answered by its own `tests/pack.rs`.
That is the pattern this change extends to every shape: the cases are
files in the pack, and the crate that implements a profile answers them
in its tests (C.7, as amended by ae9d782: "a case of the last four runs
against weir's own crate in its tests, as cadence's do").

What weir already has to build on: `weir-capture` holds `read_line`
(which refuses a malformed line naming why), the key rule, `take` and
its `Report`, which already serializes the fields C.7 names but is not
yet a named format; `weir-signals` holds `InboxAt::read` over the files
of one tip and an operation per record, each forming a `Write` or a
`Refused` naming each thing not taken, and `InboxAt::import`;
`weir-kinds` holds `read_kinds` and `read_elaboration`; `weir-adapters`
holds the folder drop and the meeting file, each a function from its
input and a raw place to lines; `weir-branch` holds `read` and `write`
over a local branch with no remote.

The seam with a host under test is fixed in the statement (weir C.7,
blueprints main 81f3f2d and ae9d782): a bare repository with no remote,
branch `inbox`, the flags `--inbox --branch --limit`, then
`--raw-dir`, `--run <declaration>` or `--import`, and the
`weir-report/1` object printed last.

Rules this design relies on: core.1 (crates answer from their inputs;
the command line is a caller), core.2 (no tool named outside the pack's
cases), core.5 and core.6 (the adapter checks), core.14 (what a host may
put in a record of its own), core.15 (the run's report), core.22 (the
report names its format), core.35 (every case cites; the check; no
network), core.36 (every profile passes), core.37 (the clause index
holds numbers, never words), core.39 and core.40 (the new crate's moon
project and pinned tools); kit:testing.1, kit:testing.3, kit:testing.5
and kit:specs.3 for the checks that prove it. There are no area pages
in `docs/architecture/` yet; core.md binds every path this change
touches.

Clauses satisfied: 65, 66, 67 (as amended), 68, 86, 87; through the
cases, every clause of A.1 to A.10 and 71; S15, S16; C.7.

## Goals / Non-Goals

**Goals:**

- Every shape C.7 names has its cases in `pack/<shape>/<case>/`, and
  the crate that implements the profile answers them in its tests.
- One check, shape-agnostic, proves 67 over the whole pack.
- `weir conform` holds any host, in any language, to the host cases
  through the command line alone.
- An adapter's own tests can hold it to the adapter checks and, for a
  meeting adapter, the meeting checks.

**Non-Goals:**

- The branch cases themselves: they are the shared-branch unit's. This
  change counts them in the check and moves none of them.
- Holding crew's Python twin of the write protocol to the branch cases
  (C.3): crew's work.
- A command that runs an adapter under the adapter checks: section 9
  names none, so the checks are a library an adapter's tests call.
- Holding millpond: millpond passes the pack in its own repository
  (plan P3).

## Decisions

### A crate for the pack, `weir-pack`, free of processes

`weir-pack` holds: reading every case's `satisfies` and the clause
index, and the check of 67; reading host cases and comparing a
`weir-report/1` and a tip's files with a case; the adapter checks and
the meeting checks; and the pack's files, built in. It runs no program
and reads no repository: the `weir` command line makes the temporary
repository, runs git through `weir-branch`, runs the host and reads the
tip, then hands what it found to `weir-pack` to compare (core.1).

*Alternatives:* putting the check and the comparisons in the `weir`
binary would leave adapter authors nothing to link for the adapter
checks; putting them in `weir-capture` would load the capture crate
with inbox and kinds concerns it does not own.

### The check reads `satisfies` alone, from every shape

Each shape's crate reads its own case format whole and refuses an
unknown field (as cadence's and the branch's tests already do). The
check reads each `case.toml` only as a TOML table and takes
`satisfies`, so a shape's format can grow without the check changing,
and the check counts the branch's cases without knowing their format.
A citation is an integer or a string such as `"75a"`, both read from
one TOML array.

### The statement's clause numbers are an index kept in the pack

C.7 says the check "reads every case and the statement", and 68 says it
runs from fixed inputs with no source. The statement lives in
agentplot/blueprints, which the gate does not have. The pack keeps
`pack/clauses.toml`: the statement's commit it was read from, and each
section's clause numbers (A.1 to A.12, B.1 to B.6), numbers only
(core.37). The check counts A.1 to A.10 and 71 and accepts any number
the index holds.

*Alternatives:* reading `requirements.md` from a path would need the
blueprints checkout in the gate, breaking 68; copying the statement
into weir would restate it, against core.37.

### Cadence's cases move into one folder each

C.7 fixes one folder per case, and the branch cases already follow it.
Each `pack/cadence/<name>.toml` becomes `pack/cadence/<name>/case.toml`,
text unchanged; cadence's `tests/pack.rs` names a case by its folder.

### `weir conform` seeds and reads the branch through `weir-branch`

For each case, `weir conform` makes a temporary folder, runs
`git init --bare` in it, and seeds branch `inbox` with one `weir-branch`
write of the case's `start/` files (an empty tree when there are none),
so the starting records pass the same `recfix` check every write does
(core.24). After the host's last step it reads the tip with
`weir_branch::read`. The temporary folder is dropped with the case.

### The host command is every word after `conform`

`weir conform [--pack <dir>] <program> [<arg>...]`: options of
`weir conform` come first, and the first word that is not one begins
the host command, run with no shell, each step's flags appended after
its words (C.7). The run step's declaration is written into the
temporary folder: `[adapter.<case name>]`, `command` the absolute path
of the case's script, `config` a JSON Schema that takes any
configuration, `secrets` empty (C.1).

### What is compared, and what is checked for its form

The report is compared field by field as the spec says, refusals by
line and key with the reason holding the case's phrase (15; C.7). The
tip is compared file for file, byte for byte, with the whole tree
counted, so a host that keeps raw material in a record or beside it
fails (17; S15). Three fields of a capture the host wrote are the
host's own (22; core.14) and cannot be fixed by a case: `machine` is
compared with this machine's short hostname, which the `weir` command
line reads and hands in; `taken_at` is checked to fall within the
case's running; and `pointer`, where expected, is checked to name the
record's machine and a path on this machine holding every raw file the
offer named, byte for byte, found by content whatever its name, and the
whole text of a cut excerpt (17, 18, 21). This compares as written
everything a case can fix and holds the rest to what the statement
says of it.

### `weir-report/1` is written once, in `weir-capture`

`Report` gains `to_json`, writing one line naming
`"format": "weir-report/1"`, and a reader the pack uses (core.22). A
host on the crates prints it with one call; a host in another language
writes the same object.

### The pack is built into the `weir` command

`weir-pack` embeds every file under `pack/` at build time with a build
script, so `weir conform <host command>` needs nothing beside the
binary, wherever a host's tests run it. `--pack <dir>` reads another
folder, for weir's own tests of faulty packs and hosts.

*Alternative:* the `include_dir` crate does the same with a new
dependency; a build script needs none.

### Adapter cases come in two kinds

A case with `lines.jsonl` fixes lines and the verdict the checks must
give; `weir-pack`'s tests run it. A case naming `adapter = "folder"` or
`"meeting"` fixes the built-in adapter's input and the lines it must
print, which must also pass the checks; `weir-adapters`' tests run it,
with `weir-pack` a dev-dependency there. The meeting checks take, from
the caller's fixture, each meeting's start by key and the count of
meetings too short, since the lines alone cannot show either (66).

### A host for the pack's own tests

`crates/weir/examples/pack-host.rs` is a host built on the crates:
`take` through an `Inbox` that writes with `weir-branch`, a bank in a
folder its tests name, and `InboxAt::import` for `--import`; it prints
`weir-report/1` last. Each fault the tests need (raw material kept in a
record, a copy of raw material beside it, a cursor kept from an
incomplete run, a wrong refusal reason, no report) is switched on by an
environment variable the test sets. `weir`'s tests run `weir conform`
against it: clean, every case passes; with each fault, the case it
breaks fails, naming its clauses (S15).

### The cases that cover the counted clauses

| clauses | shape | answered by |
|---|---|---|
| 1 to 12 | adapter, host | `weir-pack`'s tests; `weir conform` against the example host |
| 13 to 24, 71 | host | `weir conform` against the example host |
| 25 to 43, 63 | inbox, host (import) | `weir-signals`' tests; `weir conform` |
| 44 to 50 | branch | `weir-branch`'s tests (the shared-branch unit's) |
| 51 to 58 | kinds | `weir-kinds`' tests |
| 59, 60 | cadence | `weir-cadence`'s tests |
| 61, 62, 64 | adapter (built-in) | `weir-adapters`' tests |

71 is cited by an intake case whose offer carries a field the contract
does not name, holding a secret's value, which no record may hold
(core.3, core.6).

## Risks / Trade-offs

- [The shared-branch unit is not in the bolt] → `weir conform` cannot
  seed or read a branch, and the check finds 44 to 50 uncited. It must
  be merged into this unit's branch before code begins, and the first
  task checks it is there.
- [The clause index drifts from the statement] → it names the commit
  it was read from, and the statement's next amendment changes it in
  the change that follows; a cited number the statement dropped then
  fails the check.
- [Embedding `../../pack` from a build script ties the crate to the
  workspace] → the crates are taken at a git tag, which carries the
  whole workspace; publishing to a registry would need the pack moved
  into the crate, a later change's.
- [A host in another language reads `--limit` or the seeded branch
  differently] → the seam is fixed in C.7, and a mismatch fails a case
  naming what differed, which is the pack's purpose.
- [S9 says a `new-territory` move arrives unmoved, 39 says it is a
  join] → the cases follow 39, the clause ruled after; the scenario's
  words are the statement's to change.
