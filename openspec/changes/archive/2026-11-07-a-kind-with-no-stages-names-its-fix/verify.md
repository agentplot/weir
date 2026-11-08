# Verify

The work on `change/a-kind-with-no-stages-names-its-fix` at ba3ac5e,
cut from main at 5562af5, holds this change. The repository's whole
checks pass, every requirement and scenario of the two specs is met by
a test that passes in them, and all 10 tasks are ticked, each one's
condition holding. The change is archived.

## Checks

- `devenv shell -- check-fast`, run once at ba3ac5e: 36 tasks
  completed, none failed. `root:format` and `root:toolchain` ran; the
  other 34 were taken from moon's cache, which keeps a task's outcome
  only when it passed and only for the same inputs. Those 34 are every
  project's `format`, `lint`, `typecheck` and `test`, for `adapters`,
  `branch`, `cadence`, `capture`, `kinds`, `pack`, `signals` and
  `weir`, with `root:prose` and `root:hooks`. `kinds:lint` is among
  them, the crate's tests linted with every warning denied, so
  `tests/pack.rs`, its `Form::Package` holding its expectation boxed,
  passes `clippy::large_enum_variant` (core.39).
- `openspec validate a-kind-with-no-stages-names-its-fix --strict`:
  valid.
- `pack/clauses.toml` is unchanged from 5562af5, as the proposal says,
  and `pack:test` reads the three new cases' citations.

## Tasks

All 10 tasks of groups 1 to 3 are ticked, and each one's condition
holds in the run above:

- 1.1 to 1.5: `kinds:test` passes `tests/read.rs`, `tests/refuse.rs`,
  `tests/agents.rs` and `tests/package_dir.rs`, the tests each task
  names among them, over the fixtures each names.
- 1.6: `kinds:lint` passes, the crate's `missing_docs` among its
  warnings denied. `Kind::fix` and `Kind::max_fixes` each cite 52, 57,
  58 and C.4, `fix` resolved by the reader (core.32). `read_kinds`'
  comment says when either is refused, and the crate's module doc says
  a kind that lands with no stages may name both.
- 2.1: `weir:test` passes `tests/check.rs`, with
  `a_kind_that_names_its_fix_is_listed` and the unchanged
  `a_kind_that_lands_with_no_stages_is_listed`. `listing`'s doc
  comment says what it prints.
- 3.1 and 3.2: `kinds:test` passes `tests/pack.rs`, with
  `a_case_whose_kind_differs_in_its_fix_is_named` and
  `a_kind_that_names_its_fix_and_one_with_stages_that_may_not`, and
  `pack:test` passes `the_pack`.
- 3.3: `kinds:lint` passes and `check-fast` passes whole.

## Specs

Each scenario is held by the test named.

### kinds

- *A kind that lands with no stages may name its fix kind and its
  bound*: `a_kind_that_lands_with_no_stages_names_its_fix`,
  `a_fix_and_its_bound_left_out_are_returned_left_out` and
  `a_bound_without_a_fix` (`tests/read.rs`). `read_kind` reads both into
  `Kind::fix` and `Kind::max_fixes`, each `None` when left out, and
  `KIND_FIELDS` names both, so neither is among `Kind::sections`.
- *A kind's fix is a kind's name and its bound a count*:
  `a_fix_in_every_form_of_a_kinds_name` (`tests/read.rs`), each form's
  spelling, source and version, and
  `a_fix_or_a_bound_of_the_wrong_shape_is_refused` (`tests/refuse.rs`),
  the eight shapes of the scenario each named at its field, each `fix`
  string with "names no kind".
- *A declaration is checked whole and refused naming the field*:
  `a_kind_with_stages_that_names_its_fix_is_refused`, `bump` landing
  and `tidy` not, each `fix` before its `max_fixes`;
  `a_fix_of_the_wrong_shape_beside_stages`, named once by its shape;
  and `a_fix_beside_stages_already_refused`, each kind's `stages` named
  once and neither field (`tests/refuse.rs`). `read_kind` reads the
  stages before `fix` and `fix` before `max_fixes`, and judges both
  against stages only where `read_stages` gives one or more. The base's
  scenarios pass unchanged.
- *weir check reads a declaration*: `a_kind_that_names_its_fix_is_listed`
  (`crates/weir/tests/check.rs`) prints the four lines in order and
  nothing on standard error. `a_kind_that_lands_with_no_stages_is_listed`
  is unchanged and passes, neither new line printed.
- *The pack holds the kind format's cases*: the three cases under
  `pack/kinds/` cite 52 and 57, 52 and 57, and 52, 89 and 92, each
  comment citing the clauses as amended at blueprints 31f9f1c. The
  package case's hash is `ContentHash::of` its one file, asserted by
  `a_kind_that_names_its_fix_and_one_with_stages_that_may_not`.
  `a_case_whose_kind_differs_in_its_fix_is_named` gives a difference
  naming the case and both values, for `fix` and for `max_fixes`.
- *A kind's table holding agents and no stages gives settings*:
  `agents_beside_a_fix_with_no_stages` (`tests/agents.rs`), both tables
  refused naming their `stages` and no settings given. `fix` and
  `max_fixes` are in `DECLARING_FIELDS`.

### kind-packages

- *A package declares exactly one kind and names no other*:
  `a_package_of_a_kind_that_names_its_fix` (`tests/package_dir.rs`),
  the package read from a folder and from its path and bytes the same,
  `fix` spelling `fix`, `max_fixes` 2, the identity `acme/train@1` and
  the hash `ContentHash::of` its one `kind.toml`. The base's scenarios
  pass unchanged.
