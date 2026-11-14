# Design

## Context

`weir-bounded` runs a command to a limit: `output(command, limit)`
spawns it in a process group of its own, writes its standard output and
error to unnamed temporary files, and polls every 10 ms until its leader
is reaped and its group is gone, or until the limit, when `end_group`
asks the group to end, gives it `GRACE` and kills it. `Ended` says
`stopped: bool` and gives the leader's `status: ExitStatus`. weir-branch's
`Git::reach` runs the fetch, the listing and the push through it, to the
`Branch`'s limit, a field private to the crate that its constructors set
to `TRY_FOR`.

Boomdrive's runner sets a halt when it is stopped, and each try it has
in flight ends at its limit or its halt. It built that halt in
`boomdrive-bounded`, on boomdrive's
`bolt/the-runners-stop-ends-its-try-in-flight` at `069dc45c`: a `Halt`
of one shared flag, an `output(command, limit, &halt)`, a `Stopped` of
`AtLimit` and `Halted`, and an `Ended` whose exit is none where no
process ran. Boomdrive's main has since run every call to a remote
through `weir-bounded`, and its core.58 rules out the group's ending in
a crate of its own, so the halt moves here.

Record 0008's amendment of 2026-11-12 decides it: a halt is a try's
second end, by the same ending, with words of its own; a try begun with
its halt set runs no process; the wait that reads the limit reads the
halt; weir-branch's reads and writes pass their caller's halt to each
try; a halted push is never replayed. Not taken there: the halt in a
tool's own crate, the group handed to the caller, and a halt in a
static.

The statement's 69 and C.3, as amended at blueprints ad3a1ae, hold as
written: a halt ends a try sooner than its limit, never later, and
neither clause speaks of an end the caller asks for. 44, 46 and 49 hold
as written. The halt rests on record 0008's amendment and core.53.

Rules this design relies on, all in `docs/architecture/core.md`, the one
page whose area the change touches:

- core.1: no crate starts a thread or keeps state. The halt is read by
  the wait that reads the limit, and lives in its caller's handle.
- core.2: the halted call's words and the crate's sources name no tool.
- core.3: the remote named in a halted call's failure is the branch's
  address, which `check_url` holds to no password.
- core.26: a halted push is no refusal and is never replayed.
- core.28: the writers' lock is let go however a write ends.
- core.39: no moon project is added; `bounded` and `branch` keep their
  tasks.
- core.53: the limit, the halt, the group's ending and a stopped call's
  words are `weir-bounded`'s alone; a try ends at its caller's halt,
  whichever comes first, by the same ending and with words of its own;
  a try begun with its halt set runs no process; a halt in a static, or
  one that hands its caller the group, is ruled out.

The design departs from no rule.

## Goals / Non-Goals

**Goals:**

- A try of `weir-bounded` ends at the first of its limit and its
  caller's halt, by the one ending, nothing it started left running.
- A try begun with its halt set runs no process.
- A halted call fails with words of its own, naming its call and its
  remote and no limit.
- weir-branch's read and write take their caller's halt, without
  changing `read`, `write`, `Ask` or the branch's constructors.

**Non-Goals:**

- Moving Boomdrive or sluice onto the halt: each is its tool's change.
- A halt for a git call that reaches no remote, a branch with no
  remote's calls among them: they are no tries and run to their end.
- A halt for the wait on the writers' lock, which is no try.
- crew's Python twin of the protocol, which is crew's.

## Decisions

### 1. The halt is one shared flag the caller holds

```rust
#[derive(Clone, Debug, Default)]
pub struct Halt(Arc<AtomicBool>);

impl Halt {
    pub fn halt(&self);     // set, for this handle and every clone; never cleared
    pub fn halted(&self) -> bool;
}

impl PartialEq for Halt { /* Arc::ptr_eq: the same flag */ }
impl Eq for Halt {}
```

It lives in a module of its own, `crates/weir-bounded/src/halt.rs`,
and `lib.rs` re-exports it. `Halt::default()` gives a halt never set.
`halt` stores `true` and `halted` loads, each with `Ordering::SeqCst`.
Two handles are equal when they share one flag, so a `Branch` holding
one keeps its `PartialEq` and `Eq`. This is `boomdrive-bounded`'s `Halt`
at `069dc45c`, with equality added.

