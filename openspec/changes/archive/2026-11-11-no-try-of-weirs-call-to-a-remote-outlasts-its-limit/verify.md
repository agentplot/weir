# Verify

The work on `change/no-try-of-weirs-call-to-a-remote-outlasts-its-limit`
at 6e344db, cut from `bolt/no-try-of-weirs-call-to-a-remote-outlasts-its-limit`
at c781f73, holds this change. The repository's whole checks pass. Every
requirement and scenario of the two specs is met, each scenario by a
test of its name that passes in them. All 9 tasks are ticked, and each
one's condition holds. The change is archived.

## Checks

- `devenv shell -- check-fast`, run once at 6e344db: exit 0, 46 tasks
  completed, none failed. `root:prose` was taken from moon's cache,
  which keeps a task's outcome only when it passed and only for the
  same inputs. The other 45 ran: `root:format`, `root:toolchain`,
  `root:hooks`, `root:openspec-claude` and `root:specs`, and every
  project's `format`, `lint`, `typecheck` and `test`, for `adapters`,
  `bounded`, `branch`, `cadence`, `capture`, `delivery`, `kinds`,
  `pack`, `signals` and `weir`. Every test binary passed, 766 tests in
  all, `bounded:test`'s 13 and `branch:test`'s 62 among them (core.39).
  `root:specs` passed the 26 specs.
- `openspec validate no-try-of-weirs-call-to-a-remote-outlasts-its-limit
  --strict`: valid. Neither delta names a requirement that stands in
  `openspec/specs/`: `remote-limit` is new, and the four requirements
  `shared-branch-write` adds are named apart from its eleven.

## remote-limit

Each scenario is a test of its name in `crates/weir-bounded/tests/`,
passed in `bounded:test`.

- A command runs to its limit in a process group of its own:
  `output_kept` spawns with `process_group(0)` (`src/run.rs:55`), waits
  until the leader is reaped and `killpg(group, 0)` says the group is
  gone, and at the limit ends the group through `end_group` with
  `GRACE`. `a_process_that_never_ends_is_ended_with_its_group`,
  `a_process_asked_to_end_says_so_in_its_grace`,
  `a_process_that_will_not_end_is_killed` and
  `a_child_that_outlives_its_leader_is_ended_at_the_limit` (`output.rs`)
  hold its four scenarios, the background sleep's id asked of with
  `kill(pid, 0)` until `ESRCH`.
- A command that ends within its limit gives its exit and all it
  printed: `a_process_that_ends_gives_its_exit_and_all_it_printed` and
  `the_input_the_caller_sets_is_read`.
- What a command prints is kept to the part the caller names:
  `what_passes_the_kept_part_is_said_cut`. `output` is `output_kept`
  with `u64::MAX`.
- The crate reads a command's output with no thread: the output goes to
  two `tempfile::tempfile` files read from their start once the command
  has ended; `the_crate_starts_no_thread` (`sources.rs`).
- A group is asked to end before it is killed: `end_group`
  (`src/group.rs:18`) reaps, asks, sends `SIGTERM`, looks every 50 ms
  up to its grace, then `SIGKILL`, looking every 20 ms up to five
  seconds; each `unsafe` block carries its `SAFETY` comment.
  `a_group_already_gone_is_left_alone`.
- The limit and the grace are the family's: two constants, read from
  no setting, variable or file; `the_limit_and_grace_are_the_familys`
  (`words.rs`).
- A stopped call is named with its call, its remote and its limit:
  `a_stopped_calls_words_name_its_call_remote_and_limit` and
  `a_limit_is_said_in_seconds_or_milliseconds`.
- The crate names no tool: `the_crate_names_no_tool`, over the eight
  strings core.2's details list, as whole words in any case.

## shared-branch-write

