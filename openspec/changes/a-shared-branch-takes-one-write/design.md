# Design

## Context

This unit is built on the capture contract as it landed and on the inbox's
records change as it stands on its branch, whose `weir-signals` crate forms
every write an inbox takes against one tip's files and checks it there
(that change's design, *Every operation forms a write and checks it at the
same tip*). That crate leaves reading and moving the branch, `recfix` and
the commit message to this one.

The statement (`agentplot/blueprints:design/weir/requirements.md` at main)
fixes what this crate must do: writing a shared branch (44 to 50), the
branch keeper's read and move (78 to 80), the git profile's five steps and
its lock (C.3), the pack (65, 67, 68; C.7), and scenarios S5, S6 and S7.
Section 9 names the crate, `weir-branch`, and says crew keeps a Python twin
of the protocol; C.3 says the pack's cases for the protocol hold both.
Record 0002, ruling 4, gives the replay bound to the writer, five unless it
says otherwise. Plan step F4 (`design/capture/plan.md`) is this unit's
half of that step; the kind format is another unit's.

crew's twin, read at crew's main (`plugin/lib/plan.py`, `land` and
`commit`, rule `state.1`): it fetches into a bare cache, forms the write
with a `make(tip)` function, checks each changed file with `recfix`,
commits through a temporary index, pushes without force, and on a refused
push fetches again: a branch that has not moved is a failure, a moved one
has the write made again, up to five replays, six tries in all. This
design keeps those steps and that count, so one set of cases holds both.