Not taken: a halt in a static, which core.1 and core.53 rule out, and
which would halt every caller of a process at once. Not taken: a
borrowed `&AtomicBool`, which would give `Branch` a lifetime and leave
each caller to build the sharing. Not taken: a closure the wait calls,
which a `Branch` could not clone or compare.

### 2. The run takes a halt beside its limit

```rust
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Stopped { AtLimit, Halted }

#[derive(Debug)]
pub struct Ended {
    pub status: Option<ExitStatus>,  // none only where no process ran
    pub stopped: Option<Stopped>,    // none where it ended by itself
    pub stdout: Vec<u8>,
    pub stderr: Vec<u8>,
    pub stdout_cut: bool,
    pub stderr_cut: bool,
}

pub fn output(command: &mut Command, limit: Duration, halt: &Halt) -> io::Result<Ended>;
pub fn output_kept(command: &mut Command, limit: Duration, kept: u64, halt: &Halt) -> io::Result<Ended>;
```

Every caller passes a halt, `&Halt::default()` where it holds no stop,
so no call to a remote can leave it out (core.53). `end_group`, `span`,
`stopped`, `TRY_FOR` and `GRACE` are unchanged.

Not taken: `output(command, limit)` kept beside an `output_halted`, two
entry points of one run, of which a caller could take the one that
cannot be halted. Not taken: an exit made up for a command that never
ran, `ExitStatus::from_raw(0)`, which reads as a success.

### 3. The wait reads the halt

`output_kept` reads the halt before it makes its files or spawns. Where
it is set, it returns at once: no exit, `Stopped::Halted`, both outputs
empty and neither said cut. Otherwise the wait looks every 10 ms, as it
does now. At each look it reaps the leader and returns, not stopped,
when the leader is reaped and the group is gone. Otherwise it breaks
with `Stopped::AtLimit` once the limit has passed, else with
`Stopped::Halted` when the halt is set, else sleeps. Once it breaks, the
group is ended by `end_group` with `GRACE`, as at the limit now, and the
cause it broke with is the one said. A halt set while the group is being
ended changes nothing. A command that ended by itself in the same look
is not stopped. The wait is the caller's thread: the crate starts none
(core.1).

### 4. A halted call's words

```rust
pub fn halted(call: &str, remote: &str) -> String;
// "<call> with <remote> was stopped as its caller asked"
```

They name the call and the remote, no limit, since the try was not
still going at one, and no tool (core.2).

Not taken: the limit's words for a halted call, which would say it was
still going after thirty seconds when it was not. Not taken:
`boomdrive-bounded`'s `was stopped as the runner stopped`, which names a
tool.

### 5. weir-branch's branch carries its caller's halt

`Branch` gains a field `halt: Halt`, private to the crate, which
`remote` and `local` set to `Halt::default()`. A public builder gives it
the caller's:

```rust
impl Branch {
    pub fn with_halt(self, halt: Halt) -> Branch;
}
```

`lib.rs` re-exports `weir_bounded::Halt` as `weir_branch::Halt`, so a
caller reaches it through the crate whose signature names it.
`read(&branch)` and `write(&branch, &ask, form)` keep their signatures;
the halt goes from the branch to each try. A `Branch` is equal to
another when their names, places and limits are equal and their halts
share one flag.

Not taken: a `halt` field on `Ask`. `read` takes no `Ask`, and `Ask`'s
fields are public, so every caller's literal of it would break. Not
taken: `read_until` and `write_until` beside `read` and `write`, two
entry points of each call.

### 6. `reach` takes the halt and names a halted call

