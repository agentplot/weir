# Proposal

## Why

The statement names a kind by its identity, carries one kind at one
version as a package with a content hash, and lets a reader run a kind
from outside only where a lock records that hash (88 to 93, C.4, S17,
S18). weir-kinds at main, f21b6b3, reads a declaration and none of
these. flume's change a-host-declares-where-its-kinds-come-from (flume
record 0040, accepted on 2026-11-05) stops at its group 2 until
weir-kinds reads an identity, a package with its content hash and a
lock, and writes a lock: flume may read none of these with code of its
own (flume core.4, core.22; weir core.30).

## What Changes

- An identity, `<source>/<name>@<version>`, is read into its parts and
  spelled back, and a kind's name with its source or its version left
  out, as flume's launch forms and package folders write it, is read
  into its parts (88, C.4).
- A kind's name and a source's name are each one or more ASCII letters,
  digits, `_` and `-`, as the user ruled on 2026-11-05 (record 0004).
  **BREAKING**: `read-kinds` refuses a kind's table or a table of
  settings under any other key, naming it (57, C.4).
- A stage's `standing` reads with the stage, as written, in place of
  being returned unread (53, C.4).
- `read-package`: given a package, from a folder or as paths and
  bytes, and the identity it is held at, its one kind, the identity and
  its content hash; or the package refused naming the field (89, 90,
  92). A package declares exactly one kind, gives settings to no other,
  names each part as a regular file inside it, and holds no symbolic
  link, which is refused naming the link's file, as the user ruled
  (record 0004).
- The content hash is C.4's, one function over every regular file of
  the package, computed through weir-capture's sha256 (90; core.44).
- `read-lock`: every identity a lock records with its hash, or the lock
  refused naming the field (91, 93). weir-kinds spells a lock that
  `read-lock` reads back as it was, and checks a package against a lock,
  refusing a hash that is not the lock's, naming the kind and both
  hashes (91, S18). The reader writes the file.
- The pack's kinds cases hold packages and locks, and its clause index
  moves to the statement at blueprints a11f6d5, which holds 88 to 93
  (C.7, 67).

## Capabilities

### New Capabilities

- `kind-identity`: a kind's identity, and a kind's name with its
  source or version left out, read into parts and spelled back, and the
  names a source and a kind may take.
- `kind-packages`: a kind's package read at the identity it is held
  at, its refusals, and its content hash.
- `kind-locks`: a lock read and spelled, and a package checked against
  it.

### Modified Capabilities

- `kinds`: a kind's or settings table under a key that is not a name is
  refused; a stage's `standing` reads with the stage; the pack's kinds
  case format and cases hold packages and locks.

## Impact

- `crates/weir-kinds`: new public types for an identity, a named kind,
  a package, a content hash and a lock, with `read_package`,
  `read_lock`, the lock's spelling and its check; `Stage::standing`. It
  takes `weir-capture` for the sha256 and `tempfile` for its tests.
  `read-kinds` refuses a declaration it read before where a kind's key
  is not a name. A struct literal of `Stage` outside the crate names
  the new field.
- `crates/weir-kinds/moon.yml`: depends on `capture`, and every task
  reads `/crates/weir-capture/**/*`.
- `pack/kinds/`: package and lock cases; `crates/weir-kinds/tests/pack.rs`
  reads them. `pack/clauses.toml` moves to a11f6d5.
- `docs/adr/0004` and core.30, core.42 and core.43, written with this
  change; the statement's C.4, C.7 and S17 as amended at blueprints
  361a599, on the branch `design/weir-a-kinds-name-and-a-package-without-links`.
- flume moves its `weir-kinds` pin to the revision that lands this
  (flume's task 2.1).
