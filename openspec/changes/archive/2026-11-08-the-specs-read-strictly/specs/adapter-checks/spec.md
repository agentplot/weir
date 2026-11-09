## MODIFIED Requirements

### Requirement: The adapter checks hold a run's lines to the contract

The adapter checks SHALL be given a run's lines, the raw place it was handed and whether it exited reporting success, and SHALL name every breach of the contract, each with its line where it has one, the clause it breaks and why. Lines with no breach SHALL pass, and a breach SHALL stop no other line being checked (10).

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

The meeting checks SHALL be given what the adapter checks are, and, from the adapter's own fixture, each meeting's start by its key and how many meetings it held too short to hold anything; and SHALL name every breach of 66: an offer that breaks it, and a completion whose count is not the offers, the failures and the meetings left out together. A run that passes the meeting checks SHALL also pass the adapter checks.

#### Scenario: A meeting adapter that keeps 66
- **WHEN** the meeting checks are given two offers keyed `meeting/wispr/m-1` and `meeting/wispr/m-2`, each at its start with two raw files and no excerpt, and a completion listing three with one meeting too short
- **THEN** they name no breach

#### Scenario: A short meeting not counted
- **WHEN** the fixture held one meeting too short, the adapter offers two, and its completion lists two
- **THEN** the meeting checks name the completion breaking 66

#### Scenario: An event time that is not the start
- **WHEN** an offer's event time is when its transcript was saved, not its meeting's start
- **THEN** the meeting checks name that line breaking 66

## ADDED Requirements

### Requirement: The adapter checks name a line or an offer that breaks the contract

The adapter checks SHALL name as a breach a line that is not exactly one of an offer, a cursor and a completion (1; core.5); an offer without a key, a source or an event time, or whose key the key rule refuses (2, 4, 13); an event time without its offset (5; C.1); and two offers in one run under one key that differ in their source or event time, two events sharing a key (3).

#### Scenario: Two offers under one key
- **WHEN** lines 2 and 5 offer `release/acme/v1`, with two event times
- **THEN** the checks name line 5 breaking 3, and check every other line

### Requirement: The adapter checks name raw material, an excerpt or a completion that breaks the contract

The adapter checks SHALL name as a breach raw material named outside the raw place, or not in it after the run (6); an own excerpt lacking its text or any field (7); a run that exited reporting success with no completion, or with lines after it, or with two (9); a failure with no `what` or no `why` (9); and an own excerpt whose text ends with the host's closing line, which only a host writes (12, 20).

#### Scenario: Success with no completion
- **WHEN** a run's lines hold no completion and it exited reporting success
- **THEN** the checks name a breach of 9 with no line

#### Scenario: An adapter that cut its own excerpt
- **WHEN** an offer's own excerpt ends with `[cut: 12 lines and 4096 bytes left out, banked whole with the capture]`
- **THEN** the checks name that line breaking 12

### Requirement: The meeting checks name an offer that breaks 66

The meeting checks SHALL name as a breach of 66 an offer whose key is not `meeting/<service>/<the meeting's own identity>`; whose source is not `meeting`; whose event time is not its meeting's start; that names fewer than two raw files, the transcript as served and a plain copy; or that carries an own excerpt.

#### Scenario: An event time that is not the start
- **WHEN** an offer's event time is when its transcript was saved, not its meeting's start
- **THEN** the meeting checks name that line breaking 66
