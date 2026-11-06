# Design

## Context

`weir-kinds` at main, b431004, reads a kind's table in one of two ways
(`src/kinds.rs`). A table that holds `agents` and none of `version`,
`branch`, `starts_unready`, `lands`, `stages` and `elaboration` gives
settings (`gives_settings`). Any other table is a kind, read by
`read_kind`. `read_stages` refuses that kind's `stages` when it is left
out ("is missing, and a kind has at least one stage") or empty ("is
empty, and a kind has at least one stage"), whatever its `lands`. The
pack's case `a-kind-with-no-stage` declares `lands = true` with `stages`
left out and expects a reason holding `a kind has at least one stage`. `weir check` lists each kind as
`<name>@<version>` with a line per stage.

The statement as this design reads it is at blueprints 51b6316, the
user's approved plan for flume's settles design of 2026-11-06. It reads
51, 52 and 57 as amended there, C.4 with its `[kinds.train]` example,
and 56, 58 and S12 as they stand. A simple kind is one or more stages,
or none where it lands, and a kind that lands with no stages runs no
agent (51). Only a kind that lands with stages names where a failed
submission sends a place back (52). A simple kind with no stage that
does not land is refused naming the field. A kind that lands with no
stages declares them as none, `stages = []`. A kind's table that holds
settings for its agents and leaves its stages out declares no kind (57).

flume reads such a kind as a base place's, knowing it by
`kind.lands && kind.stages.is_empty()` and by nothing else (flume 202
to 206; flume's design for the change
every-place-waiting-on-the-merge-queue-is-settled-at-every-depth,
decisions 4 and 5 and its follow-up W1, this change). Its kinds profile
refuses a kind that does not land and declares none "in weir's words",
so the reasons below are what flume's operator reads.

Rules this design relies on, all in `docs/architecture/core.md`, the one
page whose area the change touches:

- core.2: no tool named. The fixtures name a kind `train`, as C.4 does,
  and nothing names a host or its branches.
- core.30: one reader of kinds.
- core.31, as amended on this branch from 51, 52 and 57 at 51b6316: a
  declaration checked whole.
- core.32: what a kind's places do is its reader's.
- core.35: each case cites its clauses.
- core.37: a clause changed in the statement first.
- core.39 and core.40: the projects' checks.
- core.41: a settings table.

The design departs from none.

## Goals / Non-Goals

**Goals:**

- `stages = []` read on a kind that lands, refused naming the kind on
  one that does not, and a table that leaves `stages` out read as at
  b431004.
- Every reader in weir that says what a kind holds says it too. That is
  the crate's docs, `weir check` and the pack's kinds cases.

**Non-Goals:**

- What a kind with no stages does: a base place, its branch, its
  submission. That is flume's (51, 58; core.4, core.32).
- Statechart kinds (94, 95), whose table holds `regions` and no
  `stages`, and the clauses after 93 in the pack's clause index. Each
  belongs to a change of its own.
- A release of weir. weir publishes none; a reader pins a revision of
  its main.

## Decisions

### 1. A kind with no stages is a `Kind` whose `stages` is empty

`read_stages` takes whether the kind lands and reads `stages = []` as no
stages where it does. `Kind` keeps its fields, and `Kind::stages` is
empty for such a kind, which is how flume tells a base place (flume
design 4). No type, flag or variant names the shape: 51 makes it a
simple kind like any other, and a name of it in weir would be a host's
word in weir (core.2).

The way not taken is a variant of `Kind`, or a field such as `base`.
Every reader would then have to match on it, and the statement names no
such field.

### 2. `stages` left out of a kind's table is refused, whether it lands or not

57 has a kind that lands with no stages declare them as none,
`stages = []`, and C.4's example says so: "none, declared so". A table
that declares a kind by any field the format names, `lands` among them,
and leaves `stages` out has declared no stages. It is refused naming
its `stages`, as at b431004. The settings rule is unchanged: a table
holding `agents` and nothing else the format names gives settings
(core.41).

The way not taken is reading `lands = true` with `stages` left out as a
kind with none. Then a table that leaves `stages` out would mean three
things, by its `agents` and its `lands`. A landing kind whose stages
were forgotten would also become a kind with no agent, read without a
word. Loosening the rule later is cheap; taking it back from readers
who rely on it is not.

### 3. The reasons say what a kind declares

Each refusal names the field `kinds.<kind>.stages`, and so the kind:

| `stages` | `lands` | reason |
|---|---|---|
| left out | any | "is missing, and a kind declares its stages: one or more, or none as `stages = []` where it lands" |
| `[]` | false or left out | "is empty, and a kind that does not land has at least one stage" |
| `[]` | true | read: a kind with no stages |

Neither reason says "a kind has at least one stage", which no longer
holds. The tests that match that phrase, and the pack case that holds
it, take the new ones. A reason that changes breaks no reader:
`Problem::reason` is words for a person, and a reader matches the field.

### 4. An empty `stages` beside a `lands` already refused

Where `lands` is not true or false, it is refused naming `lands`, and
`stages = []` is judged against no `lands`: it is not refused a second
time. A stage's `repeat` beside a refused end is judged the same way
(kinds spec, *A repeat the format does not allow is refused*). A
`stages` left out is refused whatever `lands` holds, by decision 2.

### 5. A kind with no stages holds what any kind holds

A kind that lands with no stages reads its `version`, `branch`,
`starts_unready`, `agents` and every section as any kind does, each
returned as written and none refused for the kind having no stages. 57
lists no such refusal, so adding one would depart from the statement
(core.31, core.37). Whether `starts_unready` or an `agents` table means
anything for a kind that runs no agent is its reader's (51, 58;
core.32). Such a kind names no stage a failed submission sends a place
back to, as 57 says, by construction. `on_fail` and `max_send_backs`
are fields of a stage, and on the kind's own table either is a key the
format does not name there, returned unread (56).

