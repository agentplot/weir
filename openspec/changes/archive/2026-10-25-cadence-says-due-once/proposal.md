# Proposal

## Why

Every host of the family runs things on a schedule, and each must say
"due" the same way: when a time the schedule names has come since the
last run, or when there has never been a run, and once, not once per
missed time, after a machine has slept. The statement makes this one
rule weir's (59, 60) with the clock as its seam (83 to 85) and five-field
cron in the machine's local time as its profile (C.5). The capture
plan's first build step puts it in the shared crates beside the capture
and signals crates and the conformance pack (plan F2), so that the
first host's tick has one copy of it to call, and the pack holds cases
that prove it (C.7).

## What Changes

- A new crate, `weir-cadence`. It reads a schedule written as
  five-field cron and refuses one that is not, naming the field and
  why.
- One answer, due or not, computed from the schedule, the last run and
  now alone (59). A thing that has never run is due. A thing is due
  when a named time lies after its last run and at or before now.
- A run missed while the machine slept is caught up once (60): however
  many named times passed, the answer is one "due", and after that run
  the thing is not due until the next named time (S13).
- The schedule is read in the zone of the `now` it is given, and the
  machine's clock gives `now` in the machine's local time (C.5; ruling
  7 of section 10).
- The clock seam (B.5): a clock answers `now` and `last-run`, and
  cadence asks it for those two and nothing else (85). weir supplies
  the machine's `now`; the last run is the caller's to give, since weir
  keeps no state (69).
- Cadence's cases in the conformance pack, the third shape C.7 names: a
  schedule, a last run or none, a now with the zone it is read in, and
  whether the thing is due, each citing 59 or 60. The crate's tests
  answer every case, so 59 and 60 are cited and proved before the
  pack's own check lands.
- Rust in the tree as the batch 18 ruling sets it: pinned in
  `.prototools` alone and held to that pin by the toolchain check
  (core.40), one development shell listing recutils and shellcheck,
  rustfmt and the shell formatters in `treefmt.nix`, and the crate as
  moon project `cadence` with format, lint, typecheck and test tasks in
  the fast layer.

## Capabilities

### New Capabilities

- `cadence`: when a scheduled thing is due, read from a five-field cron
  schedule, the last run and now, caught up once after a sleep, with
  the clock as its one seam, and the pack's cadence cases that prove
  it.

### Modified Capabilities

None.

## Impact

- New crate `crates/weir-cadence`, joining the workspace's `members`
  and `.moon/workspace.yml` as project `cadence`.
- New dependencies: `jiff` for zoned times, and `croner` (with its
  `jiff` feature, default features off) to find the times a cron
  pattern names, including across daylight-saving changes. The tests
  take the workspace's `serde` and `toml` to read the pack's cases.
- New pack files under `pack/cadence/`, one TOML file per case.
- The tree's toolchain: the Rust pin in `.prototools` and nowhere else,
  `rust-toolchain.toml` gone, `check-toolchain.sh` reading the pin from
  `.prototools`, `devenv.nix` listing `pkgs.recutils` and
  `pkgs.shellcheck` once with `devenv.lock` tracked. The bolt's other
  crate units carry the same lines; where one landed first, its lines
  hold and this change adds only what is missing.
- No host changes here; millpond's tick and Flywheel's later call this
  crate.

## Touches

```
Cargo.toml
Cargo.lock
crates/weir-cadence/**
crates/weir-capture/moon.yml
crates/weir-kinds/moon.yml
crates/weir/moon.yml
pack/cadence/**
.moon/workspace.yml
.moon/toolchains.yml
.prototools
rust-toolchain.toml
check-toolchain.sh
moon.yml
treefmt.nix
devenv.nix
```
