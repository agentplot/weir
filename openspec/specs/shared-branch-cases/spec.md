# shared-branch-cases Specification

## Purpose
The conformance pack's cases for the shared-branch write protocol: fixed
cases any implementation of the protocol runs, weir's crate and crew's
Python twin alike, each citing the clauses it exercises (statement 44 to
50, 65, 67, 68, 78 to 80; C.3, C.7).

## Requirements

### Requirement: The protocol's cases lie in the pack, one folder each

The pack SHALL hold the protocol's cases at `pack/branch/<case>/case.toml`, one case per folder, each naming the clauses it exercises in `satisfies` (67; C.7; core.35). Every clause of 44 to 50 SHALL be cited by at least one case.

#### Scenario: Every protocol clause is cited
- **WHEN** the cases under `pack/branch/` are read
- **THEN** each names a non-empty `satisfies` list, and every number from 44 to 50 appears in some case's list

#### Scenario: A case that cites nothing
- **WHEN** a case under `pack/branch/` has no `satisfies`
- **THEN** reading the cases fails naming that case

### Requirement: A case says the writes, their order and what must come of them

A case SHALL give, in data alone, whether the branch has a remote, its files before the case, the writes asked for in order with who asked and what each does, the writes another machine lands between a write's read and its move, the lock standing before the case, the bound, and what must come out: each write's outcome and tries, each commit's message, and the files at the last tip (65).

#### Scenario: A case read
- **WHEN** a case under `pack/branch/` is read
- **THEN** every field it holds is one the case format names, and a field it does not name fails the read naming the case and the field

### Requirement: A case's write is formed by a rule any implementation can follow

A case's write SHALL be one of two forms, each with the check it stands for: adding a file, which writes nothing when the tip holds that path (a held key, 14); or appending text to a file, which is refused naming the line when the file at the tip holds a line the case names (a standing move, 29). Both SHALL be formed from the tip's files alone (46).

#### Scenario: An add over a held path
- **WHEN** a case's write adds a path the tip already holds
- **THEN** the write writes nothing, and the case expects it so

#### Scenario: An append meeting its line
- **WHEN** a case's write appends to a file whose text at the tip holds the line the case names
- **THEN** the write is refused naming that line

### Requirement: The cases run with no network, from fixed inputs

Every case SHALL run against repositories made for it on the machine that runs it, a remote being a bare repository reached by its path, with no network and no source, and SHALL give the same verdict on every machine (68; core.35).

#### Scenario: A remote case
- **WHEN** a case with a remote is run
- **THEN** its remote is a bare repository in a temporary folder, and nothing is fetched from or pushed to any other place

### Requirement: The crate passes every case

The crate's tests SHALL run every case under `pack/branch/` against the crate and fail naming each case whose outcome, tries, commits or last tip differ from what it expects (65, 87; core.36).

#### Scenario: A case the crate fails
- **WHEN** a case expects a write tried three times and the crate tries it twice
- **THEN** the crate's tests fail naming that case and the tries expected and made

### Requirement: The cases hold the protocol's scenarios

The pack SHALL hold at least these cases: a write landing on the tip read; the first write making a branch; two writes from one tip both landing in turn; a held key met again at the new tip (S5); a second standing move refused at the new tip (S6); a branch that keeps moving, refused after a bound of two and after the bound left out; a branch with no remote (S7); a record `recfix` refuses; a lock let go while a writer waits; and a stale lock broken (44 to 50; S5, S6, S7).

#### Scenario: The cases named
- **WHEN** the folders under `pack/branch/` are listed
- **THEN** each of the cases this requirement names is among them, and each passes against the crate