The rules it relies on, all in `docs/architecture/core.md`, the only page
of `docs/architecture/` (no area page exists, so none is touched): core.1
(no state; every write one a caller asked for), core.2 (no tool of the
family named outside the pack's cases), core.3 (no secret on a command
line), core.24 (`recfix` before a write lands), core.26 (formed on the tip
read, checked on the tip it lands on, replayed up to the writer's bound),
core.27 (one commit through a temporary index, no force, `update-ref` from
the expected value), core.28 (the lock), core.29 (what and who), core.35
and core.36 (the pack's cases cite their clauses, run with no network, and
the crate passes them), core.37 (clauses cited, never restated), core.39
and core.40 (the crate a moon project, one task per check, its tools the
pinned ones). Of the kit standard: kit:testing.1 and kit:testing.3
(layered tasks declaring their inputs) and kit:testing.5 (proved by the
checks alone).

**A departure from core.1, named.** core.1 says no crate starts a process.
This crate runs `git` and `recfix` as child processes, each ended before
the call that asked for it returns, because C.3 spells the protocol in
git's commands and `recfix` has no library form, and because 69 admits
"a write it was asked to make". core.1's details rule out a thread, timer
or server a crate starts, which this is not, and nothing here outlives
the call. The rule's wording is named for madswan-design to settle.

## Goals / Non-Goals

**Goals:**

- One function every writer of a shared branch calls, which reads the tip,
  has the caller form the write on it, checks it, commits it and lands it
  on that tip alone, replaying on a moved branch up to the writer's bound.
- The pack's protocol cases in a form any implementation can run, with the
  crate passing all of them.

**Non-Goals:**

- The inbox's checks: a caller forms the write with `weir-signals`'
  operations, which check it against the tip handed to them. This crate
  depends on no other weir crate, so it builds whether or not that crate
  has landed, and a write of anything else on a branch takes it unchanged.
- Choosing the cache's folder: the caller names it, under
  `$XDG_CACHE_HOME/<tool>/git/` as C.3 says. A crate reads no environment
  variable (core.1).
- crew's twin running the cases, and crew's twin taking the lock: crew's
  work, in crew's repository.
- The pack's check of 67 over every shape: the pack's own unit. These cases
  carry `satisfies` so that check can count them.
- Retrying a refused write on its own, deleting a file from a branch, or
  merging: none is asked for (41, 44; core.27).

## Decisions

### Modules and types

`keeper` (B.3: reading a branch and moving it), `git` (running git with a
clean environment), `commit` (the temporary index and the commit), `lock`,
`recfix`, and `write` (the protocol). Public types:

- `Branch`: the branch's name and where it lies, `Remote { cache, url }`
  for a branch on a remote reached from a bare cache, or `Local { git_dir }`
  for a branch in a repository with no remote. Both give the git directory
  commits are made in and the lock lies in: the cache, or the repository's
  own.
- `Tip`: the commit read, with `file(path)` and `files(prefix)` reading the
  text at that commit alone, so a caller can hand `weir-signals` every
  `(path, text)` under `inbox/` from one tip (43, 78, 80). A file that is
  not UTF-8 is an error naming its path.
- `Ask`: what the write does, who asked, the bound (`None` is five), the
  lock's stale age (required: see *The lock*), and the clock, a
  `&dyn Fn() -> OffsetDateTime` the caller hands (core.1).
- `write(branch, ask, form)`, where `form` is
  `FnMut(Option<&Tip>) -> Result<Files, String>`: `Files` is a map from
  path to the full new text of each file the write adds or replaces, empty
  for nothing to write; `Err` is the caller's refusal, naming why.
- `Landed { commit, on, tries, broke }` and `Nothing { at, tries, broke }`
  for the outcomes that end well, `broke` naming a stale lock broken; and
  `WriteError`: `Refused { at, reason, tries }` (the caller's check),
  `Recfix { at, path, said }`, `KeptMoving { tries, bound }`,
  `Failed { step, said }` (git, `recfix` or the lock failing), and
  `Asked { field }` (an `Ask` that names no one or nothing, or an address
  with a password).

Alternative: a `Write` trait the inbox implements. Rejected: a closure over
the tip is all the protocol needs, keeps this crate free of `weir-signals`,
and is the shape crew's `make(tip)` has.

### The protocol, step by step

1. **Ask checked.** `what` and `who` are one non-empty line each; a remote
   address with a password in it is refused naming the address with the
   password left out (50, 71; core.3, core.29). Nothing runs before this.
2. **Lock taken** in the branch's git directory (see *The lock*).
3. **Read** (C.3 step 1). For a remote: the cache is made with
   `git init --bare` when its folder is empty or missing, then
   `git fetch --quiet --no-tags --no-write-fetch-head <url>
   +refs/heads/<branch>:refs/weir/<branch>`, and the tip is that ref's
   commit; a fetch that fails is followed by
   `git ls-remote --exit-code <url> refs/heads/<branch>`, whose exit 2 means
   the branch does not exist (no tip), and anything else is `Failed`. For a
   branch with no remote: `git rev-parse --verify --quiet
   refs/heads/<branch>` in its git directory, no output meaning no tip.
4. **Form** (step 2): `form` is called with the tip. `Err` ends the write
   `Refused` at that tip; an empty map ends it `Nothing`.
5. **Check**: every path is relative, its segments non-empty, not `.`,
   `..` or `.git`, holding no NUL or newline; every path ending `.rec` has
   its new text run through `recfix --check` on standard input, its
   refusal reported with the path in place of `stdin` (46; core.24).
   `recfix` missing from the path is `Failed`.
6. **Commit** (step 3), in the git directory, through a temporary index in
   a temporary folder removed when the commit is made: `read-tree` of the
   tip (or `--empty`), each file's text through `hash-object -w --stdin`,
   all paths through one `update-index --add --index-info` at mode
   `100644`, `write-tree`, and `commit-tree` with `-p <tip>` when there is
   one (45, 47; core.27). The message is `<what>\n\nAsked-by: <who>\n`
   (50; core.29). The author and committer are named `who` with an empty
   email, so a commit needs no identity of the machine's and two machines
   form a commit the same way (core.1, core.14).
7. **Move** (step 4). For a remote: `git push --porcelain --quiet <url>
   --force-with-lease=refs/heads/<branch>:<tip> <commit>:refs/heads/<branch>`,
   the expected value empty when there was no tip, which git reads as "the
   branch must not exist". For a branch with no remote: `git update-ref -m
   <what> refs/heads/<branch> <commit> <tip>`, forty zeros for no tip,
   which git reads the same way (44, 48, 79; core.27). Success ends the
   write `Landed`, and for a remote the cache's `refs/weir/<branch>` is set
   to the commit.
8. **Replay** (step 5). A refused move is followed by a fresh read (3). A
   tip unchanged means the move failed for another reason: `Failed` with
   git's words, no replay. A moved tip has the write formed again from 4,
   every check made again there (44, 46; core.26). The write is tried at
   most `bound + 1` times; past that it is `KeptMoving` with its tries and
   bound, and nothing tries it again (record 0002, ruling 4).
9. **Lock let go**, on every end, a panic among them.

Every git command runs as `git --git-dir <dir>`, with `GIT_DIR`,
`GIT_WORK_TREE`, `GIT_INDEX_FILE` (except the temporary one),
`GIT_OBJECT_DIRECTORY`, `GIT_ALTERNATE_OBJECT_DIRECTORIES`,
`GIT_COMMON_DIR` and `GIT_QUARANTINE_PATH` removed from its environment,
so a caller inside a hook or a worktree never redirects it, and with
`GIT_TERMINAL_PROMPT=0` and `LC_ALL=C`, so it never waits on a prompt and
always speaks one language. Reaching a remote is git's own: its credential
helpers, never a secret the crate passes (core.3).

The bound counts replays, so a write is tried six times when the writer
sets none: 44 says the write "is applied again ... a bounded number of
times", core.26 "replays the write ... up to the writer's bound", and
crew's twin tries `1 + REPLAYS`. The cases pin the count, so the twin and
the crate cannot drift.

Alternatives: `git2` (libgit2) or `gix` in place of git's command line.
Rejected: C.3 spells the protocol in git's commands, `gix` has no push,
and libgit2 has no lease on push; one git, the one devenv pins, serves the
crate, its tests and crew's twin alike (core.40). A working tree with
`git commit`: ruled out by 47 and core.27.

### A branch with no remote

The repository's own git directory is where the commit is made and the
lock lies, and `update-ref` with the expected old value is the move, so
the steps are C.3's with step 4 as it says (48). No working tree is
touched: the commit is made through the temporary index alone. A
repository whose working tree has the branch checked out sees the branch
move under it, as with any `update-ref`; that is the repository's owner's
to keep apart.

Alternative: a bare cache for a local branch too, fetching from the
repository and moving it with a push. Rejected: two object stores on one
machine for one branch, and a push where C.3 names `update-ref`.

### The lock

The lock is the file `weir.lock` in the git directory, holding one
record:

```
holder: <who asked>
taken_at: <RFC 3339, from the caller's clock>
```

A writer takes it by writing that text to a file of a unique name beside
it and hard-linking that file to `weir.lock`, which fails when one stands,
so two writers taking it at once cannot both succeed; the unique file is
removed either way. A writer that finds it held reads it and waits,
looking again every fifty milliseconds, until it is gone or its
`taken_at` is older than the stale age by the caller's clock. A stale lock
is broken by renaming it to a unique name, reading the renamed file, and
removing it only when it is the lock judged stale; when it is not, another
writer has broken it and taken it since, and the writer links it back and
waits again. A lock that cannot be read as that record is `Failed`, naming
the file, for the operator to break (49; C.3). A writer lets go only a
lock whose text is its own.

The stale age has no default: 49 leaves it to the keeper ("older than the
keeper allows"), and the keeper here is the caller's `Branch`, so the
caller sets it in its `Ask`. The lock is held from before the read to
after the move, so one machine's writers sharing a git directory form
their writes on the tip the one before left, and never meet a refused
push from each other; the move from the tip read is made all the same,
so a lock never stands in for it (49; core.28).

### The pack's protocol cases

`pack/branch/<case>/case.toml`, one per folder, the shape `branch` beside
C.7's `host`, `adapter` and `cadence`. A case is data alone, read with
unknown fields refused:

```toml
satisfies = [44, 46, 79]   # the clauses it exercises, never empty (67)
remote = true              # false: a repository with no remote (48)
bound = 2                  # left out: five
stale_after_seconds = 60   # the writers' stale age

[start]                    # the branch's files before the case; left out: no branch
"inbox/moves/moves.rec" = "format: weir-move/1\n..."

[lock]                     # a lock standing before the case; left out: none
holder = "curation@two"
age_seconds = 120
let_go_after_ms = 200      # its holder lets it go then; left out: never

[[write]]                  # the writes asked for, in order, one after another
who = "operator@one"
what = "move release/acme/v1#1"
append = { path = "inbox/moves/moves.rec", text = "...", refused_when = "signal: release/acme/v1#1" }
# or: add = { path = "...", text = "..." }

[[write.meanwhile]]        # another machine's write, landed after this write's read
try = 1                    # on this try; or every_try = true
who = "curation@two"
what = "..."
append = { ... }           # or add

[[expect.write]]           # one per [[write]], in order
outcome = "refused"        # landed | nothing | refused | kept-moving
tries = 2
reason_holds = "signal: release/acme/v1#1"
broke = "curation@two"     # the holder of a stale lock it broke

[[expect.commit]]          # every commit after the start, oldest first
what = "..."
asked_by = "..."

[expect.tip]               # every file at the last tip, and no other
"inbox/moves/moves.rec" = "..."
```

`add` writes the file unless the tip holds its path, when it writes
nothing (a held key, 14); `append` writes the tip's text, or none, with
`text` after it, and is refused with `refused_when` as its reason when the
tip's text holds a line equal to it (a standing move, 29). A case's
records are short recutils, each passing `recfix`.

A runner makes everything in a temporary folder: a bare remote reached by
its path, or a bare repository standing for one with no remote, seeded
with `[start]` on the branch `inbox` through git's plumbing; the writer's
own cache beside it; and `[lock]` written into the writer's git directory
as the lock's record, its `taken_at` that many seconds before now. A
meanwhile write is landed by the runner itself with git's plumbing, as
another machine's writer would, in the same message form, before the
writer's form is called on that try; so the writer's move from the tip it
read is refused. The runner checks each outcome and its tries, each
commit's message and that its one parent is the commit before it, and the
last tip's files, and names the case and what differs (65, 68; core.35,
core.36).

The crate's runner is `tests/pack.rs`, reading every folder under
`pack/branch/` from the workspace root. The cases:

| case | what it holds | satisfies |
|---|---|---|
| `lands-on-the-tip-read` | one add on a branch with files; one commit, its parent the start, its message what and who | 44, 45, 47, 50, 78, 79 |
| `first-write-makes-the-branch` | an add with no `[start]`; the branch made at a commit with no parent | 44, 79 |
| `two-writes-from-one-tip` | two adds of different paths, the second's read before the first lands; both stand, in two commits | 44, 45, 46, 79, 80 |
| `a-held-key-at-the-new-tip` | S5: two adds of one path, the first landing meanwhile; the second writes nothing after two tries | 44, 46 |
| `a-second-standing-move` | S6: two appends naming one signal, the first meanwhile; the second refused naming it after two tries | 44, 46 |
| `kept-moving-past-the-bound` | bound 2, a meanwhile add on every try; kept moving after three tries | 44 |
| `the-bound-left-out` | no bound, a meanwhile add on every try; kept moving after six tries | 44 |
| `a-branch-with-no-remote` | S7: `remote = false`, a meanwhile add on the first try; the write lands on the second | 44, 48 |
| `recfix-refuses-a-record` | an add of a `.rec` holding a line that is no field; refused, no commit | 46 |
| `a-held-lock-let-go` | a lock taken a moment ago, let go after 200 ms; the write lands, breaking nothing | 49 |
| `a-stale-lock-broken` | a lock taken 120 s ago, stale at 60 s; the write lands, naming the holder it broke | 49 |

Every clause from 44 to 50 is cited, and `tests/pack.rs` fails when one is
not or a case cites none. Lock behaviour that needs two writers at once
(two writers on one machine; the lock let go after a refusal) is the
crate's own tests, as is a push refused by a remote that has not moved,
made with a `pre-receive` hook that refuses every push.

Alternative: cases as Rust tests alone. Rejected: C.3 says the pack's
cases hold crew's twin too, and only data both can read does that.
Alternative: recutils for the case file. Rejected: a case is nested (writes
holding meanwhile writes and texts holding records); TOML holds that
plainly and treefmt's taplo already formats it.

### The crate and its project

`crates/weir-branch`, a library, in the workspace's `members`, depending on
`time` and `tempfile` (the temporary index's folder) from
`[workspace.dependencies]`, with `serde` and `toml` for its tests. It is
moon project `branch` in `.moon/workspace.yml`, its `moon.yml` as the
capture crate's: `language: 'rust'` and tasks `format`
(`cargo fmt -p weir-branch --check`), `lint`
(`cargo clippy -p weir-branch --all-targets -- -D warnings`), `typecheck`
(`cargo check -p weir-branch --all-targets`) and `test`
(`cargo test -p weir-branch`), each tagged `check-fast`, run from the
workspace root, and declaring `/crates/weir-branch/**/*`,
`/pack/branch/**/*`, `/Cargo.toml`, `/Cargo.lock`, `/devenv.nix`,
`/devenv.lock` and `/.prototools` as inputs (core.39, core.40;
kit:testing.1, kit:testing.3). Its tests run git and `recfix` from
`devenv.nix`, and fail when either is not on the path. A test fails when
a file under the crate's `src/` holds a string core.2 rules out.
`Cargo.toml`'s comment on the planned crates names the branch crate
landed.

## Risks / Trade-offs

- [The inbox's records change lands its crate in parallel, and both
  changes edit `Cargo.toml`, `Cargo.lock` and `.moon/workspace.yml`] →
  Each adds one member and one project; the second to merge takes the
  other's lines beside its own.
- [The pack's check of 67 does not yet know a `branch` shape, and C.7 names
  three] → The cases carry `satisfies` as every case does, and the crate's
  runner checks this shape's citations; C.7's list is named for
  madswan-design.
- [A local branch checked out in a working tree moves under it] → 47 is
  kept, since no working copy is touched; the working tree then differs
  from its branch, as after any `update-ref`.
- [A writer waits up to the stale age for a lock whose holder died] → The
  caller sets that age; the operator can break the lock sooner by removing
  the file.
- [Waiting and stale checks use the caller's clock, the sleeps between
  looks the thread's] → A clock that stands still makes a held lock never
  stale; the caller's clock is the machine's in use, and a test's own.
- [git's words for a refused push differ by version] → The protocol never
  reads them: a refused move is told from a moved branch by a fresh read.

## Open Questions

None.
