# Design

## Context

weir-branch reaches a branch's remote three times, in `keeper.rs`:
`read_tip`'s `git fetch` of the tip into the cache, the
`git ls-remote --exit-code` that says, after a failed fetch, whether
the branch exists, and `move_to`'s `git push` with the lease. Each runs
through `git.rs`'s `run_program`, which waits by `Command::output` with
no limit. A write takes its machine's writers' lock before its first
read and lets it go after its move (core.28), so a remote that takes
the connection and never answers holds the write, the lock and every
writer of that machine behind it.

agentplot/sluice record 0109, on sluice's main at `e6809f9`, is the
family's one design for a call's limit. Each try runs in a process group
of its own and is ended at `TRY_FOR`, thirty seconds: its group is asked
to end, given `GRACE`, two seconds, then killed, so git's remote helper
ends with it. A stopped try fails as `git <verb> with <remote> was still
going after 30s, the most a try may take, and was stopped`.
agentplot/boomdrive record 0042 copies it into `boomdrive-bounded` and
names weir-branch's calls as still unbounded. agentplot-design ruled on
2026-11-10 that weir owns the code, since sluice, Flume and Boomdrive
already take weir's crates.

The statement at blueprints ad3a1ae, on the branch
`design/weir-no-try-of-a-call-to-a-remote-outlasts-its-limit`, writes it
in:

- 69: a try of a call weir makes to a remote waits no longer than the
  family's limit, and is then ended with everything it started, the
  call failing and naming the call, the remote and the limit.
- C.3: the fetch, the listing and the push are each one try; a stopped
  fetch or listing fails the read, and a stopped push fails the write
  and is not replayed; no writer's setting changes the limit.
- Section 9: `weir-bounded` is a crate of the stack, which sluice,
  Flume and Boomdrive take in place of their copies, each in a change of
  its own.

44, 46, 78 and 79 hold as written. Record 0008, proposed on this branch,
carries agentplot-design's ruling and this design's form, each part
cheap to change.

Rules this design relies on, all in `docs/architecture/core.md`, the one
page whose area the change touches:

- core.1: no crate starts a thread or leaves a child running. Amended
  on this branch: its details name each try ended at its limit, and an
  unnamed temporary file a call lets go before it returns, as not ruled
  out.
- core.2: `weir-bounded` names no tool of the family.
- core.3: the remote named in a stopped call's words is the branch's
  address, which `check_url` already holds to no password.
- core.26: a stopped push is no refusal and is never replayed.
- core.27 and core.28: the push keeps its lease, and the lock is let go
  however the write ends.
- core.39 and core.40: the new crate is a moon project of its own, one
  task per check, its tools the tree's pins.
- core.53, written on this branch: the limit, the group's ending and the
  words are `weir-bounded`'s alone, and every call weir makes to a
  remote takes them.

Where weir has no rule of its own, kit:testing.1 and kit:testing.3 bind
the new moon project's tasks. The design departs from no rule.

## Goals / Non-Goals

**Goals:**

- No try of weir-branch's fetch, listing or push runs past `TRY_FOR`
  and its grace, and nothing it started outlives it.
- A stopped try fails naming its call, its remote and the limit.
- One crate holds the limit, the group's ending and the words, with a
  surface sluice's `bounded.rs` and `boomdrive-bounded` can each be
  swapped for.

**Non-Goals:**

- Moving sluice, Flume or Boomdrive onto `weir-bounded`: each is its
  tool's change.
- crew's Python twin of the protocol (C.3), which is crew's.
- A retry of a stopped try, or a backoff: weir has none for a failed
  read today, and a write's caller runs it again.
- Bounding a git call that reaches no remote.

## Decisions

### 1. One crate, `weir-bounded`, beside the others

`crates/weir-bounded` is a library crate depending on `libc`, for
`killpg`, and `tempfile`, and on none of weir's crates. `libc` joins the
workspace's dependencies at `0.2`; `Cargo.lock` already holds it as
another crate's dependency. Its moon project is `bounded`, with
`format`, `lint`, `typecheck` and `test`, each tagged `check-fast`, as
weir-cadence's are. weir-branch depends on it, and its moon project
gains `dependsOn: [bounded]` and `/crates/weir-bounded/**/*` among its
inputs. The crate is Unix's alone, as a process group is; weir runs on
no other host. Its `src/lib.rs` carries `#![deny(missing_docs)]`.

