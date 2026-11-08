# Proposal

## Why

flume's record 0056, accepted by the user on 2026-11-07, has flume
launch a base place's fix when a settle forecasts its branch in conflict
with its base or finds its submission failed. A base place is a place
of a kind that lands with no stages, so it has no stage whose `on_fail`
and `max_send_backs` could name its fix and bound. The record's
amendment, settled by agentplot-design within the user's decision, has
the base place's kind name them. The statement at blueprints 31f9f1c
says so for weir:

- 52 lets a kind that lands with no stages name the kind its fixes run
  as, by a name its reader resolves, and how many fixes it runs before
  its place waits on its caller.
- 57 refuses either on a kind with stages, naming the field.
- 89 lets a package name that one kind beside its own, a name and no
  part of the package.
- C.4 spells the two fields `fix` and `max_fixes` on `[kinds.train]`,
  gives neither a default, and returns `fix` as written.

flume defines no field of the format (flume core.4) and reads kinds only
through weir-kinds (core.30). weir-kinds at main, 5562af5, names neither
field. It returns `fix` and `max_fixes` among a kind's unread sections,
and reads a table holding `agents` and `fix` as settings. flume's
follow-up unit a-base-places-fix-kind-is-read-from-weirs-format waits on
this change. Until it lands, a kind in weir's format names no fix, and
its base place waits on its caller (flume 203).

## What Changes

- `read-kinds` reads `fix` and `max_fixes` on a kind's table into the
  kind. `fix` is a kind's name in any of its four forms, returned as
  written. `max_fixes` is a whole number of at least 0. Each is left out
  when left out, and a value of the wrong shape is refused naming the
  field (52, 57, C.4).
- A kind with stages that names either is refused naming the field,
  whether it lands or not (57).
- Both are fields the format names, so neither is returned among a
  kind's sections any more. A table holding `agents` and either reads
  as a kind, not as settings (core.41).
- `read-package` reads a package whose kind names its fix kind. That
  kind is a name, not a part, and the package need not hold it (89).
- `weir check` lists a kind's fix kind and bound under its line saying
  it lands with no stages.
- The pack's kinds cases expect a kind's `fix` and `max_fixes`. Three
  cases join them: a kind that lands with no stages naming both, a kind
  with stages naming both and refused, and a package of a kind naming
  both.
- core.31, with the details of core.32, core.41 and core.43, says it as
  a rule. These edits are already on this branch.

## Capabilities

### New Capabilities

None.

### Modified Capabilities

- `kinds`: a kind that lands with no stages may name its fix kind and
  its bound. The refusals of either, the settings rule, `weir check`'s
  listing and the pack's kinds cases follow.
- `kind-packages`: a package whose kind names its fix kind reads, the
  name no part of it.

## Impact

- `crates/weir-kinds`: `Kind` gains two public fields, `fix:
  Option<NamedKind>` and `max_fixes: Option<u64>`, read by `read_kinds`
  (`src/kinds.rs`), with the docs of the crate and of `read_kinds`. A
  caller that builds a `Kind` by a struct literal must give both. A
  reader that took `fix` or `max_fixes` from `Kind::sections` finds
  them on the kind instead. A declaration with a table holding `agents`
  and `fix` and no `stages` was read as settings and is now refused,
  naming the kind's `stages`.
- `crates/weir`: `weir check`'s listing, and its tests.
- `pack/kinds/`: the case format's expected kind gains `fix` and
  `max_fixes`, read by `crates/weir-kinds/tests/pack.rs`, and three
  cases join the pack.
- `docs/architecture/core.md`: core.31 and the details of core.32,
  core.41 and core.43, as amended, already on this branch.
- `pack/clauses.toml` is unchanged. 31f9f1c amends 52, 57, 89 and C.4,
  and adds, removes or renumbers no clause of weir's.
- flume moves its pin of weir-kinds to the revision of weir's main that
  lands this, in its follow-up unit
  a-base-places-fix-kind-is-read-from-weirs-format. weir publishes no
  release, so the pin is that revision.
