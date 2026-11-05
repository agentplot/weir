# Verify

The work on `change/weir-kinds-reads-a-package-and-a-lock` at 9531d18
holds against this change: every requirement and scenario of its four
specs is met by a test that passes, every task is ticked, and every check
the tasks name passes uncached. The change is archived with this report.

## Checks

- `devenv shell -- moon run kinds:format kinds:lint kinds:typecheck
  kinds:test pack:format pack:lint pack:typecheck pack:test weir:test
  root:format --force`: 10 tasks completed, none failed, none from cache
  (task 5.1).
- `devenv shell -- check-fast`: 36 tasks completed, none failed (core.39).
- `devenv shell -- openspec validate weir-kinds-reads-a-package-and-a-lock
  --strict`: valid.

## Tasks

All 17 of groups 1 to 5 are ticked, and each one's condition holds: the
tests it names exist and pass, with the checks above.

Task 1.1 asks that every existing test stay unchanged. One did not:
`a_key_a_path_cannot_hold_bare_is_quoted` in `tests/refuse.rs` read
`kinds."a.b"` and expected problems at `kinds."a.b".version` and
`kinds."a.b".stages`. The kinds spec's *A kind's key is a name* now
requires that key refused once at `kinds."a.b"` with nothing under it
read, so the old assertion could not hold beside the spec. The test keeps
its purpose, a key quoted where a field's path cannot hold it bare, under
`kinds.bump.agents.stages."a.b"`, a key that need not be a name. The spec
wins over the task's wording, so this does not fail the task. The
existing case `every-field-of-a-kind` gains `standing`. Its expected
answer is unchanged.

## Specs

### kind-identity

- *A source's name and a kind's name are letters, digits, _ and -*:
  `check_name` is the one check (`src/name.rs`), and `fields::key`,
  `read_kinds`, `Identity` and `NamedKind` all reach it through `fault`.
  `tests/names.rs` holds both scenarios, `é` and the empty text among
  them.
- *An identity reads into its parts and spells back*: `tests/identity.rs`
  holds each of the ten refusals with the part named, and the ordering
  `acme/lint@1`, `acme/review@9`, `acme/review@10`, `beta/a@1`.
- *A kind named with its source or version left out*: the four forms,
  `identity()` given only when whole, and the seven refusals, each tested.

### kind-packages

- *read-package reads a package at the identity it is held at*:
  `a_package_that_reads`, `a_package_read_twice` (bytes and modification
  times compared before and after) and `a_section_beside_the_kind`.
- *From a folder or from paths and bytes alike*:
  `the_same_files_on_disk_and_in_memory`, `a_path_given_that_is_not_within_a_package`
  with all four paths and the field named, and
  `a_folder_reached_through_a_link`, through a linked parent and a link
  to the package itself.
- *Exactly one kind and no other*: no `kind.toml`, none or two kinds,
  `kinds.fix` settings, and a `read-kinds` refusal passed through.
- *Every part a regular file inside it*: `../prompts/code.md`,
  `/etc/code.md`, `../standing.md`, a missing part, a folder part, and
  `./`, `.` and empty segments resolved and returned as written.
- *No symbolic link*: a prompt linked inside the package refused at
  `prompts/code.md`; links named in no field, to a folder, outside and
  dangling refused each by its own path; `kind.toml` a link named once.
  The Markdown link scenario reads.
- *Held at another identity*: `kinds.review.version` holding
  `acme/review@2` and `acme/review@1`, and `kinds.review` holding
  `acme/lint@1` and `acme/review@1`.
- *C.4's content hash*: the known answer
  `sha256:9fd22069…a9cf8` for `a.md` and `a/b.md`. This report recomputed
  it apart from the crate:
  `printf 'a.md\0004\000one\na/b.md\0004\000two\n' | shasum -a 256` gives
  the same digest. Two folders with different depths, times and modes
  give one hash (S17). A byte changed, or a file added, gives another
  (S18). The hash goes through `weir_capture::content_hash_reader`, and
  weir-kinds has no sha256 of its own (core.44).

### kind-locks

- *read-lock reads every identity*: two versions, an empty lock, and
  `note` and `source_url` returned unread where they stand.
- *Refused naming the field*: `kinds[2].kind` naming `kinds[0]`; a
  `kind` that is no whole identity; a `hash` without `sha256:`, in upper
  case, or of 63 and 65 digits; text that is no TOML, named by line and
  column; `kinds = "acme/review@1"` named at `kinds`.
- *spell_lock*: `acme/lint@1` spelled before `acme/review@2`, `kind`
  before `hash`, read back equal. One lock spelled twice, and its
  spelling read and spelled again, give the same bytes.
- *A package checked against a lock*: a byte changed gives a `Mismatch`
  naming `acme/review@1` and both hashes. A lock at the hash says
  recorded; one holding only `acme/review@2` says not recorded.

### kinds

- *A kind's key is a name*: `kinds."a/b"` and `kinds."review@2"` refused
  with no kind returned. `code_review`, `review-2` and `R2` read.
- *A stage reads with its name, prompt and end*: `standing` reads as
  written, with no section left unread, and is none when left out. `""`
  and `3` are refused at each stage's `standing`.
- *The pack holds package and lock cases*: `tests/pack.rs` reads exactly
  one of `declaration`, `package` and `lock` per case with
  `deny_unknown_fields`. It lays each package out under two temporary
  folders, the second two folders deeper, with its links, reads both
  through `read_package_dir`, checks against a case's `lock`, and fails
  naming the case on any difference. The ten new cases under
  `pack/kinds/` are the ones the spec lists. 88 to 93 are each cited
  (`the_cases_cite_each_of_51_to_58_and_88_to_93`).
- *The pack holds the kind format's cases*: every case passes as it says.
  A misspelt field and a case of two forms each fail naming the case.

## The statement

`pack/clauses.toml` reads blueprints a11f6d5. This report checked it
against `design/weir/requirements.md` at that commit: A.8 holds 51 to 58
and 88 to 91, and B.4 holds 81, 82, 92 and 93. The amendment the change
reads C.4, C.7 and S17 at, 361a599, is now on blueprints' `main`. Its C.4
spells the identity, the package, the hash and the lock as the crate
reads them.

## Rules

No module of `crates/weir-kinds/src` names a tool of the family
(core.2), or reads the environment, keeps state or starts a thread
(core.1). Record 0004 stands proposed, as the user's words await their
yes on the record.
