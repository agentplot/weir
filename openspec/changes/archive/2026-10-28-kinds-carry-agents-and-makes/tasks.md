# Tasks

## 1. The kinds crate

- [x] 1.1 Read a kind's `agents` table into `Kind::agents`, a `KindAgents` of the settings for every stage and each `agents.stages.<stage>` table, as written, empty when left out; refuse each table of the wrong shape naming the field (52, 57, C.4). True when `devenv shell -- moon run kinds:test` passes the tests for "A kind's agents table reads with the kind" and "An agents table of the wrong shape is refused".
- [x] 1.2 Return a `[kinds.<kind>]` table holding `agents` and nothing that declares a kind in `Declaration::settings` as `KindSettings`, never refused as a kind with no stage (57, C.4). True when `devenv shell -- moon run kinds:test` passes the tests for "A kind's table holding agents and no stages gives settings".
- [x] 1.3 Read a stage's `makes` into `Stage::makes`, none when left out, refused naming the field when not an array of non-empty strings (53, 57). True when `devenv shell -- moon run kinds:test` passes the tests for "A stage reads what it makes".
- [x] 1.4 The pack's kinds case format names `settings`, and `pack/kinds/settings-for-a-kind-declared-elsewhere` holds a kind's `agents`, a stage's `makes` and a settings-only table (52, 57; C.7). True when `devenv shell -- moon run kinds:test pack:test` passes.

## 2. The projects this change touches

- [x] 2.1 Run the checks of the projects this change touches. True when `devenv shell -- moon run kinds:format kinds:lint kinds:typecheck kinds:test pack:test weir:test` passes.
