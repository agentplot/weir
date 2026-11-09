# Proposal

## Why

No check in weir's gate runs `openspec validate --strict` over the
specs under `openspec/specs`. Run by hand with the openspec devenv.nix
pins, 1.14.3, it refuses ten of the 23 specs. Each refusal is the same
warning: a requirement whose text runs over 500 characters. There are
23 such requirements. A check that runs only by hand is no check
(core.39), so the next spec to grow past the limit would pass the gate.
sluice, flume and boomdrive each run this check as a `specs` task of
their root project.

## What Changes

- The root project gains `specs`, a `check-fast` task running
  `openspec validate --specs --strict --no-interactive` with the
  openspec devenv.nix pins (core.40). It is spelled as sluice's, flume's
  and boomdrive's: no usage statistics sent, no newer release asked
  for, the specs, the OpenSpec config, `devenv.nix` and `devenv.lock`
  its inputs.
- Each of the 23 long requirements is split into requirements that
  together say what it said, each 500 characters or fewer. The
  requirement split keeps its name and every scenario, as
  `openspec archive` requires. Each part added beside it carries a copy,
  word for word, of each scenario that shows its behavior.
- No behavior changes. No clause, rule, record, code, test or pack case
  changes.
- `CLAUDE.md`'s table of checks and `.moon/workspace.yml`'s comment name
  the new task.

## Capabilities

### New Capabilities

None.

### Modified Capabilities

Each of these has its long requirements split, saying what they said:

- `adapter-checks`: the adapter checks and the meeting checks.
- `cadence`: the pack's cadence cases.
- `capture-take`: the report `weir-report/1`.
- `host-conformance`: a host case, its steps, its report and its records
  at the tip, and the pack's host cases.
- `import-lines`: a move carried as it stands.
- `inbox-cases`: an inbox case and the clauses the inbox cases hold.
- `kind-packages`: `read-package`, and a package's one kind and its
  landing fix.
- `kinds`: a declaration's kinds, a stage, the refusal of a declaration,
  `weir check`, the pack's kinds cases, settings, and a kind's landing
  fix.
- `meeting-file`: a meeting's event time and its front matter.
- `signals-folder-import`: the import lines of a body's capture folders
  and the moves read with them.

## Impact

- `moon.yml`: the root project's `specs` task.
- `CLAUDE.md`, `.moon/workspace.yml`: the checks they list.
- `openspec/specs/`: the ten specs above, once this change is archived.
- Nothing a crate reads or builds changes. No scenario's words change,
  and each copy is proved by the test that proves the scenario it
  copies.
- Open changes are not yet validated by the gate. In sluice that check
  came as a chore of its own, and here it is left to one too.
