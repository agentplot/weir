# Verify

The change a-bounded-call-ends-at-its-callers-halt, on
`change/a-bounded-call-ends-at-its-callers-halt` at 6213a8e, holds: the
whole checks pass, every task is ticked, and every requirement and
scenario of its two delta specs is met by the code and a test of the
scenario's name.

## The whole checks

`devenv shell -- check-fast` (`moon run :#check-fast`), run once at
6213a8e: 46 tasks completed, none failed, 32 of them from moon's cache.
Ran and passed: `root:format`, `root:toolchain`, and `format`, `lint`,
`typecheck` and `test` of `bounded`, `branch` and `weir`. Taken from the
cache, their inputs unchanged: `root:hooks`, `root:openspec-claude`,
`root:prose`, `root:specs` (`openspec validate --strict`), and the four
checks of `adapters`, `cadence`, `capture`, `delivery`, `kinds`, `pack`
and `signals`.

## Tasks

All 15 tasks of `tasks.md` are ticked, 1.1 to 1.5, 2.1 to 2.6 and 3.1,
and each test a task names is in the tree under that name.

## remote-limit

| Requirement | Scenario | Where it holds |
|---|---|---|
| A command runs to its limit in a process group of its own (modified) | A process that never ends is ended with its group; A process asked to end says so in its grace; A process that will not end is killed; A child that outlives its leader is ended at the limit | `run.rs`'s `wait` breaks with `Stopped::AtLimit`; `tests/output.rs` passes `&Halt::default()` and reads `Some(Stopped::AtLimit)` in each of the four |
| A command ends at its caller's halt too | A halt ends a command with its group | `wait` reads the halt after the leader and the limit, breaks with `Stopped::Halted`, the group ended by `end_group` with `GRACE`; `tests/halt.rs` sets the halt from a scoped thread of the test's |
| A command begun with its halt set runs no process | A command begun with its halt set runs no process | `output_kept` returns before any file or spawn, `status: None`, nothing printed, nothing cut; `tests/halt.rs` |
| The first of the limit and the halt ends a command | A halt after the limit changes nothing | the cause `wait` breaks with is kept through the group's ending; `tests/halt.rs` |
| A halt is one flag its caller holds | A halt is one flag its clones share; The crate holds no static | `src/halt.rs`, `Arc<AtomicBool>`, `SeqCst`, equality by `Arc::ptr_eq`; `tests/halt.rs`; `tests/sources.rs`'s `the_crate_holds_no_static` |
| A halted call is named with its call and its remote | A halted call's words name its call and remote | `lib.rs`'s `halted`; `tests/words.rs` |

The modified requirement keeps all four scenarios the main spec holds,
so archiving drops none.

## shared-branch-write

| Requirement | Scenario | Where it holds |
|---|---|---|
| A read or a write ends its tries at its caller's halt | A halt ends a fetch its remote never answers; A read begun with its halt set reaches no remote | `Branch::with_halt`; `read_tip` passes `&branch.halt` to the fetch and the `ls-remote`; `keeper.rs`'s tests, the second reading `was_called()` false |
| A halted read fails the write and lets its lock go | A write halted while it reads lets its lock go | `write.rs`'s test: `WriteError::Failed` at `Step::Read` ending with the halted fetch's words, `weir.lock` gone |
| A halted push fails the write and is never replayed | A halt ends a push its remote never answers | `move_to` passes the halt to the push and maps the `Err` to `Step::Move`; `write::tries` returns it at once by `map_err(failed)?`, before any refusal is read; `keeper.rs`'s test fails on `Moved::Refused` |
| A branch with no remote takes no halt | A branch with no remote takes no halt | `Place::Local`'s calls take no halt; `tests/write.rs`, through `Branch::local(..).with_halt(halt)` |
| A halted call's failure names its call and its remote | A halted call's failure follows git's last line | `git.rs`'s `halted` and `stopped` share `after_last_line`; `reach` maps `Some(Stopped::Halted)` to it and reads no standard output; `git.rs`'s test |

## The rules

- core.1: the halt is read by the caller's wait; the crate starts no
  thread and holds no static, which `tests/sources.rs` checks. The
  threads that set a halt mid-call are the tests', in `tests/halt.rs`
  and in weir-branch's `src/silent.rs`, which is `#[cfg(test)]`.
- core.2: `halted`'s words name no tool, and `tests/sources.rs` and the
  `weir` crate's `names` test pass.
- core.26: a halted push is `Err` at the move, never a refusal, and is
  not replayed.
- core.28: the writers' lock is let go on a halted read.
- core.53: the limit, the halt, the group's ending and both stopped
  calls' words are `weir-bounded`'s alone; weir-branch re-exports its
  `Halt` and calls its `halted`.
- `Branch::remote`, `Branch::local`, `read`, `write` and `Ask` keep their
  signatures, so the `weir` crate builds and passes unchanged.

## Findings

None.