Not taken: a module inside weir-branch, which a tool would take the
whole write protocol to reach.

### 2. Its surface

```rust
pub const TRY_FOR: Duration = Duration::from_secs(30);
pub const GRACE: Duration = Duration::from_secs(2);

#[derive(Debug)]
pub struct Ended {
    pub status: ExitStatus,
    pub stopped: bool,
    pub stdout: Vec<u8>,
    pub stderr: Vec<u8>,
    pub stdout_cut: bool,
    pub stderr_cut: bool,
}

pub fn output(command: &mut Command, limit: Duration) -> io::Result<Ended>;
pub fn output_kept(command: &mut Command, limit: Duration, kept: u64) -> io::Result<Ended>;
pub fn end_group(group: i32, grace: Duration, reap: &mut dyn FnMut() -> bool);
pub fn stopped(call: &str, remote: &str, limit: Duration) -> String;
pub fn span(limit: Duration) -> String;
```

This is the union of sluice's `bounded.rs` and `runs::end_group` and of
`boomdrive-bounded`. `output` is `output_kept` keeping every byte.
`output_kept` stands for sluice's `finish(child, limit, kept)`: the
caller hands the command, not a child it spawned, so its input is a file
it sets as the command's standard input, as weir-branch hands git its
input today. `end_group` is public so that sluice's checks, which end
their groups through `runs::end_group`, can take it too. The error of
`output` and `output_kept` is the spawn's, or a temporary file not made
or read.

### 3. Output read from files, with no thread

`output_kept` makes two unnamed temporary files through
`tempfile::tempfile`, hands a clone of each to the command as its
standard output and error, and spawns it with `process_group(0)`. Once
the command has ended, or its group has been ended, each file is read
from its start: its first `kept` bytes kept, and the part said cut when
the file holds more. Not taken: a thread reading each pipe, as sluice
and Boomdrive read them. core.1 rules out a thread a crate starts, and
a pipe held open by a process outside the group keeps a reader waiting,
which sluice waits out with a further 200 ms. A file is read whole
whoever holds it.

### 4. When a command has ended

The wait polls every 10 ms. It reaps the leader with `try_wait`, and
once the leader is reaped it asks `killpg(group, 0)` whether any process
of the group is left. The command has ended when the leader is reaped
and the group is gone. When the limit comes first, `end_group` ends the
group, reaping the leader as it goes, and the command is said stopped,
its exit the leader's: by itself, or by the signal that ended the group.
A process of the group that outlives its leader is ended with the rest
at the limit, where a pipe would have let one that closed its output
outlive the call. git's own detached maintenance leaves the group by
`setsid`, so it is not waited for.

### 5. The group's ending

`end_group(group, grace, reap)` is sluice's `runs::end_group`, as
`boomdrive-bounded` copies it. It calls `reap`, then asks whether the
group is there, returning at once when it is not. It sends `SIGTERM` to
the group and waits, every 50 ms, up to `grace` for it to go. It then
sends `SIGKILL` and waits, every 20 ms, up to five seconds for it to go.
A group is there while `killpg(group, 0)` succeeds or fails with
anything but `ESRCH`. `reap` reaps the leader, whose parent the caller
is, since a leader left a zombie keeps its group there.

### 6. The limit and the grace

`TRY_FOR` is thirty seconds and `GRACE` two, record 0109's, constants
no function reads a setting for. A host whose calls need longer raises
`TRY_FOR`, one value for the family. In weir-branch a `Branch` carries
its try's limit in a field its constructors set to `TRY_FOR`, private
to the crate, so that the keeper's and the write's unit tests can give
a branch 300 ms. No public function sets it. Not taken: a public
setter, which would let a host bound its calls differently from the
tools that read the same remotes.

### 7. The words

`stopped(call, remote, limit)` gives `<call> with <remote> was still
going after <span>, the most a try may take, and was stopped`, and
`span` gives whole seconds as `30s` and anything else in milliseconds,
as `300ms`, as sluice's and Boomdrive's do. In weir-branch, `git.rs`
makes a stopped call's failure: the last line git wrote on standard
error, trimmed, and `; ` where there is one, then the words. The call is
`git ` and the command's first argument, `fetch`, `ls-remote` or
`push`. The remote is the branch's address. A stopped call's standard
output is never read.

