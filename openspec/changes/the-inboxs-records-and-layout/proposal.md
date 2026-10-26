# Proposal

## Why

The capture crate takes an adapter's lines into a capture and its own
signal, but nothing yet says where those records lie in an inbox, what
the inbox's other records are (a reading, a move, a batch, an item, its
brief and its sends), or which writes an inbox refuses. Until that is
fixed once, millpond and Flywheel would each lay out an inbox and check
its writes their own way, and an inbox one wrote the other could not
read (40, 42, 43). This change builds `weir-signals`, the crate section
9 names for it, so the hosts, the built-in adapters' import and the
write protocol all build on one copy (plan step F2,
`agentplot/blueprints:design/capture/plan.md`).

## What Changes

- **Where every record lies.** An inbox is the `inbox/` tree of one tip:
  each capture at `captures/<its key's segments>/` with `capture.rec`
  and its `signals.rec` beside it; `moves/moves.rec` and
  `moves/batches/<batch>.rec`; `items/<name>/` with `item.rec`,
  `brief.md` and `sends.rec` (42; C.2). The crate reads a tip's files
  into one inbox and refuses a file the layout does not place, a capture
  that lies anywhere but where its key fixes, or a record in a format
  or major it does not read (40, 43).
- **The records.** `weir-signal/1`, `weir-move/1`, `weir-batch/1`,
  `weir-item/1` and `weir-send/1` in recutils, each naming its format
  first, and a brief naming `weir-brief/1` on its first line (25, 27,
  30, 31, 36, 40; C.2). A reader of a later major reads every earlier
  one, and the crate writes only the majors it reads.
- **Nothing written changes.** A write only adds files or adds records
  to the end of `moves.rec`, a batch's file and `sends.rec`; a capture,
  a signal, an item's record and every record already in an appended
  file stay as they were, and `brief.md` alone is rewritten (41).
- **Readings.** A capture's own signal from the capture crate, or a
  reader's signals, numbered from 1 as `<key>#<n>`; one reading per
  capture, written whole or refused whole naming each signal not taken;
  a signal whose excerpt stands neither in the raw material the writer
  hands in nor as the own excerpt as cut is refused (25 to 28).
- **Moves.** A signal has at most one standing move; a second is
  refused naming the one that stands; only a replacement naming who
  made it and the move it replaces changes one. A move is one of drop,
  answered, challenge, join, attach and route, each with its
  consequence, or is refused (29 to 31).
- **Batches.** A batch's moves lie in its own file and do not stand
  while it is proposed; it stands whole, its moves added to
  `moves.rec`, or is refused whole naming each move that could not be
  taken (32). No operation deletes an unmoved signal (33).
- **Items, briefs and sends.** An item's signals are read from the
  standing joins and attaches that name it and its weight computed from
  them; its
  name is one the inbox has never had and holds no `/`; it is made by a
  join or from an accepted conclusion; its brief is the text curation or
  the operator hands; a send keeps the brief as handed, and its outcome
  is added beside it (34 to 36).
- **The import operation.** A host's import (75a) writes the captures
  an import reads from a body before its inbox existed, one per import
  line, with the pointer, signals and moves each was read with: a line
  whose capture names no key, source or event time refused and
  reported; each held by key, a key the inbox holds writing nothing, so
  a second import of one body writes nothing; no raw material
  banked, no pointer of the host's own and no excerpt checked; a move
  not of the six, or without its consequence, not taken and its signal
  left unmoved, named in the import's report; a join making or growing a
  proposed item with no brief (37 to 39, 63; S8, S9). Reading crew's
  folders into import lines, and the import line's own type in
  `weir-capture`, are the built-in adapters unit's (madswan-design's
  ruling, blueprints main f4aabd9).
- **The checks.** Every operation forms its write from the inbox read at
  one tip and checks it there, so the write protocol can form it again
  on a new tip and every check is made again (46): no held key written
  again, no second reading, no second standing move, no used item or
  batch name, and every record valid where it lies.
- The crate is moon project `signals`, with format, lint, typecheck and
  test tasks, its tests holding every file it forms to `recfix`
  (core.24, core.39).

## Capabilities

### New Capabilities

- `inbox-layout`: where each record of an inbox lies, how an inbox is
  read whole from one tip, the formats and majors its records name, and
  the rule that a write only adds.
- `inbox-signals`: a capture's signals, its one reading, and the moves
  and batches curation makes on them.
- `inbox-items`: items, their signals and weight, their briefs and their
  sends with what became of each.
- `inbox-import`: a host's import operation, carrying captures read
  before their inbox existed into it, with their pointers, signals and
  moves.

### Modified Capabilities

None.

## Impact

- New crate `crates/weir-signals`, depending on `weir-capture` (its
  key, capture record, recutils, own signal and cut) and on `time`,
  joining the workspace's `members` with moon project `signals` in
  `.moon/workspace.yml`.
- `Cargo.toml`'s comment on the planned crates names the signals crate
  landed, and `Cargo.lock` gains the crate.
- No change to `weir-capture`: the inbox writes the `Taken` it hands
  over as `capture.rec` and `signals.rec`.
- Later work builds on it: the built-in adapters' import line, read in
  `weir-capture`, is handed to the import operation as the imported
  capture this crate defines; the write protocol (`weir-branch`) reads a tip,
  calls these operations on it and lands the write; the conformance
  pack's host cases expect these records.

## Touches

```
Cargo.toml
Cargo.lock
crates/weir-signals/**
.moon/workspace.yml
```
