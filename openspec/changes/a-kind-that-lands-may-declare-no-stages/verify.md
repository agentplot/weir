# Verify

The work on `change/a-kind-that-lands-may-declare-no-stages` at 80c7f68,
cut from main at b431004, holds against this change. Every requirement
and scenario of its two specs is met by a test that passes. Every task
is ticked, and every check the tasks name passes uncached. The change is
archived with this report.

## Checks

- `devenv shell -- moon run kinds:format kinds:lint kinds:typecheck
  kinds:test weir:format weir:lint weir:typecheck weir:test pack:test
  root:format --force`: 10 tasks completed, none from cache, none
  failed, 254 tests passed (task 4.1).
- `devenv shell -- check-fast`: 36 tasks completed, none failed
  (core.39).
- `openspec validate a-kind-that-lands-may-declare-no-stages --strict`:
  valid.
- Each MODIFIED requirement of the two deltas was compared with the
  requirement of the same name in `openspec/specs/`. Every scenario of
  the base is kept, and the body text changes only where the change
  says. *A declaration is checked whole* keeps each refusal of the base
  as a list item, *any kind has no stage* split into its two amended
  refusals. *The pack holds the kind format's cases* keeps each case the
  base lists and adds the two of 51 and 57.

## Tasks

All 10 tasks of groups 1 to 4 are ticked, and each one's condition
holds: the tests it names exist and pass with the checks above.

Task 1.2 asks that every other existing test stay unchanged. Besides
the reasons task 1.2 and task 2.1 name, one existing test changes, and
the spec requires it. `a_kind_with_no_stage_is_refused` read
`no-stages.toml`, which held `bare`, stages left out, and `empty`,
`stages = []`, both refused for one reason. The kinds spec's amended
*A kind with no stage is refused* is about a kind that does not land
with `stages = []`, so the fixture now holds `empty` and `not-landing`,
and the test compares each problem's reason as well as its field.
`bare` moves to the new `stages-left-out.toml` beside `landing`. Each
kind the old fixture held is still tested.

## Specs

### kinds

- *A declaration reads into its kinds*: `a_kind_that_lands_with_no_stages`
  (`tests/read.rs`, over `lands-with-no-stages.toml`) reads `train` at
  version 1, branch `train`, landing, no stages, no section unread.
  `a_kind_with_no_stages_holds_what_any_kind_holds` reads version 2,
  `starts_unready`, the `agents` table and `owner` unread. The three
  scenarios of the base pass unchanged.
- *A declaration is checked whole and refused naming the field*:
  `a_kind_with_no_stage_is_refused` (`tests/refuse.rs`) compares the
  whole refusal of `empty`, `lands` left out, and `not-landing`,
  `lands = false`, each at `kinds.<kind>.stages` with "is empty, and a
  kind that does not land has at least one stage".
  `a_kinds_stages_left_out_are_refused_whether_it_lands_or_not` does the
  same for `bare` and `landing` with "is missing, and a kind declares its
  stages: one or more, or none as `stages = []` where it lands".
  `empty_stages_beside_a_lands_of_the_wrong_shape` gives the one field
  `kinds.train.lands`. *Every failing field is named* holds through
  `a_refusal_reads_one_problem_a_line` over `two-faults.toml` with the
  new reason.
- *A kind's table holding agents and no stages gives settings*:
  `a_table_with_neither_agents_nor_stages_is_still_a_kind_with_no_stage`
  (`tests/agents.rs`) names `kinds.tidy.stages` and now checks the new
  reason too. `agents_beside_stages_declared_as_none` reads `train` with
  no stages, its `agents` table as written and no settings.
- *weir check reads a declaration*: `a_kind_that_lands_with_no_stages_is_listed`
  (`crates/weir/tests/check.rs`) prints exactly `train@1` and
  `  lands with no stages`, nothing on standard error, exit 0.
  `every_problem_of_the_kinds_is_named` expects `kinds.bare.stages` with
  the reason of task 1.2. `listing` adds the line only where a kind's
  stages are empty, which `read_kinds` returns only for a kind that
  lands.
- *The pack holds the kind format's cases*: the three cases of design 7
  are under `pack/kinds/`, each citing 51 and 57, the landing one 52 as
  well. `a_kind_that_lands_with_no_stages_and_its_two_neighbours`
  (`tests/pack.rs`) checks each declaration's shape and expected answer,
  and that the crate gives it. `the_cases_of_s10_s11_and_s12` names
  `a-kind-that-does-not-land-with-no-stages` as S12's. No file outside
  the change's own documents still names `a-kind-with-no-stage`.

### kind-packages

- *read-package reads a package at the identity it is held at*:
  `a_package_of_a_kind_that_lands_with_no_stages`
  (`tests/package_dir.rs`) lays out a folder holding only `kind.toml`,
  reads it through `read_package_dir` and the same path and bytes
  through `read_package`, and gets one answer: `train` landing with no
  stages, held at `acme/train@1`, its hash `ContentHash::of` that one
  file.

## The code

`read_kind` reads `lands` before the stages and hands `read_stages`
`Some(lands)` where it read and `None` where it was refused. The
problems are pushed in the same order as at b431004. `read_stages`
refuses `stages` left out whatever `lands` is, and `stages = []` only
where `lands` read false or was left out (design 2 to 4). `Kind` keeps
its fields, and no type or flag names the shape (design 1). No code in
`crates/*/src` indexes a kind's first stage, so `weir check` and
`read-package` are the only readers in weir that meet a kind with none,
and both are tested here.

## The statement

`design/weir/requirements.md` at blueprints 51b6316, on blueprints'
`main`, amends 51, 52 and 57 and adds C.4's `[kinds.train]` example as
the change reads them. 51b6316 adds, removes and renumbers no clause of
weir's, so `pack/clauses.toml` stays at a11f6d5 (design 8).

S12 is not amended at 51b6316, and still reads "a declaration with no
stage is refused the same way". The change reads it as a kind that does
not land with no stage, which 57 as amended refuses. The pack's case
`a-kind-that-does-not-land-with-no-stages` holds that reading. A
declaration whose one kind lands with `stages = []` now reads, so S12's
words are wider than 57's. This does not hold the archive, since the
work follows 57. It is the statement's to say in S12 whether a kind that
lands is meant.

## Rules

core.31 on this branch says what the code does: no stage refused on a
kind that does not land, `stages` left out refused on a table that
declares a kind, and `stages = []` read on a kind that lands. core.41's
settings rule is unchanged, and a table holding `stages` is never
settings. No module of `crates/weir-kinds/src` or `crates/weir/src` the
change touches names a tool of the family (core.2). The new fixtures
and cases name a kind `train`, as C.4 does, and no host.