### 8. Which calls are tries

`git.rs`'s `Git` gains `reach(args, remote, limit)`, which builds the
same command as `run`, in the same environment, and runs it through
`weir_bounded::output`. A stopped call returns `Err` with its failure,
as a git that could not start does. Any other end returns its `Ran` as
`run` would. `read_tip` runs its fetch and its `ls-remote` through
`reach`, and `move_to` its push. Each maps the `Err` to a `Failure`, at
`Step::Read` or `Step::Move`, as it maps a git that could not start.
`run` and `run_program` stay for every call that reaches no remote: the
cache's `init`, `rev-parse`, `update-ref`, `cat-file`, `ls-tree`, and
the commit's calls.

Not taken: every git call bounded, which would add a way to fail,
killed partway through a local write, with nothing to bound.

### 9. A stopped push fails the write

`move_to` returns a stopped push as `Err(Failure)` at `Step::Move`,
never as `Moved::Refused`. So `write::tries` maps it to
`WriteError::Failed` at the move, with no fresh read and no replay.
The lock is let go when the write returns, as on any failure. Whether
the push landed is read at the next write's fetch, where every check is
made again (46; core.26). A write that did land meets its own records
there: a held key, a second standing move, a used item name.

Not taken: a stopped push taken as a refusal and replayed, which forms
the write again on a tip that may already hold it. Not taken: the
branch read again before deciding, as record 0109 has sluice do, which
asks the remote that just held a try for thirty seconds to answer again
within the same call.

### 10. The tests' silent remote

weir-branch's unit tests reach no network, as the pack's cases reach
none (68). A test's silent remote
is `ssh://silent.invalid/inbox.git`, reached from a bare cache the test
makes with `git init --bare`. The cache's `core.sshCommand` is a shell
that writes its process id to a file in the test's folder and then
`exec`s `sleep 60`, and its `ssh.variant` is `simple`, so git runs it
once, with no probe. A fetch or a push then waits on a process that
never answers. The test reads the id, and once the call has returned,
asks `kill -0` whether that process is gone. The keeper's tests are in
`keeper.rs`'s `mod tests`, and the write's in a `mod tests` of
`write.rs`, since a branch's limit is the crate's alone to set.

`weir-bounded`'s tests are integration tests under its `tests/`, run
through `sh`, `sleep` and `cat`, each to a limit of 300 ms or ten
seconds. A background process's id is printed by the command and asked
of with `libc::kill(pid, 0)`. The crate's sources are read by a test
that finds no tool named and no thread started.

### 11. The README and the workspace

`Cargo.toml` lists `crates/weir-bounded` in `members` and in its
comment's list of crates. `.moon/workspace.yml` lists `bounded`. The
README's table of crates gains `weir-bounded`, beside `weir-branch`.
weir-branch's `lib.rs` doc says each try of git that reaches the remote
is ended at `weir-bounded`'s limit.

## Risks / Trade-offs

- [A fetch or push that needs longer than thirty seconds, a large first
  fetch on a slow link, is stopped every time] → `TRY_FOR` is raised for
  the family, as record 0109 says. weir's inboxes are small and its
  fetches incremental.
- [A stopped push that landed is reported failed] → the next write's
  fetch reads it, and the check on that tip meets what it wrote (46).
- [git in a group of its own no longer takes a terminal's Ctrl-C with
  its caller] → a caller ended by a signal leaves its try running until
  it ends by itself, as sluice's and the runner's do. A caller ended by
  `SIGTERM` already left git running before this change, since the
  signal reaches the caller alone.
- [A process of the group that sets its own user cannot be signalled,
  and is taken as still there] → the wait for it after the kill is
  bounded at five seconds, and the call returns.
- [A write against a silent remote now takes about 32 seconds to fail,
  where before it never failed] → that is the change.

## Migration Plan

Nothing migrates: no record, layout or command line changes, and a write
against a remote that answers lands as before. sluice, Flume and
Boomdrive each take `weir-bounded` in a change of their own, from the
revision of weir that lands this one.
