# kind-packages Specification

## Purpose
A kind's package: one kind at one version with every part it names, as
it travels between places, read at the identity it is held at and
hashed the same on every machine (89, 90, 92, C.4).

## Requirements

### Requirement: read-package reads a package at the identity it is held at
`read-package` SHALL take a package and the identity it is held at, and
give the package's one kind, as `read-kinds` reads it, with that
identity, its content hash and the declaration's sections the format
does not name; or the package refused, each field named with its reason
(92). The same package and identity SHALL give the same answer every
time, and reading SHALL change nothing (93).

#### Scenario: A package that reads
- **WHEN** a folder holds `kind.toml` declaring `review` at version 1 with one stage whose prompt is `prompts/review.md`, and `prompts/review.md`, and is read held at `acme/review@1`
- **THEN** it gives `review`'s stage, the identity `acme/review@1` and the folder's content hash

#### Scenario: A package read twice
- **WHEN** that package is read twice
- **THEN** both reads give the same kind, identity and hash, and every file of the folder is as it was, its bytes and its modification time

#### Scenario: A section beside the kind
- **WHEN** a package's `kind.toml` holds a `[profiles]` table beside its kind
- **THEN** the package reads, with `profiles` returned unread (56)

### Requirement: A package is read from a folder or from paths and bytes alike
A package SHALL be read from a folder on disk, or from its files given
as paths within the package and their bytes, as a tool's built-in kinds
are held, and the same files SHALL give the same answer either way
(90, 92). A path given SHALL be relative, with `/` between its
segments and no segment empty, `.` or `..`; any other SHALL be refused
naming it. The path to a folder SHALL be the reader's, and MAY pass
through symbolic links (C.4 as amended at blueprints 361a599).

#### Scenario: The same files on disk and in memory
- **WHEN** a package's files are read from a folder, and the same paths and bytes are given in memory, each at `flume/change@2`
- **THEN** both give the same kind, identity and content hash

#### Scenario: A path given that is not within a package
- **WHEN** files are given at `/kind.toml`, `./kind.toml`, `prompts//code.md` or `../kind.toml`
- **THEN** the package is refused naming that path

#### Scenario: A folder reached through a link
- **WHEN** a package's folder is read through a path one of whose folders is a symbolic link
- **THEN** it reads as it does through the folder's own path, with the same hash

### Requirement: A package declares exactly one kind and names no other
A package's `kind.toml`, at its root, SHALL declare exactly one kind,
and SHALL give settings to no kind it does not declare, since such a
table names a kind outside the package (89, 57). A package with no
`kind.toml`, a `kind.toml` that is not a file or that `read-kinds`
refuses, one declaring no kind or several, or one giving settings to
another kind SHALL be refused naming the field.

#### Scenario: No kind.toml
- **WHEN** a folder holds `prompts/review.md` and no `kind.toml`
- **THEN** the package is refused naming `kind.toml`

#### Scenario: No kind, or two
- **WHEN** a package's `kind.toml` declares no kind, or declares `review` and `lint`
- **THEN** the package is refused naming `kinds`, the reason saying a package declares exactly one kind

#### Scenario: Settings for another kind
- **WHEN** a package's `kind.toml` declares `review` and holds `[kinds.fix.agents]` with `model = "opus"`
- **THEN** the package is refused naming `kinds.fix`

#### Scenario: A declaration read-kinds refuses
- **WHEN** a package's `kind.toml` declares a stage with two deliverables
- **THEN** the package is refused naming that stage's `deliverables`, as `read-kinds` names it

### Requirement: Every part a package names is a regular file inside it
Every `prompt` and `standing` path a package's `kind.toml` gives SHALL
be relative to that file, hold no `..` segment, not be absolute, and
name a regular file of the package; a path that does not SHALL be
refused naming the field (89; C.4 as amended at blueprints 361a599;
S17). A `.` segment or an empty one SHALL name the folder it stands in.

#### Scenario: A prompt outside the package (S17)
- **WHEN** a package's `kind.toml` names its stage's prompt `../prompts/code.md`
- **THEN** the package is refused naming `kinds.review.stages[0].prompt`

#### Scenario: Absolute paths and standing instructions
- **WHEN** a stage's prompt is `/etc/code.md`, or its `standing` is `../standing.md`
- **THEN** the package is refused naming that stage's `prompt`, or its `standing`

#### Scenario: A part the package does not hold
- **WHEN** a stage's prompt names `prompts/code.md` and the package holds no such file, or holds `prompts/code.md` as a folder
- **THEN** the package is refused naming that stage's `prompt`

#### Scenario: A path with a dot segment
- **WHEN** a stage's prompt is `./prompts/code.md` and the package holds `prompts/code.md`
- **THEN** the package reads, the prompt returned as written

### Requirement: A package holds no symbolic link
A package SHALL hold no symbolic link, whether it leads to a file inside
the package, a folder, a file outside or nowhere; one that does SHALL be
refused, naming the link's file by its path within the package (89; C.4
as amended at blueprints 361a599; S17; record 0004). A part MAY name
another by a relative path, as a Markdown link does: weir parses no part
but `kind.toml`.

#### Scenario: A prompt that is a link inside the package (S17)
- **WHEN** a package's `prompts/code.md` is a symbolic link to `prompts/other.md`, a regular file of the same package
- **THEN** the package is refused naming `prompts/code.md`

#### Scenario: Links of every other kind
- **WHEN** a package holds a link named in no field, a link to a folder, a link to a file outside the package, or a link to nothing, `kind.toml` itself a link among them
- **THEN** the package is refused naming each link's file

#### Scenario: A Markdown link between parts
- **WHEN** a package's prompt holds the Markdown link `[the checklist](../checklist.md)`
- **THEN** the package reads, since no part's text is parsed but `kind.toml`'s

### Requirement: A package held at another identity is refused naming both
A package read at an identity whose name or version is not its
declaration's SHALL be refused naming the kind's field, with both the
identity it was held at and its declaration's, the source the one it
was held at (89, 92; core.43).

#### Scenario: Held at another version
- **WHEN** a package declaring `review` at version 1 is read held at `acme/review@2`
- **THEN** it is refused naming `kinds.review.version`, the reason holding `acme/review@2` and `acme/review@1`

#### Scenario: Held under another name
- **WHEN** a package declaring `review` is read held at `acme/lint@1`
- **THEN** it is refused naming `kinds.review`, the reason holding `acme/lint@1` and `acme/review@1`

### Requirement: A package's content hash is C.4's, over every regular file
A package's content hash SHALL be `sha256:` and the lowercase hex sha256
of the concatenation, in byte order of the whole paths, of each regular
file's path relative to the package's root, a NUL byte, its length in
bytes in decimal, a NUL byte and its bytes (90, C.4; core.44). Every
regular file SHALL be hashed, one no field names among them, and
nothing else of a file or of where the package lies SHALL reach it.

#### Scenario: A known package's hash
- **WHEN** a package's files are `a.md` holding `one` and a newline, and `a/b.md` holding `two` and a newline
- **THEN** its content hash is `sha256:9fd22069b62b656b446d5cc2b60f897a2640d2d005bab6cfb39d6289ca8a9cf8`, `a.md` hashed before `a/b.md` since `.` comes before `/`

#### Scenario: Two machines, two folders (S17)
- **WHEN** one package is copied to two folders at different paths, its files written at different times and with different modes
- **THEN** both have the same content hash

#### Scenario: One byte changed (S18)
- **WHEN** one byte of one prompt of a package is changed, or a file no field names is added to it
- **THEN** its content hash differs from the one before