A table holding `agents`, `lands = true` and `stages = []` is a kind
with no stages and its `agents` table, not settings, since it holds
`stages` (core.41).

The way not taken is refusing `agents` or `starts_unready` on a kind
with no stages. That refusal is the reader's to make where it holds one.

### 6. `weir check` says a kind lands with no stages

`weir check` lists such a kind as `<name>@<version>` followed by the
line `  lands with no stages`. The way not taken is the name alone,
which reads as a kind whose stages the listing dropped.

### 7. The pack's cases

The case format is unchanged. A kind's expected `stages` may be `[]`,
which `Vec<ExpectStage>` reads already.

- `a-kind-with-no-stage` declares `lands = true` with `stages` left
  out. That is no longer S12's declaration with no stage, so the case
  is renamed `a-kind-that-lands-with-its-stages-left-out`. It keeps
  its declaration and expects the reason of decision 3, `a kind
  declares its stages` (57).
- `a-kind-that-lands-with-no-stages` (new) declares `[kinds.train]`
  with `branch = "train"`, `lands = true` and `stages = []`, as C.4's
  example does, and expects `train` at version 1 with no stages (51, 52,
  57).
- `a-kind-that-does-not-land-with-no-stages` (new) declares `stages =
  []` on a kind with `lands` left out, and expects it refused naming
  `kinds.<kind>.stages` with `a kind that does not land has at least
  one stage` (51, 57). This is S12's declaration with no stage, and
  `the_cases_of_s10_s11_and_s12` in `crates/weir-kinds/tests/pack.rs`
  names it in place of `a-kind-with-no-stage`.

The three together pin the shape. A reader that reads `stages = []`
whatever `lands` holds fails the third. A reader that reads `stages`
left out as none fails the first. Each case's comment cites 51 and 57
as amended at blueprints 51b6316.

### 8. The clause index stays at a11f6d5

`pack/clauses.toml` holds the statement's clause numbers, each under
its section, and moves when a clause is added, removed or renumbered.
51b6316 amends 51, 52 and 57 and C.4's example, and adds, removes or
renumbers no clause of weir's. Moving the index to 51b6316 would take
in 94 to 99, whose cases are the statechart change's.

## Risks / Trade-offs

- A reader of weir-kinds that indexes a kind's first stage, or assumes
  at least one, now meets a kind with none wherever a declaration it
  reads holds one. weir's own readers, `weir check` and `read-package`,
  are checked here. flume's reader moves with its pin (flume's design,
  decision 6).
- A declaration a reader wrote expecting `stages = []` to be refused on
  a landing kind now reads. A reader that wants to refuse it still can,
  by its own rule (58).

## Migration Plan

flume moves its `weir-kinds` pin in `Cargo.toml` to the revision of
weir's main that lands this change. weir publishes no release and
tags none, so the pin is that revision. Nothing else moves.
