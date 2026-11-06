# Tasks

Each group is one session's work for an agent that reads this change
(proposal, design, specs) and its group, and builds on the groups
before it. The statement is read at blueprints 51b6316: 51, 52 and 57
as amended there, and C.4's `[kinds.train]` example. core.31, as
amended from those clauses, is already on this branch. Tests read
fixtures under `crates/weir-kinds/tests/fixtures/` and the pack's cases,
and reach no file outside a temporary folder of their own. No fixture,
case or doc names a host, a tool's branch or a use of a kind with no
stages (core.2).

## 1. A kind that lands may declare no stages

- [x] 1.1 Have `read_stages` take whether the kind's `lands` read true, read false or left out, or was refused, and read `stages = []` as no stages on a kind that lands. On a kind whose `lands` is false or left out, refuse it naming `kinds.<kind>.stages` with "is empty, and a kind that does not land has at least one stage". Where `lands` was refused, judge it against no `lands`, as `read_repeat` judges a `repeat` against a refused end (design 1, 3, 4; 51, 57; core.31). True when `devenv shell -- moon run kinds:test` passes tests over fixtures holding the kinds spec's *A kind that lands with no stages*, *A kind with no stage is refused* and *Empty stages beside a lands of the wrong shape*.
- [x] 1.2 Refuse `stages` left out of a kind's table that declares a kind, whatever its `lands`, naming `kinds.<kind>.stages` with "is missing, and a kind declares its stages: one or more, or none as `stages = []` where it lands". Keep a table holding `agents` and nothing else the format names as settings, and a table holding `stages` as a kind (design 2, 3; 57, C.4; core.31, core.41). True when `devenv shell -- moon run kinds:test` passes three groups of tests. First, tests holding the kinds spec's *A kind's stages left out are refused, whether it lands or not*, *A table with neither agents nor stages* and *Agents beside stages declared as none*. Second, `tests/refuse.rs` with its expected reasons for `kinds.bare.stages` taking the new phrase. Third, every other existing test unchanged.
- [ ] 1.3 Hold what a kind with no stages reads besides: its version, branch, `starts_unready`, `agents` table and sections, none refused for its having no stages (design 5; 51, 52, 58; core.32). True when `devenv shell -- moon run kinds:test` passes a test over a fixture holding the kinds spec's *A kind with no stages holds what any kind holds*.
- [ ] 1.4 Hold a package whose kind lands with no stages, its `kind.toml` the one file it holds (design 5; 89, 92; 51, 57). True when `devenv shell -- moon run kinds:test` passes a test holding the kind-packages spec's *A package of a kind that lands with no stages*. The test reads the package from a temporary folder through `read_package_dir`, and from the same path and bytes through `read_package`.
- [ ] 1.5 Have the docs say what a kind holds. `Kind::stages` holds at least one stage, or none where the kind lands (51). `read_kinds`' and `read_stages`' comments say when `stages` is refused. `crates/weir-kinds/src/lib.rs`' module doc says a kind's stages are none where it lands. Then run the checks of the project this group touched. True when `devenv shell -- moon run kinds:format kinds:lint kinds:typecheck kinds:test` passes.

## 2. weir check

- [ ] 2.1 Have `weir check` list a kind that lands with no stages as `<name>@<version>` followed by the line `  lands with no stages`, and have `listing`'s doc comment say so (design 6; 51). True when `devenv shell -- moon run weir:test` passes two tests in `crates/weir/tests/check.rs`. One, over a fixture, holds the kinds spec's *A kind that lands with no stages is listed*. The other is `every_problem_of_the_kinds_is_named`, expecting `kinds.bare.stages` with the reason of task 1.2.
- [ ] 2.2 Run the checks of the project this group touched. True when `devenv shell -- moon run weir:format weir:lint weir:typecheck weir:test` passes.

## 3. The pack

- [ ] 3.1 Rename `pack/kinds/a-kind-with-no-stage` to `pack/kinds/a-kind-that-lands-with-its-stages-left-out`, its comment saying it lands with its stages left out and citing 51 and 57 as amended at blueprints 51b6316, its `reason_holds` `a kind declares its stages`. Add `pack/kinds/a-kind-that-lands-with-no-stages`, expecting `train` at version 1 with `stages = []`. Add `pack/kinds/a-kind-that-does-not-land-with-no-stages`, refused naming that kind's `stages` with `a kind that does not land has at least one stage`. Each cites 51 and 57 (design 7; 51, 52, 57, S12, C.7; core.35). True when `devenv shell -- moon run kinds:test` passes, every kinds case answered as it says.
- [ ] 3.2 In `crates/weir-kinds/tests/pack.rs`, have `the_cases_of_s10_s11_and_s12` name `a-kind-that-does-not-land-with-no-stages` as S12's declaration with no stage, in place of `a-kind-with-no-stage`. Add a test that the three cases of task 3.1 are in the pack, the first two refused naming their kind's `stages` and the third reading `train` with no stages (design 7). True when `devenv shell -- moon run kinds:test` passes tests holding the kinds spec's *A kind that lands with no stages and its two neighbours*.

## 4. The projects this change touches

- [ ] 4.1 Run the checks of the projects this change touches. True when `devenv shell -- moon run kinds:format kinds:lint kinds:typecheck kinds:test weir:format weir:lint weir:typecheck weir:test pack:test root:format` passes.
