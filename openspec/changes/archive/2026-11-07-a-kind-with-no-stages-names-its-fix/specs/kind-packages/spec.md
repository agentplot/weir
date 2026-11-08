# Spec Delta

## MODIFIED Requirements

### Requirement: A package declares exactly one kind and names no other
A package's `kind.toml`, at its root, SHALL declare exactly one kind,
and SHALL give settings to no kind it does not declare, since such a
table names a kind outside the package (89, 57). A package with no
`kind.toml`, a `kind.toml` that is not a file or that `read-kinds`
refuses, one declaring no kind or several, or one giving settings to
another kind SHALL be refused naming the field. A kind that lands with
no stages and names by `fix` the kind its fixes run as names that kind
by a name and no part of the package. The package SHALL read with its
`fix` as written, whether or not it holds the kind named. Its content
hash SHALL be C.4's over its files, `kind.toml` with the name in it
among them, and no file of the kind named (89 and C.4 as amended at
blueprints 31f9f1c; 90).

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
- **WHEN** a folder holds only `kind.toml`, declaring `train` with `lands = true`, `stages = []`, `fix = "fix"` and `max_fixes = 2`, and is read held at `acme/train@1`, and the same file is given as a path and its bytes
- **THEN** both give `train` landing with no stages, its `fix` spelling `fix` and its `max_fixes` 2, the identity `acme/train@1`, and one content hash, the hash of that one file