`Git::reach(args, remote, limit, halt)` passes the halt to
`weir_bounded::output`, then matches `Ended::stopped`. `None` gives its
`Ran` with `code` the leader's exit code. `Some(Stopped::AtLimit)` gives
`Err` with the stopped call's failure, as now. `Some(Stopped::Halted)`
gives `Err` with the halted call's failure. In `git.rs`, `stopped(call,
remote, limit, err)` and a new `halted(call, remote, err)` each put
git's last line on standard error and `; ` before their words, where
git wrote one, through one function both call. The call is `git ` and
the command's first argument. A halted call's standard output is never
read.

`read_tip` passes `&branch.halt` to its fetch and its `ls-remote`, and
`move_to` to its push. Each maps the `Err` to a `Failure` at
`Step::Read` or `Step::Move`, as it maps a stopped one. weir-branch's
`lib.rs` doc and the docs of `Branch`, `read` and `move_to` say that
each try ends at the branch's limit or its caller's halt.

### 7. A halted read or push fails the write, with no replay

A halted push returns `Err(Failure)` at `Step::Move` from `move_to`,
never `Moved::Refused`, so `write::tries` maps it to
`WriteError::Failed` at the move, with no fresh read and no replay
(core.26). A halted fetch fails the read and the write that read it. A
halt set after a refused push halts the fresh read, so the write fails
at the read and forms nothing again. The lock is let go when the write
returns, as on any failure (core.28). A write begun with its halt set
takes the lock, runs no git to the remote, fails at the read and lets
the lock go. Whether a halted push landed is read at the next write's
fetch, where every check is made again (46), as for a stopped one.

### 8. A branch with no remote takes no halt

A branch with no remote reaches no remote, so its calls are no tries
and run to their end whatever its halt says. A write of one given a set
halt lands. The wait on the writers' lock reads no halt either: it is no
try, and record 0008's amendment reaches the halt to the tries alone.

### 9. The tests

`weir-bounded`'s tests stay integration tests under its `tests/`. Those
of `output.rs` and `words.rs` pass `&Halt::default()` and read
`Some(Stopped::AtLimit)`, `None` and `status.and_then(|s| s.code())`
where they read `stopped` and `status` now. A new `tests/halt.rs` holds
the halt's own scenarios. A test sets a halt mid-call from a thread of
its own, made by `std::thread::scope`, as weir-branch's `tests/lock.rs`
makes its writers: that thread is the test's, and core.1 rules out one
the crate's code starts. `tests/sources.rs` gains a check that no
source declares a static: no line, trimmed, begins with `static `,
`pub static ` or `pub(crate) static `, and none holds `thread_local!`.

weir-branch's halt tests over a silent remote are in `keeper.rs`'s and
`write.rs`'s `mod tests`, beside the limit's, since the silent remote
of `src/silent.rs` is the crate's own test code. `Silent` gains
`branch_halted_by(halt)`, the branch `inbox` on the silent remote at
the family's limit, given that halt, so a test shows the halt ending a
try well before its limit. It also gains `was_called()`, whether the
silent remote's script wrote its process id. The local branch's test is
in `tests/write.rs`, through the public `Branch::local` and
`with_halt`.

### 10. The README

The README's row for `weir-bounded` says a call to a remote run to the
family's limit for a try, or to its caller's halt, ended with everything
it started.

## Risks / Trade-offs

- [`output`'s and `output_kept`'s signatures and `Ended`'s fields change]
  → Boomdrive is the one caller outside weir, pinned at `9cba691`. Its
  record 0042's amendment of 2026-11-12 already plans the move to this
  halt, in a change of its own.
- [A halted push that landed is reported failed] → the next write's
  fetch reads it, and the check on that tip meets what it wrote (46), as
  for a stopped push.
- [A write waiting on another writer's lock does not end at its halt]
  → that writer's tries are bounded, each at its limit and grace, and a
  stale lock is broken at the stale age its caller sets. Record 0008's
  amendment reaches the halt to the tries alone.
- [A halt is read every 10 ms, so a try ends up to 10 ms after it is
  set, and then within its grace] → well within a service manager's
  stop.

## Migration Plan

Nothing migrates in weir: no record, layout, case, moon project or
command line changes. weir-branch's callers that give no halt run as
before, the `weir` crate's among them. Boomdrive moves its pin and
passes its stop's halt in a change of its own; sluice takes the halt, if
it chooses, when it takes `weir-bounded`.
