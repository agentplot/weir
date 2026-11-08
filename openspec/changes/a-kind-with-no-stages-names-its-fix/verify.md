# Verify

The work on `change/a-kind-with-no-stages-names-its-fix` at df42ca9,
cut from main at 5562af5, falls short of this change. The repository's
whole checks fail: `kinds:lint` refuses `crates/weir-kinds/tests/pack.rs`
by `clippy::large_enum_variant`. Every requirement and scenario of the
two specs is met by a test that passes, and every task is ticked, but
task 1.6's condition, `kinds:lint` passing, does not hold at df42ca9.
The change is not archived. It goes back to its code stage.

## Checks

- `devenv shell -- check-fast`, run once at df42ca9: exit 1. One task
  failed, `kinds:lint`:

  ```text
  error: large size difference between variants
    --> crates/weir-kinds/tests/pack.rs:46:1
     |
  46 | /  enum Form {
  47 | |      /// A declaration's text.
  48 | |      Declaration { text: String, expect: Expect },
     | |      -------------------------------------------- the second-largest variant contains at least 128 bytes
  49 | |      /// A package and the identity it is held at.
  50 | |/     Package {
  51 | ||         package: PackageCase,
  52 | ||         held: Identity,
  53 | ||         expect: ExpectPackage,
  54 | ||     },
     | ||_____- the largest variant contains at least 344 bytes
  55 | |      /// A lock's text.
  56 | |      Lock { text: String, expect: ExpectLock },
  57 | |  }
     | |__^ the entire enum is at least 344 bytes
     = note: `-D clippy::large-enum-variant` implied by `-D warnings`
  error: could not compile `weir-kinds` (test "pack") due to 1 previous error
  ```

  `ExpectPackage` holds an `ExpectKind`, which f1453ea gives `fix:
  Option<String>` and `max_fixes: Option<u64>`. That takes `Form::Package`
  past clippy's bound against the next variant. The lint denies warnings,
  so the crate's lint task fails (core.39).

- The failure stopped the run. These tasks completed with no failure
  before it stopped: every project's `format`; `root:prose`,
  `root:toolchain`, `root:hooks` and `root:format`; `kinds:typecheck`
  and `kinds:test`, every test binary passing, the 12 new tests among
  them; `pack:typecheck` and `pack:test`; `weir:typecheck`; and the
  `typecheck` and `test` tasks of `adapters`, `cadence`, `capture` and
  `signals`, with `branch:lint` and `branch:typecheck`. These tasks were
  cut off before they finished, so this run gives no outcome for them:
  `weir:lint`, `pack:lint`, `adapters:lint`, `branch:test`,
  `cadence:lint`, `capture:lint`, `signals:lint` and `weir:test`.
  `weir:test` had passed `tests/capture.rs` and `tests/check.rs`, 9 tests
  with `a_kind_that_names_its_fix_is_listed` and
  `a_kind_that_lands_with_no_stages_is_listed`, and was in
  `tests/conform.rs`.
- `openspec validate a-kind-with-no-stages-names-its-fix --strict`:
  valid.

## Tasks

All 9 tasks of groups 1 to 3 are ticked. Each one's tests exist and pass
in the run above, but one condition fails at df42ca9:

- Task 1.6 is true when `devenv shell -- moon run kinds:lint` passes.
  It fails, by the error above, at `tests/pack.rs`, which tasks 3.1 and
  3.2 changed after 1.6 was ticked. Tasks 3.1 and 3.2 name
  `kinds:test` and `pack:test` alone, and both pass. The crate's lint
  covers its tests, so the gate fails on their work.

## Specs

Each scenario is held by the test named, passing in `kinds:test` or
`weir:test` above.

### kinds

- *A kind that lands with no stages may name its fix kind and its
  bound*: `a_kind_that_lands_with_no_stages_names_its_fix`,
  `a_fix_and_its_bound_left_out_are_returned_left_out` and
  `a_bound_without_a_fix` (`tests/read.rs`). `fix` and `max_fixes`
  are in `KIND_FIELDS`, so neither is among `Kind::sections`.
- *A kind's fix is a kind's name and its bound a count*:
  `a_fix_in_every_form_of_a_kinds_name` (`tests/read.rs`) and
  `a_fix_or_a_bound_of_the_wrong_shape_is_refused` (`tests/refuse.rs`),
  the eight shapes of the scenario each named at its field, each `fix`
  string with "names no kind".
- *A declaration is checked whole and refused naming the field*:
  `a_kind_with_stages_that_names_its_fix_is_refused`,
  `a_fix_of_the_wrong_shape_beside_stages` and
  `a_fix_beside_stages_already_refused` (`tests/refuse.rs`). `read_kind`
  judges both fields against stages only where `read_stages` gives a
  non-empty list, and refuses a wrong shape before it, as design 4 says.
  The base's scenarios pass unchanged.
- *weir check reads a declaration*: `a_kind_that_names_its_fix_is_listed`
  (`crates/weir/tests/check.rs`) prints the four lines in order.
  `a_kind_that_lands_with_no_stages_is_listed` is unchanged and passes.
- *The pack holds the kind format's cases*: the three cases under
  `pack/kinds/` cite 52 and 57, 52 and 57, and 52, 89 and 92.
  `a_kind_that_names_its_fix_and_one_with_stages_that_may_not` and
  `a_case_whose_kind_differs_in_its_fix_is_named` (`tests/pack.rs`)
  pass, and `pack:test` reads the citations.
- *A kind's table holding agents and no stages gives settings*:
  `agents_beside_a_fix_with_no_stages` (`tests/agents.rs`).
  `DECLARING_FIELDS` holds `fix` and `max_fixes`.

### kind-packages

- *A package declares exactly one kind and names no other*:
  `a_package_of_a_kind_that_names_its_fix` (`tests/package_dir.rs`)
  reads the package from a folder and from its path and bytes, the two
  equal, `fix` spelling `fix`, `max_fixes` 2, the hash `ContentHash::of`
  its one file.

## What the code stage does

Make `kinds:lint` pass on `crates/weir-kinds/tests/pack.rs`, such as by
boxing the large field of `Form::Package`, as clippy suggests. Then run
`devenv shell -- check-fast` whole, the tasks cut off above among it.
