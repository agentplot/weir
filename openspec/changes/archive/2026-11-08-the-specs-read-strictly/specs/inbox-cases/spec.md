## MODIFIED Requirements

### Requirement: An inbox case holds a starting inbox, one operation and what must come of it

An inbox case SHALL hold, in its folder under `pack/inbox/`: `case.toml` naming its clauses and one operation of the records with its arguments; `start/`, the files of the tip the operation is formed on, which may be absent; and either `tip/`, every file the tip must hold once the write is applied, or the refusal expected (C.2; C.7). A field the case format does not name SHALL fail the case's read, naming the case and the field.

#### Scenario: A case of a second standing move
- **WHEN** a case's `start/` holds a signal with a standing move, and its operation moves that signal again
- **THEN** the case expects the refusal, naming the signal and a phrase saying it holds a standing move

#### Scenario: A misspelt field
- **WHEN** an inbox case names `operaton` in place of `operation`
- **THEN** reading it fails, naming the case and `operaton`

### Requirement: The inbox cases hold the inbox's clauses

The pack SHALL hold inbox cases exercising at least: a reading written whole, a second reading refused, and a signal whose excerpt stands in neither the raw material nor the own excerpt refused (25 to 28); and a move of each of the six words with its consequence, a word not of the six refused, a second standing move refused, and a replacement naming who replaced what (29 to 31). Every clause of 25 to 43 SHALL be cited by at least one inbox case or host case.

#### Scenario: The cases named
- **WHEN** the inbox cases of the pack are read
- **THEN** each kind of case this requirement names is among them, and each passes against the signals crate

## ADDED Requirements

### Requirement: An inbox case's operation is one of the records' and its refusal names what is not taken

An inbox case's operation SHALL be one of a reading, a move, a replacement, a batch proposed, a batch decided, an item proposed, a brief written, a send, a send's outcome, or an import with its import lines in a file of the case; and the refusal it expects SHALL name each thing not taken by what it names, its field where it has one, and a phrase its reason holds (C.2; C.7).

#### Scenario: A case of a second standing move
- **WHEN** a case's `start/` holds a signal with a standing move, and its operation moves that signal again
- **THEN** the case expects the refusal, naming the signal and a phrase saying it holds a standing move

### Requirement: The inbox cases hold batches, items, briefs and sends

The pack SHALL hold inbox cases exercising at least: a batch refused whole, and a proposed batch's moves not standing (32); an item proposed under a name once used, and one nesting under another, both refused (35); and a brief rewritten in place and a send keeping the brief as handed (36, 41).

#### Scenario: The cases named
- **WHEN** the inbox cases of the pack are read
- **THEN** each kind of case this requirement names is among them, and each passes against the signals crate

### Requirement: The inbox cases hold imports, formats and the layout

The pack SHALL hold inbox cases exercising at least: import lines written as read, an imported capture held by key, a `new-territory` move read as a join, and a move of another word left unmoved (37 to 39, 63); a record naming no format refused (40); and a tip holding a file the layout does not place refused (42, 43).

#### Scenario: The cases named
- **WHEN** the inbox cases of the pack are read
- **THEN** each kind of case this requirement names is among them, and each passes against the signals crate
