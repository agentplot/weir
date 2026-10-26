# Spec Delta

## Purpose

What an adapter tells a host and how it is declared and started: the three kinds of line it prints, the key rule, the offer's fields, its declaration with its configuration's JSON Schema, and the content hash file keys are made from (statement 1 to 12, 72 to 74, C.1).

## ADDED Requirements

### Requirement: Every line is exactly one of three

A line SHALL be read as an offer, a cursor or a completion when it is one JSON object holding exactly one of the keys `capture`, `cursor` and `done`; any other line SHALL be refused as malformed, naming its reason (1; C.1; core.5). A top-level key the contract does not name SHALL be ignored, not refused (core.6).

#### Scenario: One key of three reads as that line
- **WHEN** the lines `{"capture": {…}}`, `{"cursor": "2026-10-17T04:30:05Z"}` and `{"done": {"listed": 14, "failed": []}}` are read
- **THEN** they read as an offer, a cursor and a completion, in that order

#### Scenario: A line with none or two of the keys is malformed
- **WHEN** the line `{"cursor": "a", "done": {"listed": 0, "failed": []}}` is read, and then `{"note": 1}`, and then `not json`
- **THEN** each is refused as malformed with a reason naming what it lacks or holds twice

#### Scenario: An unknown top-level key is ignored
- **WHEN** the line `{"cursor": "c1", "trace": "x"}` is read
- **THEN** it reads as the cursor `c1`

### Requirement: An offer carries a key, a source and an event time

An offer SHALL be read only when it holds `key`, `source` and `event_at`, the event time in RFC 3339 with its offset; `title`, `url`, `raw` and `excerpt` are optional, and a field the contract does not name SHALL be ignored (2, 5, 13; C.1; core.6). The event time SHALL be kept as offered.

#### Scenario: A missing required field refuses the offer
- **WHEN** an offer holds a key and a source but no `event_at`
- **THEN** it is refused, naming `event_at` as missing

#### Scenario: An event time without its offset refuses the offer
- **WHEN** an offer's `event_at` is `2026-10-10T14:02:11`
- **THEN** it is refused, naming `event_at` as not an RFC 3339 time with an offset

#### Scenario: The event time is kept as offered
- **WHEN** an offer's `event_at` is `2026-10-10T16:02:11+02:00`
- **THEN** the offer read holds `2026-10-10T16:02:11+02:00`, unchanged

### Requirement: A key names nothing in an inbox but its own capture

A key SHALL be one or more `/`-separated segments, and SHALL be refused when it starts with `/`; when a segment is empty, `.` or `..`; when a segment is the name of a file the inbox's layout places, `capture.rec`, `signals.rec`, `moves.rec`, `item.rec`, `brief.md` or `sends.rec`, since each segment is a folder; or when its first segment is `captures`, `moves` or `items` (3, 4; C.1; C.2; core.7).

#### Scenario: A key that reaches another root is refused
- **WHEN** keys `moves/x`, `captures/x`, `items/x`, `/release/x` and `release/../x` are checked
- **THEN** each is refused, naming the rule it breaks

#### Scenario: A key that names another key's place is refused
- **WHEN** keys `release//x`, `release/x/`, `release/./x` and the empty key are checked
- **THEN** each is refused, naming the rule it breaks

#### Scenario: A key that names a file of the layout is refused
- **WHEN** keys `release/capture.rec`, `release/x/signals.rec`, `capture.rec`, `deploy/moves.rec`, `deploy/item.rec`, `deploy/brief.md` and `deploy/sends.rec` are checked
- **THEN** each is refused, naming the segment and the file it would stand where

#### Scenario: A key of plain segments is taken
- **WHEN** the key `release/mise/2026.10.10` is checked
- **THEN** it is taken, with the segments `release`, `mise`, `2026.10.10`

### Requirement: An own excerpt holds every field of a signal

An offer's `excerpt` SHALL hold `text`, non-empty; `kind`, one of `constraint`, `ask`, `question`, `commitment`, `reaction`; `asserted_by`; `assertion`; `subjects` and `argues_with`, each a list that may be empty; and `position`. An offer whose excerpt lacks any of them, or holds an empty text or another kind, SHALL be refused naming the field (7, 13, 25; C.1).

#### Scenario: An excerpt with no text is refused
- **WHEN** an offer's excerpt holds every field but `text` is empty
- **THEN** the offer is refused, naming `excerpt.text`

#### Scenario: An excerpt of an unknown nature is refused
- **WHEN** an offer's excerpt has `kind` `"opinion"`
- **THEN** the offer is refused, naming `excerpt.kind`

