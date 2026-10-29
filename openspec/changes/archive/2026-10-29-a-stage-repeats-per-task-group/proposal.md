# Proposal

## Why

The statement at blueprints d75cf53 lets a stage say it repeats per
task group, worked one group of its change's tasks at a time, and only
a stage whose deliverable is tasks ticked may (53, 57, C.4). flume's
record 0019 builds on it: its change@2's code stage repeats (S10).
Today `weir-kinds` returns a stage's `repeat` in the stage's unread
sections, so a runner of places could read it only by defining a field
of the format itself.

## What Changes

- A stage's `repeat` reads with the stage: `per-task-group`, the one
  value the format gives it, or none when left out (53, C.4).
- `repeat` with any other value, or of any other shape, is refused
  naming the field; so is `repeat` on a conversation or on a stage
  whose deliverable is not `tasks` (57).
- The pack's kinds case format names a stage's `repeat`, and the pack
  holds flume's change@2 read with its code stage repeating (S10) and a
  `repeat` on a stage of commits refused (57; C.7).

## Capabilities

### New Capabilities

None.

### Modified Capabilities

- `kinds`: a stage's `repeat`, its refusals, and the pack's case format
  and cases for it.

## Impact

- `crates/weir-kinds`: `Stage::repeat` and the new `Repeat`; every
  existing field reads as before. A struct literal of `Stage` outside
  the crate names the new field.
- `pack/kinds/`: two cases; `tests/pack.rs` reads a stage's `repeat`.
