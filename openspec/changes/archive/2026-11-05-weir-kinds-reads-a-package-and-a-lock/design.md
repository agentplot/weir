# Design

## Context

`weir-kinds` at main, f21b6b3, is `read_kinds` and `read_elaboration`
over a declaration's text (51 to 58, 81, 82). It keys each kind by any
TOML key, so `kinds."a/b@1"` reads, and it returns a stage's `standing`
among the stage's unread sections, since `standing` is not among
`STAGE_FIELDS`. `weir-capture` holds weir's sha256, `content_hash` over
bytes and `content_hash_reader` over a reader, as 64 lowercase hex
digits (section 9). The pack's clause index, `pack/clauses.toml`, was
read from the statement at blueprints 9e537bb, before 88 to 93.

flume's change a-host-declares-where-its-kinds-come-from names what it
needs of weir-kinds (its design, *What this change rests on* 1): an
identity read with its source and version each optional and spelled
back; read-package given a folder and its identity, with 89's refusals;
the content hash over a package read from disk or given as paths and
bytes, for flume's built-in kinds; read-lock; and a lock written in
C.4's form. flume record 0022 gives weir-kinds a lock's reading and
writing, and flume's core.4 rules out flume reading or spelling any of
it itself.

Across the family, sluice, flume and the runner each hash one byte
string with sha256 for keys and digests of their own; none hashes a set
of files with their paths. So weir-kinds computes the package hash, and
computes it through weir-capture's sha256 rather than a copy (core.44).

The statement as this design reads it: 53, 56, 57, 58, 88 to 93, C.4,
C.7, S10, S17 and S18, as at blueprints a11f6d5, with C.4, C.7 and S17
as amended at blueprints 361a599, which states the user's two rulings
of 2026-11-05 (record 0004).