#### Scenario: An excerpt with empty lists is taken
- **WHEN** an offer's excerpt holds every field and `subjects` and `argues_with` are `[]`
- **THEN** the offer is read with its own excerpt

### Requirement: Raw material is named inside the raw place

Each path in an offer's `raw` SHALL be relative to the raw place, and an offer naming a path that is absolute, holds a `..` segment, or resolves outside the raw place SHALL be refused naming the path (6; C.1).

#### Scenario: A raw path outside the raw place is refused
- **WHEN** an offer's `raw` is `["../elsewhere.md"]`
- **THEN** the offer is refused, naming `../elsewhere.md`

### Requirement: A cursor is opaque and a completion counts the listing

A cursor SHALL be a string kept as given and never parsed or compared (8; core.9). A completion SHALL hold `listed`, a count, and `failed`, each entry with `what` and `why` (9).

#### Scenario: A cursor is kept byte for byte
- **WHEN** the line `{"cursor": "page=3&since=2026-10-17"}` is read
- **THEN** the cursor read is `page=3&since=2026-10-17`, unchanged

#### Scenario: A completion lacking a failure's why is malformed
- **WHEN** the line `{"done": {"listed": 2, "failed": [{"what": "x"}]}}` is read
- **THEN** it is refused as malformed, naming `done.failed[0].why`

### Requirement: Lines are written as the host reads them

An offer, a cursor or a completion SHALL be written as one line of JSON with no newline inside it, which reads back as the same line (C.1; core.34).

#### Scenario: A written offer reads back the same
- **WHEN** an offer with an own excerpt whose text holds several lines is written and the result read
- **THEN** the written form is one line, and the line read equals the offer written

### Requirement: An adapter's declaration is read whole

A declaration SHALL be a TOML table `[adapter.<name>]` holding `command`, a non-empty list of strings; `config`, the path of a JSON Schema beside the declaration; and `secrets`, a list of names. A declaration missing a field, or holding one of the wrong shape, SHALL be refused naming the field (11, 72; C.1).

#### Scenario: A declaration is read
- **WHEN** `[adapter.meetings]` with `command = ["meetings", "capture"]`, `config = "config.schema.json"` and `secrets = ["MEETINGS_TOKEN"]` is read
- **THEN** the declaration of `meetings` holds that command, that schema path and that one secret name

#### Scenario: A declaration with no command is refused
- **WHEN** `[adapter.x]` holds `config` and `secrets` but no `command`
- **THEN** it is refused, naming `adapter.x.command`

### Requirement: A configuration is checked against its JSON Schema

An adapter's configuration SHALL be checked against the JSON Schema its declaration names, as draft 2020-12, without fetching anything a schema refers to; a configuration that fails SHALL be refused naming each place it fails (11; section 10, question 8; record 0002).

#### Scenario: A configuration missing a required property is refused
- **WHEN** the schema requires the property `folder` and the configuration is `{}`
- **THEN** the configuration is refused, naming `folder`

#### Scenario: A configuration that fits is taken
- **WHEN** the schema requires `folder`, a string, and the configuration is `{"folder": "/tmp/drop"}`
- **THEN** the configuration is taken

### Requirement: A run is started from the declaration alone

The command line a run is started with SHALL be the declaration's command followed by `--raw-dir <dir>` and, when a cursor is kept, `--cursor <cursor>`; its environment SHALL name `CAPTURE_CONFIG` and each declared secret by its declared name, and carry no secret's value (11, 12, 73; C.1; core.3).

#### Scenario: A run with a cursor
- **WHEN** a run of `meetings` is formed with raw place `/r/1` and cursor `c9`
- **THEN** its command line is `meetings capture --raw-dir /r/1 --cursor c9` and its environment names `CAPTURE_CONFIG` and `MEETINGS_TOKEN`

#### Scenario: A first run has no cursor
- **WHEN** a run of `meetings` is formed with raw place `/r/1` and no cursor
- **THEN** its command line ends `--raw-dir /r/1`

### Requirement: The content hash is sha256

The content hash of a file's bytes SHALL be their sha256 as 64 lowercase hexadecimal digits, the same for the same bytes whatever the file's name (61, 62; C.6; core.34).

#### Scenario: Known bytes hash as sha256
- **WHEN** the bytes `abc` are hashed
- **THEN** the hash is `ba7816bf8f01cfea414140de5dae2223b00361a396177a9cb410ff61f20015ad`
