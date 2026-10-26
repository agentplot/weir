# Proposal

## Why

flume runs a kind's places and stages, and sounding runs an elaboration
kind's rounds too, and each reads kinds a tool, a repository or a launch
declares. Neither can be built until the family has one reader of the
one format, so that a kind declared for flume is read by sounding with
no format of its own (purpose; 51 to 58). flume's core waits on it
(plan step F4, L3).

## What Changes

- A new crate, `weir-kinds`, reads a TOML declaration into every kind it
  declares: each kind's version, its stages in order, how its places'
  branches are named, whether its first stage starts unready, whether it
  lands, and, on a stage, where a failed submission sends a place back
  and how many times (51, 52, 53, C.4).
- Every stage ends on exactly one deliverable, spelled one of six ways,
  or is a conversation with none (51, 54).
- A stage's quiet time is one whole number followed by `s`, `m` or `h`,
  as `45m` (57, C.4).
- A declaration is checked whole. One that fails is refused with every
  field that fails it named, each with its reason, and nothing of it is
  returned (57, 81). Two stages of one kind sharing a name, and an
  `on_fail` naming no stage of its kind, are refused the same way (57,
  C.4).
- The format gives no default for `quiet`, `on_fail`, `max_send_backs`
  or an elaboration's `ending`: each comes back left out when it is
  left out, and what that means is the reader's (58, C.4).
- A table or field the format does not name comes back unread and never
  fails the read (56, 82). An elaboration kind's section is one of
  these to a reader of places, and is read, and checked, only when its
  reader asks for it (55).
- The same declaration always reads the same, and reading changes
  nothing (58, 82). Which declaration a kind's name resolves to stays
  the reader's rule (58).
- A new crate, `weir`, is the family's `weir` command line, holding for
  now one command: `weir check <declaration>` reads a declaration, the
  elaboration section included, and either lists its kinds or names
  every field that fails it (section 9).
- The workspace gains its Rust toolchain, pinned in `.prototools` alone
  and checked on every host that runs the gate, and its one shell lists
  recutils and shellcheck (core.39, core.40).

## Capabilities

### New Capabilities

- `kinds`: the kind format, read and checked: a declaration's kinds and
  stages, their deliverables, the sections a reader does not know, the
  elaboration section, and `weir check`.

### Modified Capabilities

None.

## Impact

- New crates `crates/weir-kinds` and `crates/weir`, each a workspace
  member and a moon project of its own, named `kinds` and `weir`.
- The workspace's first Rust: `.prototools` pins it, `.moon/toolchains.yml`
  reads the pin, a root task fails when a tool on the path is another
  version, and treefmt formats Rust and shell. No other file names the
  version.
- `devenv.nix` lists recutils and shellcheck once, for every crate of
  the workspace; `devenv.lock` stays tracked.
- New dependencies: `toml` for reading, `clap` for the command line.
- millpond, flume and sounding can depend on `weir-kinds` at a tag;
  flume's own kinds read from a declaration (S10).

## Touches

```
Cargo.toml
Cargo.lock
rust-toolchain.toml
.prototools
.moon/toolchains.yml
.moon/workspace.yml
moon.yml
check-toolchain.sh
treefmt.nix
devenv.nix
devenv.lock
crates/weir-kinds/**
crates/weir/**
```
