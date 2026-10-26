# Proposal

## Why

Every tool in the family that hands captures in (headwater's and eddy's
adapters, sounding's offers, the built-in adapters) and every host that
takes them (millpond now, Flywheel later) must agree on one contract, or
two hosts given the same lines write different inbox records. weir has
no code yet; this change builds its first crate, `weir-capture`, which
holds that contract once, so the rest of the bolt and the tools after it
build on it (plan step F2, `agentplot/blueprints:design/capture/plan.md`).

## What Changes

- **An adapter's lines.** A line an adapter prints is read as exactly one
  of an offer, a cursor or a completion, or refused as malformed with its
  reason (1, 2, 7 to 9; C.1). The crate also prints lines, so an adapter
  built on it says them the same way.
- **The key rule.** A key is made of `/`-separated segments and is
  refused when it starts with `/`, has a segment that is empty, `.`,
  `..` or the name of a file the inbox's layout places (`capture.rec`,
  `signals.rec` and the rest), or begins with `captures`, `moves` or
  `items`, so it names nothing in an inbox but its own capture (3, 4;
  C.1).
- **An adapter's declaration.** The `[adapter.<name>]` table is read
  from TOML, its command, its configuration's JSON Schema (draft 2020-12)
  and its secrets by name; a configuration is checked against that
  schema; and the command line and environment names a run is started
  with are formed from it alone (11, 72, 73; C.1).
- **How a host takes lines.** One function takes a run's or an intake's
  lines in the seven steps of C.1, through two seams the host supplies,
  its inbox and its raw bank: an offer missing a key, a source or an
  event time, or with a bad key or an incomplete own excerpt, is refused
  and named; a key the inbox holds is counted held and writes and banks
  nothing; the raw material is banked whole before the capture is
  written, with a pointer `<hostname>:<path>`; an own excerpt becomes the
  capture's one signal, cut at a line to fit the limit with C.1's one
  closing line, `[cut: <n> lines and <b> bytes left out, banked whole
  with the capture]`, and the whole text banked, and a limit too short
  to hold that line is refused; every other
  capture is unread; and the run is reported, complete or not, with the
  cursor it keeps (13 to 24, 75 to 77).
- **Same lines, same records.** The capture's record, `weir-capture/1`,
  is written in recutils, and the own signal is handed to the host as a
  value; nothing but the taken-at time, the machine and how the capture
  came in varies between two hosts given the same lines (24; I8).
- **The sha256 content hash** the built-in adapters key files by (C.6).
- **The Rust toolchain and the shell.** weir's first crate brings Rust
  into the workspace, pinned in `.prototools` alone and checked, the
  shell's recutils and shellcheck, and its moon project, `capture`, with
  format, lint, typecheck and test tasks (core.39, core.40). This unit
  lands first of the bolt's three, so it carries what madswan-design's
  batch 18 ruling gives the first unit.

## Capabilities

### New Capabilities

- `adapter-contract`: what an adapter tells a host and how it is
  declared and started: the three lines, the key rule, the offer's
  fields, the declaration and its configuration's schema, and the
  content hash.
- `capture-take`: what a host does with an adapter's lines: refusal,
  holding, banking, the capture record, the own signal and its cut, and
  the run's report.

### Modified Capabilities

None.

## Impact

- New crate `crates/weir-capture`, joining the workspace's `members`,
  with a moon project `capture`.
- The workspace gains Rust: `.prototools` alone pins it,
  `.moon/toolchains.yml` reads the pin, a root check fails when a Rust
  tool on the path is not the pinned build, and `treefmt.nix` formats
  Rust and shell. `rust-toolchain.toml` is not in the tree.
- `devenv.nix` lists GNU recutils, so the crate's tests hold its records
  to `recfix`, and shellcheck, for the shell's scripts; `devenv.lock`
  stays tracked.
- Later units build on this crate: the inbox's records and layout (the
  own signal's record, `weir-signal/1`, and where each record lies), the
  built-in adapters, and the conformance pack.

## Touches

```
Cargo.toml
Cargo.lock
crates/weir-capture/**
.moon/workspace.yml
.moon/toolchains.yml
.prototools
rust-toolchain.toml
check-toolchain.sh
moon.yml
treefmt.nix
devenv.nix
devenv.lock
```
