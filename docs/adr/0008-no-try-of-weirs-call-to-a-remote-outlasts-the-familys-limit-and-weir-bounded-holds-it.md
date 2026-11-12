---
number: 8
title: No try of weir's call to a remote outlasts the family's limit, and weir-bounded holds it
status: accepted
date: 2026-11-11
decision-makers:
- agentplot-design
- the construct stage of change/no-try-of-weirs-call-to-a-remote-outlasts-its-limit
---

# No try of weir's call to a remote outlasts the family's limit, and weir-bounded holds it

Status: accepted on 2026-11-12 by agentplot-design. It is the design agent's own choice, made on the statement and the rules written, and cheap to change: it holds no choice of the user's. The direction is agentplot-design's ruling of 2026-11-10: weir owns the code of a call's limit, since sluice, Flume and Boomdrive already take weir's crates. The limit itself is agentplot/sluice record 0109's, the family's one design for it. The form below is the construct stage's own choice, each part cheap to change.

Sources: the brief of the change `no-try-of-weirs-call-to-a-remote-outlasts-its-limit`; agentplot/sluice record 0109, on sluice's main at `e6809f9`, with its `crates/sluice/src/bounded.rs`, `runs::end_group` and `retry::TRY_FOR` there; agentplot/boomdrive record 0042 and its rule core.58, on boomdrive's `bolt/no-try-of-the-runners-call-to-a-remote-outlasts-its-limit` at `9a72117f`, with its crate `boomdrive-bounded`; agentplot-ops's signal `2026-11-10-agentplot-ops-e19d134d/01`; the statement in agentplot/blueprints, `design/weir/requirements.md`, 44, 46, 69, C.3 and section 9 as amended at blueprints `ad3a1ae`; `crates/weir-branch/src/keeper.rs` `read_tip` and `move_to`, and `crates/weir-branch/src/git.rs` `run_program`, at `c781f73`; rules core.1, core.2, core.3, core.26, core.27, core.28, core.39 and core.40.

## Context and problem statement

On deneb on 2026-11-10, sluice's reads of one repository each hung for 10 to 17 minutes. The cause was a git call to a remote with no limit: its helper waited on a socket the remote had taken and never answered. Record 0109 bounds every try of sluice's own calls at thirty seconds. Record 0042 bounds the runner's fetch and listing the same way, through a copy in a crate of the runner's own. It names weir-branch's fetch and push of crew's state repository as still unbounded, and says the family has no shared owner of a call's limit to cite.

weir-branch reaches a remote three times, each by `Command::output` with no limit: `read_tip`'s `git fetch` of the branch's tip, the `git ls-remote` that says whether a branch not fetched exists, and `move_to`'s `git push`. A write holds the writers' lock of its machine across all three (core.28). So one silent state repository holds that write, every later writer of that machine behind the lock, and the partition of whatever tool is writing.

agentplot-design ruled on 2026-11-10 that the code owner is weir.

## Considered options

For the owner:

1. **A crate of weir's own, `weir-bounded`, that weir-branch calls and any tool of the family takes in place of its copy** (chosen).
2. **A module inside weir-branch.** A tool would take the whole write protocol to bound a call of its own.
3. **Each tool keeps its copy.** sluice's `bounded.rs` and boomdrive's `boomdrive-bounded` already differ: boomdrive's has no kept part, and its group's ending is a copy of sluice's.

For reading what a try prints:

1. **Unnamed temporary files, read once the try has ended** (chosen).
2. **Each pipe read on a thread of its own**, as sluice and Boomdrive read them. core.1 rules out a thread a crate starts.

For a stopped push:

1. **The write fails at its move, naming the push, and is not replayed** (chosen).
2. **Taken as a refusal and replayed**, as any failed push is today. A stopped push may have landed, and a replay forms the write again on a tip that may hold it.
3. **The branch read again before deciding**, as record 0109 has sluice do. The remote that just held a try for thirty seconds is asked again in the same call.

## Decision outcome

