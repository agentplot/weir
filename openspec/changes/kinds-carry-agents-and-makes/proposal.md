# Proposal

## Why

The statement at blueprints 4d1f88b lets a declaration give settings to
every stage's agents of a kind, and to a kind it does not declare, a
tool's own among them, for every stage or for stages it names (52, 57,
58, C.4). Today `weir-kinds` returns a kind's `agents` table as an
unread section, and refuses a `[kinds.<kind>]` table holding only
`agents` as a kind with no stage, so no reader can take such settings
from a repository's file. The family's runner of places also reads what
a stage makes besides its deliverable from the stage's `makes`, a field
of the kind format by its statement's C.2 and clause 129, which
`weir-kinds` returns today as an unread section.

## What Changes

- A kind's `agents` table reads with the kind: the settings for every
  stage's agents as written, and each `agents.stages.<stage>` table's
  settings for that stage as written (52, C.4). A stage's own `agents`
  read as before.
- A `[kinds.<kind>]` table holding `agents` and nothing that declares a
  kind is returned apart from the kinds, as settings given to that
  name, and never refused as a kind with no stage (57, C.4). Whether a
  stage it names exists, and which declaration it joins, stay the
  reader's (58).
- A stage's `makes`, a list of names, reads with the stage, none when
  left out (53).
- An `agents` table, its `stages` table or a stage's table there, and a
  `makes`, of the wrong shape is refused naming the field (57).
- The pack's kinds case format names the kinds a declaration gives
  settings to, and a case holds the settings-only table and a kind's
  `agents` (52, 57; C.7).

## Capabilities

### New Capabilities

None.

### Modified Capabilities

- `kinds`: a kind's agents table, settings given to a kind declared
  elsewhere, a stage's `makes`, and the pack's case format.

## Impact

- `crates/weir-kinds`: `Kind::agents`, `Stage::makes`,
  `Declaration::settings`, and the new `KindAgents` and `KindSettings`;
  every existing field reads as before. A struct literal of `Kind`,
  `Stage` or `Declaration` outside the crate names the new fields.
- `pack/kinds/`: one case; `tests/pack.rs` reads its `settings`.
