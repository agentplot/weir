# import-lines Specification

## Purpose
The import line: the fourth line C.1 shows, printed only by an import and read only by a host's import operation, never by `take`. It carries one capture read before its inbox existed, with the pointer, signals and moves it was read with (37 to 39, 63, 75a; C.1).

## Requirements

### Requirement: An import line carries one capture with its pointer, signals and moves

An import line SHALL be one JSON object holding `import` and none of `capture`, `cursor` and `done` at its top, its value an object holding `capture`, `came_in`, `signals` and `moves`, and optionally `pointer` (63; C.1). A field the line's shape does not name SHALL be ignored (core.6). `read_import_line` SHALL read it, or refuse it as malformed naming the field, as `import.signals[1].kind`, and the capture's key where one was read.

#### Scenario: A release capture with one signal and one move
- **WHEN** the line `{"import": {"capture": {"key": "release/mise/2026.10.10", "source": "release", "event_at": "2026-10-10T14:02:11Z", "title": "mise 2026.10.10"}, "pointer": "mac-studio:~/.local/state/x/", "came_in": "import:crew-folders", "signals": [{"n": 1, "kind": "ask", "asserted_by": "jdx/mise", "assertion": "mise 2026.10.10 is released.", "subjects": ["release"], "argues_with": [], "excerpt": "notes", "position": "release notes"}], "moves": [{"signal": 1, "move": "route", "target": "unit/bump-mise", "by": "planner", "at": "2026-10-11T00:00:00Z"}]}}` is read
- **THEN** it reads as that capture, pointer, came-in, one signal and one move, each field as written

#### Scenario: A line holding an adapter's key as well
- **WHEN** a line holds both `import` and `cursor`
- **THEN** it is refused as malformed, naming both

### Requirement: The capture is an offer's object without raw material or an excerpt

The `capture` object SHALL hold `key`, `source` and `event_at` and MAY hold `title` and `url`, each read as an offer's are: the key by the key rule, the event time in RFC 3339 with its offset and kept as written (3 to 5; C.1). A `capture` holding `raw` or `excerpt` SHALL be refused, naming the field, since an import copies no raw material and an imported capture's signals are its reading (37, 63).

#### Scenario: A key the rule refuses
- **WHEN** an import line's capture key is `moves/x`
- **THEN** the line is refused, naming `import.capture.key` and the rule

#### Scenario: Raw material named in an import
- **WHEN** an import line's capture holds `raw`
- **THEN** the line is refused, naming `import.capture.raw`

### Requirement: The pointer is kept as the body gave it

`pointer` SHALL be read as a non-empty string and kept byte for byte, or be absent, and nothing SHALL be checked of it (37, 63). `came_in` SHALL be `import:<body>` with a body named, and any other value refused naming `import.came_in` (22).

#### Scenario: A pointer of another machine
- **WHEN** an import line's pointer is `deneb:~/.local/state/crew/raw/a.jsonl`
- **THEN** it reads as that string, unchanged

#### Scenario: A came-in of an adapter's run
- **WHEN** an import line's `came_in` is `run:folder`
- **THEN** the line is refused, naming `import.came_in`

### Requirement: Every signal holds every field of a signal

`signals` SHALL be a list, possibly empty, each entry holding every field of a signal: `kind`, one of the five natures; `asserted_by`; `assertion`; `subjects` and `argues_with`, lists that may be empty; `excerpt`, not empty; and `position` (25; C.1). A signal lacking a field, or with one of the wrong shape or a nature not of the five, SHALL refuse the line whole, naming the field (28). No excerpt SHALL be looked for in anything (37).

#### Scenario: A capture with no signals
- **WHEN** an import line's `signals` is `[]`
- **THEN** it reads with no signals

#### Scenario: A nature not of the five
- **WHEN** an import line's second signal has `kind` `finding`
- **THEN** the line is refused, naming `import.signals[1].kind`

### Requirement: Signals are numbered from 1 in the list's order

Each signal SHALL hold `n`, numbered 1, 2, 3 in the list's order, and a number out of order SHALL refuse the line whole, naming the field, as `import.signals[1].n` (27, 28; C.1).

#### Scenario: Signals numbered out of order
- **WHEN** an import line's signals are numbered 1 and 3
- **THEN** the line is refused, naming `import.signals[1].n`

### Requirement: A move of any word is carried as it stands

`moves` SHALL be a list, possibly empty, whose entries hold `signal`, a whole number, and `move`, a non-empty word, and MAY hold `target`, `note`, `by` and `at`, strings kept as written, `note` the reason or note the body gave with the move, for the import operation to keep with it (31, 39; C.1, C.2). A move SHALL be read whatever its word and whether or not it carries its consequence, so the import operation can leave its signal unmoved and report it (39; core.25); only a move whose `signal` is not a whole number or whose `move` is not a non-empty string SHALL refuse the line, naming the field.

#### Scenario: A word not of the six
- **WHEN** an import line carries a move `{"signal": 1, "move": "promote", "by": "curation", "at": "2026-10-12T00:00:00Z"}`
- **THEN** it reads with that move, word `promote`, unchanged

#### Scenario: A drop with no reason
- **WHEN** an import line carries a move `{"signal": 1, "move": "drop"}`
- **THEN** it reads with that move, no target and no note

#### Scenario: A move with its note
- **WHEN** an import line carries a move `{"signal": 1, "move": "attach", "target": "intent/x", "note": "the same ask as the standup's"}`
- **THEN** it reads with that move, its note `the same ask as the standup's` as written

### Requirement: An import line is written as it is read

`ImportLine::to_json` SHALL write one line of compact JSON, with no newline inside it, its fields in the order C.1 shows, a move's `note` after its `target`, and an absent `pointer`, `title`, `url`, `target`, `note`, `by` or `at` left out, which `read_import_line` reads back equal (C.1).

#### Scenario: A line round-trips
- **WHEN** an import line whose signal's excerpt holds three lines, with no pointer and one move with a note, is written and read back
- **THEN** it is one line and reads back equal to the line written
