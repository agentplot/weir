# Proposal

## Why

Every host of the family runs things on a schedule, and each must say
"due" the same way: when a time the schedule names has come since the
last run, or when there has never been a run, and once, not once per
missed time, after a machine has slept. The statement makes this one
rule weir's (59, 60) with the clock as its seam (83 to 85) and five-field
cron in the machine's local time as its profile (C.5). The capture
plan's first build step puts it in the shared crates beside the capture
and signals crates (plan F2), so that the first host's tick has one
copy of it to call.

## What Changes

- A new crate, `weir-cadence`, the first Rust crate in the tree. It
  reads a schedule written as five-field cron and refuses one that is
  not, naming the field and why.
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
- Rust enters the tree: the toolchain pinned once and held to its pin,
  rustfmt in the formatter, and the crate as a moon project of its own
  with format, lint, typecheck and test tasks in the fast layer.

## Capabilities

### New Capabilities

- `cadence`: when a scheduled thing is due, read from a five-field cron
  schedule, the last run and now, caught up once after a sleep, with
  the clock as its one seam.

### Modified Capabilities

None.

## Impact

- New crate `crates/weir-cadence`, joining the workspace's `members`
  and `.moon/workspace.yml`.
- New dependencies: `jiff` for zoned times, and `croner` (with its
  `jiff` feature, default features off) to find the times a cron
  pattern names, including across daylight-saving changes.
- The tree's toolchain: a Rust pin in `.prototools` with
  `rust-toolchain.toml` and a check that holds the two together
  (core.40), rustfmt and the shell formatters in `treefmt.nix`, and a
  rustfmt in `devenv.nix` that is the pinned toolchain's. The other
  units of this bolt that add a crate need the same; whichever lands
  first brings it, and the rest add only their crate's lines.
- No host changes here; millpond's tick and Flywheel's later call this
  crate.

## Touches

```
Cargo.toml
Cargo.lock
crates/weir-cadence/**
.moon/workspace.yml
.moon/toolchains.yml
.prototools
rust-toolchain.toml
check-toolchain.sh
moon.yml
treefmt.nix
devenv.nix
```
