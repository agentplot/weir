# Proposal

## Why

An inbox lies on a shared branch that several machines and tools write at
once: two hosts taking one release, curation and the operator moving one
signal in the same second. Unless every writer meets one protocol, one of
those writes is lost, or the branch takes two writes from one tip and the
second check is never made (44 to 50). This change builds `weir-branch`,
the crate section 9 names for the write protocol, so millpond and Flywheel
write a branch one way, and gives the conformance pack the protocol's
cases, so crew's Python twin of the same steps is held to them too (C.3;
plan step F4, `agentplot/blueprints:design/capture/plan.md`).

## What Changes

- **Reading a branch.** A writer reads a branch's tip, the commit it
  stands at, and can read every file at that tip whole; a branch that does
  not exist yet reads as no tip. Reading changes nothing (78, 80; I7).
- **A write is formed against the tip read.** The writer's caller is handed
  the tip and its files, and gives back the files the write adds or
  replaces, nothing to write, or a refusal naming why. Every check the
  write rests on is the caller's, made against that tip, plus `recfix` on
  every `.rec` file the write changes (46; C.3 step 2).
- **One commit, no working copy.** The write is one commit on the tip read,
  made through a temporary index in a bare cache the caller names, touching
  no working tree, with a message naming what it did and who asked
  (45, 47, 50; C.3 step 3).
- **It lands only on that tip.** The branch is moved only while it still
  stands at the tip read: a push with `--force-with-lease` naming that tip,
  never with force; a branch with no remote moves by `update-ref` from that
  tip (44, 48, 79; C.3 step 4).
- **A moved branch replays the write.** When the branch has moved, the
  write is formed again on the new tip, every check made again there, up to
  the bound the writer sets, five unless it says otherwise; then it is
  reported refused, naming why, and never tried again on its own (44;
  C.3 step 5; record 0002, ruling 4). A push refused while the branch has
  not moved is a failure reported with git's words, not a replay.
- **A lock keeps one machine's writers in turn.** A lock in the cache's git
  directory names its holder and when it was taken; a writer waits for it,
  breaks it once it is older than the age the caller sets, and never lets
  it stand in for the check at the tip (49; C.3).
- **The protocol's cases.** `pack/branch/<case>/case.toml`, each citing the
  clauses it exercises: a write on a branch read, a moved branch replayed,
  a held key and a second standing move met again at the new tip (S5, S6),
  the bound, a branch with no remote (S7), a file `recfix` refuses, and a
  held and a stale lock. The crate's tests run every case; the case format
  is written so crew's twin can run the same cases (65, 67, 68; C.3, C.7).
- The crate is moon project `branch`, with format, lint, typecheck and test
  tasks (core.39).

## Capabilities

### New Capabilities

- `shared-branch-write`: reading a shared branch and writing it through the
  one protocol: formed against the tip read, checked there, one commit with
  no working copy, landing only on that tip, replayed up to the writer's
  bound, on a branch with a remote or without one, naming what it did and
  who asked.
- `shared-branch-lock`: the lock that keeps one machine's writers in turn,
  what it names, how a writer waits for it and breaks a stale one, and that
  it never stands in for the check at the tip.
- `shared-branch-cases`: the conformance pack's cases for the write
  protocol, the form any implementation of the protocol runs them in, and
  the clauses each cites.

### Modified Capabilities

None.

## Impact

- New crate `crates/weir-branch`, a library depending on `time` and
  `tempfile`, with `toml` and `serde` for its tests; it runs `git` and `recfix`
  for the write its caller asks for, both from `devenv.nix`. It joins the
  workspace's `members`, with moon project `branch` in
  `.moon/workspace.yml`; `Cargo.toml`'s comment names the branch crate
  landed, and `Cargo.lock` gains it.
- New `pack/branch/`, the protocol's cases, TOML, which `treefmt.nix`'s
  taplo already claims.
- No change to `weir-capture`, and no dependency on `weir-signals`: a host
  hands the protocol the inbox's operations as the way it forms a write,
  so the inbox's checks are the ones made at each tip.
- crew's twin is not changed here: running these cases from crew's tests
  is crew's work.

## Touches

```
Cargo.toml
Cargo.lock
.moon/workspace.yml
crates/weir-branch/**
pack/branch/**
```
