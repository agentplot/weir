# Proposal

## Why

Every host and every adapter of the family must be held to one
contract, and nothing yet says whether one keeps it. The conformance
pack answers that question the same way on any machine, with no source
and no network: a host or an adapter that fails one case does not keep
the contract, and a profile, host or adapter that fails the pack is not
one (65 to 68, 86, 87). It is the last part of plan step F2
(`agentplot/blueprints:design/capture/plan.md`), and millpond's core
waits on it: millpond is done when it passes the pack. The design
agent's two rulings for this unit fix the seam between the pack and a
host under test, the six shapes the pack holds, and what the pack's
check counts (blueprints main 81f3f2d and ae9d782: weir 67 and C.7,
millpond C.1).

## What Changes

- **The pack's six shapes, one folder per case.** Cases lie under
  `pack/<shape>/<case>/`, the shapes `host`, `adapter`, `cadence`,
  `branch`, `inbox` and `kinds`, each case naming in `satisfies` the
  clauses it exercises (67; C.7). The cadence cases already in the pack
  move from one file each into one folder each. The branch cases are the
  shared-branch unit's and are counted, not written, here.
- **The pack's check.** A new crate, `weir-pack`, reads every case of
  every shape and an index of the statement's clause numbers kept in
  the pack, and fails naming the number when a case cites a number that
  names no clause, or when a clause of A.1 to A.10 or 71 is cited by no
  case (67, as amended; S16). The check's own tests prove 65, 67 and
  68; it counts neither them nor 69 and 70, which weir's rules hold.
- **Host cases and `weir conform`.** `weir conform <host command>` runs
  a host against every host case through the seam C.7 fixes: a bare
  repository with no remote in a temporary folder, its branch `inbox`
  seeded with the case's starting records, and the host run once or
  twice as the case asks, through its intake, a run of the case's own
  adapter, or an import. After each step it reads the `weir-report/1`
  object the host prints last, and after the last the records at the
  branch's tip, and compares both with the case. A host that keeps raw
  material in a record, or anywhere on the branch, fails, and the
  failure names the case and the clauses it cites (65, 87; S15). It
  says, case by case, which passed and which failed and why, and exits
  non-zero when any failed.
- **The report a host prints.** `weir-capture` writes and reads
  `weir-report/1`, the one JSON object C.7 names: the keys taken, the
  count held, each refusal with its line, key and reason, the listing's
  count and failures, the cursor kept and whether the run was complete
  (23, 76; C.7). A host built on the crates prints it with one call.
- **Adapter cases and the adapter checks.** `weir-pack` holds the
  checks an adapter's lines must pass (1 to 12; C.1) and the meeting
  checks a meeting adapter passes (66), as functions an adapter's own
  tests call over its output. Adapter cases fix lines in and the
  verdict the checks must give, and cases of the folder drop and the
  meeting file fix their input and the lines expected, which must also
  pass the checks (61, 62, 64).
- **Inbox and kinds cases.** Inbox cases fix an inbox's starting
  records, one operation of the records and the records or refusal
  expected (25 to 43, 63; C.2), run against `weir-signals` in its
  tests. Kinds cases fix a declaration and the kinds read or each field
  refused (51 to 58; C.4), run against `weir-kinds` in its tests.
- **A host for the pack's own tests.** An example host built on the
  crates proves `weir conform` passes a host that keeps the contract,
  and fails each fault a host can make, the raw material kept in a
  record among them.

## Capabilities

### New Capabilities

- `conformance-pack`: the pack's layout and shapes, what every case
  cites, the index of the statement's clauses, and the check that fails
  on a number naming no clause or a counted clause no case cites.
- `host-conformance`: host cases and `weir conform`: the seeded bare
  repository, the three invocations, the report and the records
  compared with each case, and what it prints and exits with.
- `adapter-checks`: the adapter checks and the meeting checks, the
  adapter cases that fix their verdicts, and the built-in adapters'
  cases.
- `inbox-cases`: the inbox shape's cases, each a starting inbox, one
  operation and what must come of it, run against the signals crate.

### Modified Capabilities

- `capture-take`: the report of a run, an intake or an import is
  written and read as `weir-report/1`.
- `cadence`: cadence's cases lie one folder each, as every case of the
  pack does.
- `kinds`: the pack holds the kind format's cases, each run against the
  kinds crate.

## Impact

- New crate `crates/weir-pack`, a workspace member and moon project
  `pack`; `weir` gains `conform` and depends on `weir-pack`,
  `weir-signals` and `weir-branch`.
- The change depends on the shared-branch unit
  (`a-shared-branch-takes-one-write`) being in the bolt before its code
  begins: `weir conform` seeds and reads a branch through `weir-branch`,
  and the pack's check needs `pack/branch/` to find 44 to 50 cited.
- Shared files edited: `Cargo.toml`, `Cargo.lock`,
  `.moon/workspace.yml`, `treefmt.nix`.

## Touches

```
Cargo.toml
Cargo.lock
.moon/workspace.yml
treefmt.nix
pack/clauses.toml
pack/host/**
pack/adapter/**
pack/inbox/**
pack/kinds/**
pack/cadence/**
crates/weir-pack/**
crates/weir/**
crates/weir-capture/src/take.rs
crates/weir-capture/src/lib.rs
crates/weir-capture/tests/**
crates/weir-signals/Cargo.toml
crates/weir-signals/moon.yml
crates/weir-signals/tests/**
crates/weir-kinds/Cargo.toml
crates/weir-kinds/moon.yml
crates/weir-kinds/tests/**
crates/weir-adapters/Cargo.toml
crates/weir-adapters/moon.yml
crates/weir-adapters/tests/**
crates/weir-cadence/moon.yml
crates/weir-cadence/tests/pack.rs
```
