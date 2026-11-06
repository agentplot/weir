# Proposal

## Why

The statement at blueprints 51b6316 lets a kind that lands declare no
stages, `stages = []`: such a kind runs no agent, and what its places
do is its reader's rule (51, 52 and 57 as amended there, and C.4's
example beside them). flume reads a kind of that shape as a base place's,
a branch that work lands on and that itself lands, with no worktree and
no agent (flume 202 to 206). flume reads kinds only through weir-kinds
(core.30), and weir-kinds at main, b431004, refuses every simple kind
with no stage, `stages = []` among them. flume's follow-up W1 waits on
this change, and flume moves its pin of weir-kinds to the revision that
lands it.

## What Changes

- `read-kinds` reads `stages = []` on a kind that lands as a kind with
  no stages (51, 57).
- `stages = []` on a kind that does not land, its `lands` false or left
  out, stays refused naming the kind's `stages`, the reason saying a
  kind that does not land has at least one stage (57, S12).
- A kind's table that leaves `stages` out reads as it does at
  b431004: as settings where it holds `agents` and nothing else the
  format names, and otherwise refused naming its `stages`, whether it
  lands or not. Only the reason changes, to say that a kind declares its
  stages, one or more, or none as `stages = []` where it lands (57,
  C.4). No refusal says "a kind has at least one stage" any more.
- `weir check` lists a kind that lands with no stages with a line
  saying so.
- The kinds spec, the crate's docs and the pack's kinds cases say what
  a kind holds as amended, and core.31 says it as a rule.

## Capabilities

### New Capabilities

None.

### Modified Capabilities

- `kinds`: a kind that lands may declare `stages = []`; the refusals of
  a kind with no stage, their reasons, `weir check`'s listing and the
  pack's kinds cases follow.
- `kind-packages`: a package whose kind lands with no stages reads, its
  `kind.toml` the one file it holds.

## Impact

- `crates/weir-kinds`: `read_kinds` (`src/kinds.rs`), and the docs of
  `Kind::stages` and of the crate. No public type changes. `Kind::stages`
  may now be empty where `Kind::lands` is true, so a reader that takes a
  kind's first stage must handle none.
- `crates/weir`: `weir check`'s listing, and its tests' reasons.
- `pack/kinds/`: the case of a kind with no stage is renamed for what
  it holds, a kind that lands with its stages left out, and takes the
  new reason. Two cases join it: a kind that lands with `stages = []`,
  and S12's kind that does not land with none.
- `docs/architecture/core.md`: core.31 as amended, already on this
  branch.
- `pack/clauses.toml` is unchanged. 51b6316 amends 51, 52 and 57 and
  adds, removes or renumbers no clause of weir's.
- flume moves its pin of weir-kinds to the revision of weir's main that
  lands this. weir publishes no release, so the pin is that revision.
