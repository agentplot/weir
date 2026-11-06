# Spec Delta

## MODIFIED Requirements

### Requirement: read-package reads a package at the identity it is held at
`read-package` SHALL take a package and the identity it is held at, and
give the package's one kind, as `read-kinds` reads it, with that
identity, its content hash and the declaration's sections the format
does not name; or the package refused, each field named with its reason
(92). The same package and identity SHALL give the same answer every
time, and reading SHALL change nothing (93). A package whose kind lands
with no stages names no part, and SHALL read with its `kind.toml` as
the one file it holds (51, 57 as amended at blueprints 51b6316; 89).

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
