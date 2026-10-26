# Proposal

## Why

The family's hosts need three ways in that no other tool owns: a file
dropped in a folder, a meeting transcript saved as a file, and the
captures crew already holds in its signals folders, which predate any
inbox. Section 9 puts all three in weir, so millpond and later Flywheel
run one copy of each (plan step F3,
`agentplot/blueprints:design/capture/plan.md`). The folder drop and the
meeting file are adapters like any other and speak the contract (61,
62, 64). The import is not an adapter: an imported capture carries a
pointer, several signals and their moves, which no offer carries, so it
prints import lines of its own shape for a host's import operation
(63, 75a; madswan-design's ruling, blueprints main f4aabd9).

## What Changes

- **The import line.** `weir-capture` gains the fourth line C.1
  shows, `{"import": {...}}`: the capture as an offer's object without
  `raw` and `excerpt`, the pointer as the body gave it, `came_in`
  `import:<body>`, the signals numbered from 1 with every field of a
  signal, and the moves by signal number, a move of any word carried as
  it stands. It is read, refused naming the field, and written, and
  `take` refuses it as a line that is not an adapter's (63, 64, 75a;
  C.1).
- **The folder drop.** `weir capture folder <dir>` offers one capture
  per file in the folder, keyed `folder/<folder name>/<sha256 of the
  file>` with the capture crate's hash, so a file dropped twice, or
  under another name, is one capture. Its event time is the date the
  file's name begins with, or else its last write; its source is
  `folder`, the file is copied whole into the raw place as its raw
  material, and it carries no excerpt (61; C.6; S14).
- **The meeting file.** `weir capture meeting <file>` offers one
  capture for the file, keyed `meeting/file/<sha256 of the file>`, its
  event time the meeting's start where the file states one, else as the
  folder drop's; its source is `meeting`, the transcript is its raw
  material, and it carries no excerpt (62; C.6; section 10, question 4).
- **The signals-folder import.** `weir import signals <dir>` reads
  crew's folders, a `capture.md` and one `NN-<slug>.md` per signal for
  each capture, with `moves.rec`, and prints one import line per
  capture. A release capture keeps its key `release/<name>/<version>`
  and every other takes its folder's name; the pointer is the one crew
  wrote; signals and moves are as read. crew's `new-territory` is
  carried as a join into a proposed item named for its target, and a
  word not of the six as it stands, for the import operation to leave
  unmoved. Nothing is copied from raw material, and reading one folder
  twice prints the same lines (37 to 39, 63; C.6; S8, S9; core.25).
- **The `weir` command.** A new crate, `weir`, holds the command line
  that runs the three: the two adapters print contract lines and exit
  as C.1 says, and the import prints import lines and names on
  standard error each capture it could not read and each move it did
  not carry (section 9).
- **Two crates and their moon projects.** `weir-adapters` holds the
  folder drop, the meeting file and the reader of crew's folders;
  `weir` holds the command. Each joins the workspace's `members` and is
  a moon project with format, lint, typecheck and test tasks (core.39).

## Capabilities

### New Capabilities

- `import-lines`: the import line's shape, read and written, and what
  refuses one.
- `folder-drop`: the folder drop's offers, keys, event times and raw
  material.
- `meeting-file`: the meeting file's offer, key, event time and raw
  material.
- `signals-folder-import`: crew's folders read into import lines, their
  keys, pointers, signals and moves.
- `weir-command`: `weir capture folder`, `weir capture meeting` and
  `weir import signals`, their arguments, output and exit status.

### Modified Capabilities

- `adapter-contract`: a line holding `import` is refused by the line
  reader `take` uses, named as an import line read by the import
  operation alone (64).

## Impact

- New crates `crates/weir-adapters` (moon project `adapters`) and
  `crates/weir` (moon project `weir`, the `weir` binary), joining the
  workspace's `members`; `Cargo.toml`'s comment on the planned crates
  names the adapters crate landed, and `Cargo.lock` gains both.
- `weir-capture` gains the import line's type beside its other lines,
  and its line reader names an import line as one `take` does not read.
- The inbox's records and layout unit of this bolt writes import lines
  into an inbox through its import operation (75a), which takes its
  own `ImportedCapture`. One conversion from this change's `ImportLine`
  hands a line over; it lives in `weir-signals` and is written by
  whichever of the two units is built second, so this change edits
  `crates/weir-signals` only when that crate is already on the bolt.
- millpond's `millpond capture folder|meeting` and `millpond import`
  (plan step P3) call these.

## Touches

```
Cargo.toml
Cargo.lock
.moon/workspace.yml
crates/weir-capture/**
crates/weir-adapters/**
crates/weir/**
crates/weir-signals/src/**
```
