# adapter-checks Specification

## Purpose
Holding an adapter to the contract: the checks an adapter's lines must
pass, the meeting checks a meeting adapter passes, and the adapter
shape's cases that fix what each check says, the built-in adapters'
among them (statement 1 to 12, 61, 62, 64, 65, 66; C.1, C.6, C.7).

## Requirements

### Requirement: The adapter checks hold a run's lines to the contract

The adapter checks SHALL be given a run's lines, the raw place it was handed and whether it exited reporting success, and SHALL name every breach, each with its line where it has one, the clause it breaks and why: a line that is not exactly one of an offer, a cursor and a completion (1; core.5); an offer without a key, a source or an event time, or whose key the key rule refuses (2, 4, 13); an event time without its offset (5; C.1); two offers in one run under one key that differ in their source or event time, two events sharing a key (3); raw material named outside the raw place, or not in it after the run (6); an own excerpt lacking its text or any field (7); a run that exited reporting success with no completion, or with lines after it, or with two (9); a failure with no `what` or no `why` (9); and an own excerpt whose text ends with the host's closing line, which only a host writes (12, 20). Lines with no breach SHALL pass, and a breach SHALL stop no other line being checked (10).

#### Scenario: A run that keeps the contract
- **WHEN** the checks are given three offers with their raw files in the raw place, a cursor and a completion, and success
- **THEN** they name no breach

#### Scenario: Two offers under one key
- **WHEN** lines 2 and 5 offer `release/acme/v1`, with two event times
- **THEN** the checks name line 5 breaking 3, and check every other line

#### Scenario: Success with no completion
- **WHEN** a run's lines hold no completion and it exited reporting success
- **THEN** the checks name a breach of 9 with no line

#### Scenario: An adapter that cut its own excerpt
- **WHEN** an offer's own excerpt ends with `[cut: 12 lines and 4096 bytes left out, banked whole with the capture]`
- **THEN** the checks name that line breaking 12

### Requirement: The meeting checks hold a meeting adapter to 66

The meeting checks SHALL be given what the adapter checks are, and, from the adapter's own fixture, each meeting's start by its key and how many meetings it held too short to hold anything; and SHALL name every breach of 66: an offer whose key is not `meeting/<service>/<the meeting's own identity>`; whose source is not `meeting`; whose event time is not its meeting's start; that names fewer than two raw files, the transcript as served and a plain copy; or that carries an own excerpt; and a completion whose count is not the offers, the failures and the meetings left out together. A run that passes the meeting checks SHALL also pass the adapter checks.

#### Scenario: A meeting adapter that keeps 66
- **WHEN** the meeting checks are given two offers keyed `meeting/wispr/m-1` and `meeting/wispr/m-2`, each at its start with two raw files and no excerpt, and a completion listing three with one meeting too short
- **THEN** they name no breach

#### Scenario: A short meeting not counted
- **WHEN** the fixture held one meeting too short, the adapter offers two, and its completion lists two
- **THEN** the meeting checks name the completion breaking 66

#### Scenario: An event time that is not the start
- **WHEN** an offer's event time is when its transcript was saved, not its meeting's start
- **THEN** the meeting checks name that line breaking 66

### Requirement: An adapter case fixes lines in and the verdict out

An adapter case SHALL hold its lines (`lines.jsonl`), its raw place where it has one, whether the run exited reporting success, whether it is checked as a meeting adapter, with the fixture's starts and short meetings when it is, and the verdict the checks must give: that the lines pass, or each breach by its line, its clause and a phrase its reason holds. The pack's tests SHALL run every adapter case of this kind and fail naming each case whose verdict differs (65).

#### Scenario: A case whose lines must fail
- **WHEN** a case expects line 2 to break 4 with a reason holding `captures`, and its line 2 offers `captures/x`
- **THEN** the checks give that breach and the case passes

#### Scenario: A case the checks answer otherwise
- **WHEN** a case expects its lines to pass and the checks name a breach
- **THEN** the pack's tests fail, naming the case and the breach

### Requirement: The built-in adapters pass the checks on the pack's cases

An adapter case MAY instead name a built-in adapter, the folder drop or the meeting file, with its input, a folder or a file in the case's folder, and the lines it must print. weir's adapters' tests SHALL run each such case: the adapter's lines SHALL be the lines the case expects, and SHALL pass the adapter checks (61, 62, 64; core.34, core.36).

#### Scenario: A file dropped twice (S14)
- **WHEN** a case gives the folder drop a folder holding one file under two names
- **THEN** the folder drop prints one offer keyed by the file's sha256 and a completion, as the case expects, and the lines pass the adapter checks

#### Scenario: A meeting file with its start
- **WHEN** a case gives the meeting file a transcript whose front matter says `start: 2026-10-20T15:00:00-04:00`
- **THEN** it prints one offer keyed `meeting/file/<sha256>` at that start, as the case expects, and the lines pass the adapter checks

### Requirement: The adapter checks run from fixed inputs alone

The adapter checks and the meeting checks SHALL read nothing but the lines, the raw place and the fixture's facts they are handed: no network and no source (68; core.35).

#### Scenario: The checks over fixed lines
- **WHEN** the checks are given one case's lines on two machines
- **THEN** both name the same breaches
