# Proposal

## Why

weir-branch reaches a remote three times, each waited on with no limit:
the fetch of a branch's tip, the listing that says whether a branch not
fetched exists, and the push that moves it. A state repository that
takes the connection and never answers holds the write, the writers'
lock of its machine (core.28) and every writer behind it, for as long as
the remote keeps the connection. On deneb on 2026-11-10 a call of that
kind held sluice's reads for 10 to 17 minutes (agentplot-ops's signal
`2026-11-10-agentplot-ops-e19d134d/01`).

agentplot/sluice record 0109 is the family's one design for a call's
limit: each try in a process group of its own, ended at thirty seconds,
asked, given two seconds, then killed. agentplot/boomdrive record 0042
copies it into a crate of the runner's own and names weir-branch's fetch
and push as still unbounded, with no family-wide owner to cite.
agentplot-design ruled on 2026-11-10 that the code owner is weir, since
sluice, Flume and Boomdrive already take weir's crates.

The statement at blueprints ad3a1ae, on the branch
`design/weir-no-try-of-a-call-to-a-remote-outlasts-its-limit`, writes it
in: 69 bounds a try of a call to a remote by the family's limit, C.3
says which of the protocol's calls are tries and what a stopped one
does, and section 9 names `weir-bounded`. Record 0008, proposed, gives
the form, and core.53 says it as a rule, on this branch.

## What Changes

- A new crate, `weir-bounded`, with its moon project `bounded`. It
  holds `TRY_FOR` and `GRACE`, the run of a command to a limit in a
  process group of its own, the group's ending, and a stopped call's
  words. It reads a try's output from unnamed temporary files and starts
  no thread (core.1). Its public surface is the union of sluice's
  `bounded.rs` and `boomdrive-bounded`, so each tool can take it in
  place of its copy.
- weir-branch runs its fetch, its listing and its push through
  `weir-bounded`, each one try to `TRY_FOR`. A stopped fetch or listing
  fails the read. A stopped push fails the write at its move and is not
  replayed. Every other git call runs as before.
- `Cargo.toml`, `.moon/workspace.yml` and the README name the new crate.

## Capabilities

### New Capabilities

- `remote-limit`: a command run to a limit in a process group of its
  own and ended with everything it started, what it printed and whether
  its output was cut, the family's limit and grace, and a stopped call's
  words.

### Modified Capabilities

- `shared-branch-write`: no try of the protocol's call to a remote
  outlasts its limit; a stopped fetch fails the read, and a stopped push
  fails the write without a replay.

## Impact

- New: `crates/weir-bounded/` with its `moon.yml`.
- `crates/weir-branch`: `git.rs` gains the bounded run of a call to a
  remote; `keeper.rs`'s `Branch` carries its try's limit, and
  `read_tip` and `move_to` run their remote calls through it; a
  dependency on `weir-bounded`, and its moon project's `dependsOn` and
  inputs.
- `Cargo.toml`'s members, its crate list and a workspace dependency on
  `libc`, already in `Cargo.lock`; `.moon/workspace.yml`; the README's
  table of crates.
- No record, layout, case or command line changes. A write against a
  remote that answers lands as before.
- sluice, Flume and Boomdrive take `weir-bounded` in changes of their
  own. None moves in this one.