- Each call to the remote is one try to its limit: `read_tip`'s fetch
  and `ls-remote` and `move_to`'s push run through `Git::reach`
  (`keeper.rs:423`, `:444`, `:516`), which runs git through
  `weir_bounded::output` to the branch's limit, `TRY_FOR` from both
  constructors and private to the crate. No other git call in the
  workspace reaches a remote: the cache's and the commit's calls, and
  `weir conform`'s seeding of a bare repository with no remote (core.46),
  run as before. `a_remote_that_never_answers_a_fetch` (`keeper.rs`)
  holds the scenario: it fails at `Step::Read` in under five seconds,
  naming `git fetch`, the remote's address and `300ms`, and the silent
  remote's process is gone.
- A stopped read fails the write and lets its lock go:
  `a_write_whose_remote_never_answers` (`write.rs`) holds it,
  `WriteError::Failed` at `Step::Read` ending with the stopped fetch's
  words, and the cache's `weir.lock` gone.
- A stopped push fails the write and is never replayed: `move_to`
  returns it as `Err` at `Step::Move`, so `tries` maps it to
  `WriteError::Failed` through `?` and never reaches the
  `Moved::Refused` arm's fresh read (`write.rs:335`).
  `a_remote_that_never_answers_a_push` (`keeper.rs`) holds the scenario,
  moving a commit of the empty tree from no tip.
- A stopped call's failure names its call, its remote and its limit:
  `git::stopped` (`git.rs:103`) puts git's last line on standard error,
  trimmed, and `; ` before the words, and a stopped `Ended`'s standard
  output is never read. `a_stopped_calls_failure_follows_gits_last_line`
  (`git.rs`) holds it.

The silent remote of design 10 is `src/silent.rs`, compiled for tests
alone, an `ssh://silent.invalid` address over a `core.sshCommand` that
writes its id and becomes `sleep 60`: no test reaches a network (68).

## Tasks

All 9 are ticked, and each one's condition holds in the run above.

- 1.1: `crates/weir-bounded` with `libc` and `tempfile`,
  `doctest = false`, the workspace's lints and a description citing 69,
  C.3 and record 0008; `libc = "0.2"` among the workspace's
  dependencies; the crate in `members` and in the comment's list; its
  `moon.yml` with the four tasks tagged `check-fast` and weir-cadence's
  inputs; `bounded` in `.moon/workspace.yml`; `lib.rs` with
  `#![deny(missing_docs)]` and its module doc. The three `words` tests
  pass.
- 1.2: `end_group` through `libc::killpg`; the `output` test of it
  passes.
- 1.3: `Ended`, `output` and `output_kept` as design 2 to 4 give them,
  polling every 10 ms; the seven `output` tests pass.
- 1.4: `tests/sources.rs`; its two tests pass.
- 2.1: weir-branch depends on `weir-bounded`, its moon project has
  `dependsOn: [bounded]` and the crate's files among its four tasks'
  inputs; `Branch` carries `limit`, `pub(crate)`; `Git::reach` and
  `git::stopped`; `run` and `run_program` left for the local calls. Its
  test passes.
- 2.2, 2.3 and 2.4: as above; `lib.rs`'s module doc says each try that
  reaches the remote is ended at `weir-bounded`'s limit. Their tests
  pass.
- 3.1: the README's table names `weir-bounded` after `weir-branch`;
  `root:prose` passes.

## Seen and not asked

- `wait` (`src/run.rs:100`) waits on the leader with no limit when
  `end_group` returns with it not reaped, which happens only when a
  `SIGKILL` sent to it has not ended it within five seconds. git, run by
  its own user, is ended by one; design's risk about a process that
  cannot be signalled names a process of the group, not its leader, and
  the call returns there. No scenario asks of it.
- The listing is bounded by `reach` as the fetch is, and is held by
  reading `keeper.rs:444`: the silent remote stops the fetch first, so
  no test reaches it.
- Record 0008 stays proposed, and core.53 waits on the operator's yes to
  it, as `core.md` says. The statement it cites is at blueprints ad3a1ae,
  on the branch `design/weir-no-try-of-a-call-to-a-remote-outlasts-its-limit`.
