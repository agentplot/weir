## Purpose

A lock of kinds: the identities it records, each with the content hash
of its package, read, spelled and checked against a package the same way
by every reader (91, 93, C.4).

## ADDED Requirements

### Requirement: read-lock reads every identity a lock records with its hash
`read-lock` SHALL read a lock's TOML into every identity its `kinds`
tables record, each with its content hash, by identity, and every key
the format does not name, beside the lock or an entry, returned unread
(91, 93, C.4; core.30). A lock with no `kinds` table SHALL record none.
The same text SHALL give the same answer every time, and reading SHALL
change nothing (93).

#### Scenario: Two versions of one kind
- **WHEN** a lock records `acme/review@1` and `acme/review@2`, each with its hash
- **THEN** it reads with both identities and their hashes

#### Scenario: An empty lock
- **WHEN** a lock's text is empty
- **THEN** it reads recording no identity, and is not refused

#### Scenario: Keys the format does not name
- **WHEN** a lock holds a top-level `note = "pinned by hand"` and an entry holds `source_url` beside `kind` and `hash`
- **THEN** it reads with its entries, and both keys are returned unread where they stand

### Requirement: A lock is refused naming the field
A lock SHALL be refused, naming every field that fails with its reason,
when its text is not TOML, `kinds` is not an array of tables, an entry
has no `kind` or no `hash`, a `kind` is not a whole identity, a `hash`
is not `sha256:` and 64 lowercase hex digits, or an identity is
recorded twice (91, 93, C.4; core.45).

#### Scenario: An identity recorded twice
- **WHEN** a lock records `acme/review@1` in its first and its third entry
- **THEN** it is refused naming `kinds[2].kind`, the reason naming `kinds[0]`

#### Scenario: Fields of the wrong shape
- **WHEN** an entry's `kind` is `acme/review`, or its `hash` is `4e07…` with no `sha256:`, holds upper-case hex, or is not 64 digits long
- **THEN** the lock is refused naming that entry's `kind` or `hash`

#### Scenario: A lock that is not TOML, or has no array of tables
- **WHEN** a lock's text does not parse as TOML, or holds `kinds = "acme/review@1"`
- **THEN** it is refused, saying where in the text it fails, or naming `kinds`

### Requirement: weir-kinds spells a lock that read-lock reads back
weir-kinds SHALL spell a lock as C.4's TOML: one `[[kinds]]` table per
identity, in order of source, name and version, each `kind` then
`hash`, with the keys the format does not name kept where they stood;
and `read-lock` SHALL read the spelling back as the lock it was spelled
from (C.4 as amended at blueprints 361a599; core.30). Writing the file
SHALL be the reader's.

#### Scenario: A lock spelled and read back
- **WHEN** a lock recording `acme/review@2`, then `acme/lint@1`, with a top-level key the format does not name, is spelled
- **THEN** the text holds `acme/lint@1`'s table before `acme/review@2`'s, each `kind` before `hash`, and reads back as that lock, its unread key among it

#### Scenario: The same lock spells the same
- **WHEN** one lock is spelled twice, and a lock read from that spelling is spelled again
- **THEN** all three texts are the same bytes

### Requirement: A package is checked against a lock
weir-kinds SHALL check a package's identity and content hash against a
lock: recorded at that hash, not recorded, or refused where the lock
records the identity at another hash, naming the kind and both hashes
(91, S18; core.45). Whether a kind not recorded runs, and which locks
are read, SHALL be the reader's (91; core.32).

#### Scenario: A package changed by a byte (S18)
- **WHEN** a lock records `acme/review@1` at the hash of a package, and the package held at that identity has one byte of a prompt changed
- **THEN** the check refuses it, naming `acme/review@1`, the lock's hash and the package's

#### Scenario: Recorded and not recorded
- **WHEN** a package of `acme/review@1` is checked against a lock that records it at its hash, and against one that records only `acme/review@2`
- **THEN** the first says it is recorded, and the second that it is not, refusing nothing
