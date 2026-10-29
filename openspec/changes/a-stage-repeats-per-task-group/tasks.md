# Tasks

## 1. The kinds crate

- [x] 1.1 Read a stage's `repeat` into `Stage::repeat`, an `Option<Repeat>` with the one variant `PerTaskGroup` spelled `per-task-group`, none when left out, `repeat` among `STAGE_FIELDS` (53, C.4). True when `devenv shell -- moon run kinds:test` passes the tests for "A stage reads whether it repeats per task group".
- [x] 1.2 Refuse, naming the stage's `repeat`, any value but `per-task-group`, and the field on a conversation or on a stage whose deliverable is not `tasks`, judging it against no end the read already refused (57, C.4). True when `devenv shell -- moon run kinds:test` passes the tests for "A repeat the format does not allow is refused".
- [x] 1.3 The pack's kinds case format names a stage's `repeat`; `pack/kinds/flumes-change-2-read-twice-the-same` holds change@2 with its code stage repeating, and `pack/kinds/repeat-on-a-stage-of-commits` its refusal (53, 57, 58; S10; C.7). True when `devenv shell -- moon run kinds:test` passes.

## 2. The projects this change touches

- [x] 2.1 Run the checks of the projects this change touches. True when `devenv shell -- moon run kinds:format kinds:lint kinds:typecheck kinds:test pack:test weir:test root:format` passes.