Rules this design relies on, all in `docs/architecture/core.md`, the one
page whose area the change touches: core.1 (a read writes nothing and
keeps nothing), core.2 (no tool named; `repo` and `launch` allowed),
core.30 (one reader of kinds, packages and locks, and one speller of
locks, as this change amends it), core.31 and core.41 (a declaration
checked whole), core.32 (resolution, versions and which locks are the
reader's), core.35 (each case cites its clauses), core.39 and core.40
(the kinds project's checks), core.42 and core.43 (as this change
amends them, from record 0004), core.44 (one hash) and core.45 (a lock).
The design departs from none.

## Goals / Non-Goals

**Goals:**

- Everything flume's group 2 pins: an identity and a named kind, a
  package from disk or from paths and bytes, its content hash, a lock
  read, spelled and checked.
- The pack holding a case for each of 88 to 93, so the check counts
  88 to 91 once the index moves.

**Non-Goals:**

- Statechart kinds and the engine (94 to 99) and a deliverable `plan`
  (54 since blueprints 9c7490c): neither is a package's or a lock's, and
  each is a change of its own.
- Writing a lock's file, holding it against a second writer, or
  choosing which locks to read: the reader's (91; core.1, core.32).
- `weir check` reading a package: section 9 names `weir check
  <declaration>` alone.

## Decisions

### 1. One name check, used everywhere a name is read

A name is one or more ASCII letters, digits, `_` and `-`: the keys
`fields::key` already writes bare. One public check answers whether a
text is a name, and `read-kinds`, the identity, the named kind and a
reader checking a source's file name all call it. `read-kinds` refuses a
kind's or settings table under any other key, naming `kinds."<key>"`,
and reads nothing under it, so its fault is named once. This is the
user's ruling (record 0004); the option they did not take was any
non-empty key holding no `/` or `@`.

### 2. An identity and a named kind are types weir spells

`Identity` holds a source, a name and a version, ordered by source,
then name, then version as a number, and spells `<source>/<name>@<version>`.
`NamedKind` holds a name with its source and version each optional, the
four forms flume's launches write; it gives an `Identity` when whole.
Each parses from text and spells back as it was read. The version is
decimal digits with no leading zero, at least 1, so each identity has
one spelling and a lock never holds one kind under two (C.4 as amended).
Accepting `02` as 2 was the other way, and would let a lock hold
`acme/review@02` beside `acme/review@2`. The crate gives `repo` and
`launch` as constants, the sources C.4 names; a tool's own source is
the name its reader passes in (core.2, core.42).

### 3. A package is read from files, a folder being one way to give them

`read_package` takes a package's files, each a path within the package
and its bytes, and the identity it is held at. `read_package_dir` walks
a folder into those files and then makes the same reading, so a folder
and paths and bytes can never answer two ways (92). flume's built-in
kinds come in as paths and bytes.

The walk reads each entry's own metadata and never follows a link. A
link anywhere in the package is a problem naming its path; a regular
file is read; a folder is walked; any other entry is no part and is not
hashed, as C.4 hashes regular files alone. A path is its segments' bytes
joined by `/`, as the file system gives them. The folder's own path is
the reader's and may pass through links (C.4 as amended), so a package
under a linked `$XDG_DATA_HOME` or a nix profile reads as it does
through its own path. Given paths must be relative, with no segment
empty, `.` or `..`; the reader that gives one otherwise is told which.
A path given twice, or one beneath another file's, is refused naming
it, since no folder holds such files; taking the last of two, or
hashing both, was the other way, and would answer for files no folder
on disk can hold. An entry the walk cannot read is refused naming its
path within the package, and a folder that cannot be read is refused
naming it as given. An entry named once, a link or one not read, is
not named again where `kind.toml` or a stage's part is that entry or
lies beneath it.

### 4. A package is checked whole, as a declaration is

One reading gathers every problem and refuses the package with all of
them, as `read-kinds` does (57, 92). It checks in this order:

- every link, by its path;
- `kind.toml` at the root, a regular file, read by `read-kinds`, whose
  problems are the package's;
- exactly one kind, else a problem naming `kinds`, and no settings
  table, else a problem naming `kinds.<other>`;
- the held identity's name and version against the declaration's, a
  problem naming `kinds.<name>.version`, or `kinds.<name>` where the
  name differs, with both identities;
- every stage's `prompt` and `standing`: not absolute, no `..`
  segment, and naming a regular file of the package, `.` and empty
  segments resolved as the folder they stand in.

A package reads into `Package`: its identity, its `Kind`, its
`ContentHash` and the declaration's top-level sections.

Two of these follow 89 where C.4 said less before its amendment, and
the amendment now says both. A part the package does not hold is
refused, since 89 has a package hold every part its declaration names;
leaving a missing prompt to fail when it runs would let a lock record a
hash for a kind that cannot run. Settings given to another kind are
refused, since 89 has nothing a package names lie outside it and such
a table names a kind declared elsewhere (57); returning them with the
package would let it change another kind's agents, which that kind's
lock does not cover.

### 5. The content hash goes through weir-capture's sha256

`ContentHash` holds `sha256:` and 64 lowercase hex digits, parses from
and spells to that text, and is computed by one function over a
package's files, sorted by their path's bytes: each path, a NUL, its
length in decimal, a NUL and its bytes, streamed through
`weir_capture::content_hash_reader` (90, C.4; core.44). The package
reading and the hash call that one function. weir-kinds takes
`weir-capture` for it; a second sha256 in weir-kinds was the other way,
two encodings of the family's one hash where core.44 allows one. A
known-answer test pins the function apart from any case: `a.md` before
`a/b.md`, as their bytes order them.

### 6. A lock is read by identity, spelled in identity order, and checked

`Lock` holds its entries by `Identity`, each with its `ContentHash` and
the entry's unread keys, and the lock's own unread keys (core.30).
`read_lock` refuses whole, naming each field: text that is not TOML,
`kinds` not an array of tables, an entry with no `kind` or `hash`, a
`kind` that is no whole identity, a `hash` that is no content hash, and
an identity recorded again, named at its second entry with its first
(91, 93). It reads an identity of any source, `repo` and `launch` among
them, since the statement gives read-lock no such refusal and the
reader's own source is the reader's to know; flume refuses those at
`flume kinds lock` (its 155).

`spell_lock` gives the TOML: the lock's unread keys first, then one
`[[kinds]]` table per entry in identity order, `kind` then `hash` as
C.4 writes them, then the entry's unread keys, each value spelled as
TOML spells it (C.4 as amended). One lock always spells the same bytes,
so `flume kinds lock` run twice leaves the same file (flume core.17).
Keeping the order entries were read or added in was the other way, and
would make the text depend on the history of the writes. Each unread
key is one line, `<key> = <value>`, a table among them inline: spelling
a table under a `[header]` was the other way, and inside a `[[kinds]]`
table that header would need `kinds.` before it, which TOML's spelling
of a table alone does not give. A key the format names, put among the
unread keys by a reader, is not spelled there, since its field is
spelled in its place and TOML refuses a key given twice. The reader
writes the file, beside the lock and renamed over it as flume's design
has it, since weir writes no file of its own (core.1).

`Lock::check`, given an identity and a hash, answers recorded or not
recorded, or refuses with a mismatch naming the kind, the lock's hash
and the one given (91, S18; core.45). What not recorded means is the
reader's (91).

### 7. `standing` reads with the stage

`standing` joins `STAGE_FIELDS` as `Stage::standing`, an
`Option<String>` returned as written and refused, as `prompt` is, when
it is not a non-empty string (53, 57, C.4). The package reading checks
it as it checks a prompt. How a reader sends the text stays its own
(58; core.32).

### 8. The pack's kinds cases take three forms

A kinds `case.toml` holds exactly one of `declaration`, as today,
`package` or `lock`. A package's case gives `held`, the identity; its
`files`, a table of path to text; `links`, a table of path to target,
where the case needs one; and `lock`, a lock's text, where the case
checks against one. Its `expect` holds the kind, as a declaration case's
`kinds` entry is written, with its `identity` and `hash`, or `refused`.
A lock's case gives the lock's text, and its `expect` holds `entries`,
each `kind` and `hash`, or `refused`.

A package's kind is written under `expect.kind`, beside `identity`,
`hash` and `unread`, the declaration's sections beside its kind. A
lock's mismatch names no field of the package, so a case names it at
`lock`, its own field, the reason holding the kind and both hashes;
naming it at the identity was the other way, a field that is no field.
A lock that records no entry for the package refuses nothing, and the
case reads as it would with no lock (91). A path of a case's files or
links that is not plain within the package, or that lies beneath one of
its links, fails the case, so laying a case out writes nothing outside
its folder.

The files are written in the case, so the pack gains no file type for
treefmt to claim and no tracked link, which the pack's `build.rs`
would follow. The kinds crate's test lays each package out twice, in
two temporary folders at different depths, makes the case's links
there, reads both through `read_package_dir`, and fails where they
differ or either differs from the case (S17). The cases' hashes are
values the function computed once; the known-answer test of decision 5
is what pins the function.

### 9. The clause index moves to a11f6d5

`pack/clauses.toml` moves to the statement at blueprints a11f6d5, its
A.8 gaining 88 to 91 and its B.4 92 and 93; the C.4, C.7 and S17
amendment at 361a599 adds no number. The check then counts 88 to 91, as 67 counts A.8, and the new
cases cite them. The index stops before ee2dd50, which adds A.13 and 94
to 99: 67 counts A.13 there, and weir builds no engine yet.

## Risks / Trade-offs

- [`read-kinds` refuses a declaration it read before, where a kind's key
  is not a name] → No declaration in the family's repositories names a
  kind by a quoted key; the refusal names the key.
- [weir-kinds compiles weir-capture, and with it its JSON and schema
  crates, into every reader] → One sha256 for the family is what 90 and
  core.44 ask; the cost is build time, and nothing of it runs at a
  read.
- [A lock rewritten by its reader loses its comments] → A lock is
  written by `flume kinds lock`; a note kept in a key the format does
  not name is spelled back where it stood.
- [A package built with links, as a nix `symlinkJoin` lays one out, is
  refused] → The user's ruling (record 0004); the builder copies its
  files in, and the refusal names each link.
- [A stray file in a package, `.DS_Store` among them, moves its hash] →
  C.4 hashes every regular file, so the lock refuses a package that
  changed; the refusal names both hashes.

## Migration Plan

A reader takes this by moving its `weir-kinds` pin; flume's task 2.1
does, once this lands. Every declaration that reads today reads the
same, but for a kind under a key that is not a name, and a stage's
`standing`, which moves from its sections to its own field.
