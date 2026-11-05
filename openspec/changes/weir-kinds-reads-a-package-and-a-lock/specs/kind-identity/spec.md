## Purpose

How a kind is named across the family: its identity, its source, its
name and its version, read from the text a reader is given and spelled
back one way (88, C.4).

## ADDED Requirements

### Requirement: A source's name and a kind's name are letters, digits, _ and -
A source's name and a kind's name SHALL each be one or more ASCII
letters, digits, `_` and `-`, and any other text SHALL be refused as a
name, saying why (C.4 as amended at blueprints 361a599; record 0004).
The check SHALL be the one every read of an identity, a named kind and a
declaration's kind uses, and a reader SHALL be able to ask it of a
source's name alone.

#### Scenario: Names that are names
- **WHEN** `acme`, `review`, `code_review`, `review-2` and `R2` are checked
- **THEN** each is a name

#### Scenario: Text that is not a name
- **WHEN** `ac me`, `acme.x`, `a/b`, `review@2`, `é`, and the empty text are checked
- **THEN** each is refused, the reason saying a name is one or more ASCII letters, digits, `_` and `-`

### Requirement: An identity reads into its parts and spells back
An identity SHALL read from `<source>/<name>@<version>` into its source,
its name and its version, the version a whole number of at least 1 in
decimal digits with no leading zero, and SHALL spell back as the text it
was read from (88, C.4). Text with any part missing, a part that is not
a name, or a version of another shape SHALL be refused as an identity,
saying which part fails.

#### Scenario: An identity read and spelled
- **WHEN** `acme/review@1` and `flume/change@12` are read as identities
- **THEN** they read as the source `acme`, the name `review` and the version 1, and the source `flume`, the name `change` and the version 12, and each spells back as it was written

#### Scenario: Text that is no identity
- **WHEN** `acme/review`, `review@1`, `acme/review@0`, `acme/review@02`, `acme/review@+1`, `acme/review@1.5`, `a/b/c@1`, `/review@1`, `acme/@1` or `acme/re view@1` is read as an identity
- **THEN** it is refused, the reason naming the part that fails

#### Scenario: Identities in order
- **WHEN** `acme/review@10`, `acme/review@9`, `acme/lint@1` and `beta/a@1` are put in order
- **THEN** they stand `acme/lint@1`, `acme/review@9`, `acme/review@10`, `beta/a@1`: by source, then name, then version as a number

### Requirement: A kind named with its source or version left out reads into its parts
A kind's name as a reader is given it, `<name>`, `<name>@<version>`,
`<source>/<name>` or `<source>/<name>@<version>`, SHALL read into its
name, and its source and its version each where given, the parts
checked as an identity's are (C.4 as amended at blueprints 361a599). It
SHALL spell back as it was written. What a name with a part left out
names, and which version it takes, SHALL be the reader's (58; core.32).

#### Scenario: The four forms
- **WHEN** `review`, `review@2`, `acme/review` and `acme/review@2` are read as named kinds
- **THEN** they read as the name `review` with no source and no version; with the version 2; with the source `acme`; and with both, and each spells back as it was written

#### Scenario: A named kind left whole
- **WHEN** `acme/review@2` is read as a named kind
- **THEN** it gives the identity `acme/review@2`, and `acme/review` gives none, its version left out

#### Scenario: Text that names no kind
- **WHEN** `review@`, `@2`, `acme/`, `/review`, `a/b/c`, `review@0` or `re view` is read as a named kind
- **THEN** it is refused, the reason naming the part that fails
