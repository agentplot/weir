## MODIFIED Requirements

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

#### Scenario: A package of a kind that lands with no stages
- **WHEN** a folder holds only `kind.toml`, declaring `train` with `lands = true` and `stages = []`, and is read held at `acme/train@1`, and the same file is given as a path and its bytes
- **THEN** both give `train` landing with no stages, the identity `acme/train@1`, and one content hash, the hash of that one file

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

#### Scenario: A package of a kind that names its fix
- **WHEN** a folder holds only `kind.toml`, declaring `train` with `lands = true`, `stages = []`, `landing_fix = "acme/fix@1"` and `max_landing_fixes = 2`, and is read held at `acme/train@1`, and the same file is given as a path and its bytes
- **THEN** both give `train` landing with no stages, its `landing_fix` spelling `acme/fix@1` and its `max_landing_fixes` 2, the identity `acme/train@1`, and one content hash, the hash of that one file

## ADDED Requirements

### Requirement: A package of a kind that lands with no stages reads with its kind.toml alone
A package whose kind lands with no stages names no part, and SHALL read
with its `kind.toml` as the one file it holds (51, 57 as amended at
blueprints 51b6316; 89).

#### Scenario: A package of a kind that lands with no stages
- **WHEN** a folder holds only `kind.toml`, declaring `train` with `lands = true` and `stages = []`, and is read held at `acme/train@1`, and the same file is given as a path and its bytes
- **THEN** both give `train` landing with no stages, the identity `acme/train@1`, and one content hash, the hash of that one file

### Requirement: A package names its landing fix by an identity and no part of it
A kind that lands with no stages and names its landing fix by
`landing_fix` names that kind by an identity and no part of the package.
The package SHALL read with its `landing_fix` as written, whether or not
it holds the kind named. Its content hash SHALL be C.4's over its files,
`kind.toml` with the identity in it among them, and no file of the kind
named (89 and C.4 as amended at blueprints 8eec6dd; 90; record 0005).

#### Scenario: A package of a kind that names its fix
- **WHEN** a folder holds only `kind.toml`, declaring `train` with `lands = true`, `stages = []`, `landing_fix = "acme/fix@1"` and `max_landing_fixes = 2`, and is read held at `acme/train@1`, and the same file is given as a path and its bytes
- **THEN** both give `train` landing with no stages, its `landing_fix` spelling `acme/fix@1` and its `max_landing_fixes` 2, the identity `acme/train@1`, and one content hash, the hash of that one file