- **The limit is the family's.** A try of a git call to a remote runs in a process group of its own. At thirty seconds, `TRY_FOR`, its group is asked to end, given two seconds, `GRACE`, then killed, so git's remote helper ends with it. No caller's setting changes the limit: a host whose calls need longer raises `TRY_FOR`, one value for the family, as record 0109 has it. A unit test of weir-branch gives its branch a shorter one, which the crate's own code alone can set.
- **One owner.** `crates/weir-bounded` holds `TRY_FOR`, `GRACE`, the run of a command to a limit, the group's ending and a stopped call's words. It depends on none of weir's crates, and names no tool of the family (core.2). Its public surface is the union of sluice's `bounded.rs` and `boomdrive-bounded`: `output`, `output_kept`, `Ended` with its cut flags, `stopped`, `span` and `end_group`. Each tool can then take it in place of its copy as a dependency swap, in a change of its own. Not taken: option 2, since a tool would depend on the write protocol to bound a call of its own. Not taken: option 3, since one behaviour would have owners that drift apart, record 0109's reason for one owner.
- **No thread.** A try's standard output and error are written to unnamed temporary files, as weir-branch already hands git its input. Each file is read once the try has ended, its first `kept` bytes kept and the rest said cut. A try has ended once its leader has exited and no process of its group is left. A process of its group that outlives its leader is ended at the limit with the rest. Not taken: a thread per pipe, which core.1 rules out. A pipe also lets a process outside the group hold the call open, which a file does not.
- **Only a call that reaches the remote.** The bounded calls are weir-branch's `git fetch`, `git ls-remote` and `git push` of a branch on a remote. Every other git call, all local to the cache or the repository, runs to its end as before. Not taken: every git call bounded, which would add a way to fail, killed partway through a local write, with nothing to bound.
- **The words.** A stopped call fails with `git <fetch|ls-remote|push> with <remote> was still going after 30s, the most a try may take, and was stopped`. Where git wrote a line on standard error, its last line comes first, then `; `. The remote is the branch's address, which holds no password (core.3). A stopped call's standard output is never read as an answer.
- **One try.** A stopped fetch or listing fails the read, `Step::Read`, and the write that read it. A stopped push fails the write at `Step::Move`. It is no refusal, so it is never replayed (core.26). The next write's fetch reads whether it landed, and every check is made there again (46). Not taken: option 2, replay, which forms a write again on a tip that may already hold it. Not taken: option 3, a read before deciding, which doubles what a silent remote costs the writer. weir has no retry of a failed read, and a write's caller runs it again.
- **The lock.** The writers' lock is let go when a write fails, as on any failure, so a stopped try holds it no longer than the try's limit and grace.

### Consequences

- No try of weir-branch's call to a remote takes longer than thirty seconds and its grace, and nothing it started outlives it. A write holds its machine's lock no longer than its tries.
- Against a remote that never answers, a write fails in about 32 seconds, naming the fetch, the remote and the limit. Each replay of a write refused by a moved branch has its own tries, so a write's longest is its bound's tries, each at most a fetch and a push.
- A stopped push that landed is reported failed. The next write's fetch reads it, and a check on that tip meets what it wrote: a held key, a second standing move, a used item name.
- A fetch or a push that needs longer than thirty seconds, a large first fetch on a slow link, is stopped every time until `TRY_FOR` is raised.
- The crate runs on Unix alone, since a process group is Unix's. weir runs on no other host.
- sluice's `bounded.rs` and `runs::end_group`, `boomdrive-bounded`, and Flume's `fetch_branch` can each take `weir-bounded` in a change of its own. None moves in this one.
- This record adds core.53. The change `no-try-of-weirs-call-to-a-remote-outlasts-its-limit` builds it.

## Amendment, 2026-11-12: a try ends at its caller's halt too

Status: accepted on 2026-11-12 by agentplot-design. It is the design agent's own choice, decided by this record's one owner and by agentplot/boomdrive's core.58, which rules out the group's ending in a crate of the runner's own. It is cheap to change and holds no choice of the user's.

Sources: the question of Flume's fix place `fix--the-runners-stop-ends-its-try-in-flight--take-main-f20992b` of 2026-11-12; agentplot/boomdrive record 0042 and its amendment "the runner's stop ends its try in flight", on boomdrive's `bolt/the-runners-stop-ends-its-try-in-flight` at `069dc45c`; boomdrive's core.58 on its main at `f20992bc`; agentplot/sluice record 0109, its Ctrl-C point.

### Context

Boomdrive's runner ends the tries it has in flight when it is stopped, so a stop leaves no git running and the runner stops well within launchd's twenty seconds. Its stop sets a halt, and each try ends at its limit or its halt. That halt was built in `boomdrive-bounded`. Boomdrive's main has since run every call to a remote through `weir-bounded`, and its core.58 rules out the group's ending in a crate of its own. `weir-bounded` takes a limit alone and starts the call itself. So its caller can neither end the wait nor reach the group.

### Decision outcome

- **A halt is a try's second end.** `weir-bounded`'s run of a command takes, beside its limit, a halt its caller holds: one shared flag the caller may set from any thread. A try ends at whichever comes first. At either, its group is asked to end, given `GRACE`, then killed, by the one ending. A try begun with its halt set runs no process. A caller that never sets its halt sees its call run as before.
- **The words.** A halted call fails with words of its own, apart from the limit's. They say the call to its remote was stopped as its caller asked, after git's last line on standard error and `; ` where git wrote one. They name no tool (core.2). A halted call's standard output is never read as an answer.
- **No thread.** The wait that reads the limit reads the halt too (core.1).
- **weir-branch passes its caller's halt.** A read or a write of weir-branch's can be given its caller's halt, which reaches each of its tries. A halted push fails its write as a stopped one does, and is never replayed (core.26).
- **Not taken.**
  - The halt in a crate of a tool's own: a copy of the group's ending, which this record's one owner and boomdrive's core.58 rule out.
  - `weir-bounded` handing its caller the group to end: each caller would write an ending of its own, the drift this record's one owner prevents.
  - A halt in a static of the crate: a test runs many callers in one process, and core.1 rules out a static.

### Consequences

- Boomdrive's runner takes its stop's halt through `weir-bounded`, so its fetch, its listing and weir-branch's read and write of crew's state repository all end at its stop.
- sluice can end its tries on Ctrl-C by the same halt once it takes `weir-bounded`, which is sluice's to decide.
- core.53 is amended in the same commit.
