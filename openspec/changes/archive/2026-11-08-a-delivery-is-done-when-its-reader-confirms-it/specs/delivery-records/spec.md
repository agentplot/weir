## ADDED Requirements

### Requirement: A delivery's records are the four of C.8

`weir-delivery` SHALL define four records, each naming its format (40; C.8; core.52):

- `weir-delivery/1`: `key`, a `reader` for each reader in the order given, `message`, `recorded_at`, `expires_at` (100, 104);
- `weir-handover/1`: `key`, `reader`, `by`, `at` (103);
- `weir-confirmation/1`: `key`, `reader`, `at` (102);
- `weir-known/1`: `reader`, `known_at` (106).

#### Scenario: Each record names its format
- **WHEN** a delivery, a hand-over, a confirmation and a known name are made
- **THEN** their formats are `weir-delivery/1`, `weir-handover/1`, `weir-confirmation/1` and `weir-known/1`

### Requirement: A key, a reader and a way are each one line

A key, a reader and a way (`by`) SHALL each be one line of text, never empty and never beginning or ending with a space or a tab. Any other SHALL be refused naming the field it was given for (C.8; core.47).

#### Scenario: A key holding a line break
- **WHEN** a key is given as `place/7f3a` followed by a line break and `end/2`
- **THEN** it is refused naming `key`, the reason saying a key is one line

#### Scenario: A reader with a space at its end
- **WHEN** a reader is given as `planner ` with a space at its end
- **THEN** it is refused naming `reader`

### Requirement: A message keeps its lines

A message SHALL be text of one or more lines, never empty, kept as given (100; C.8).

#### Scenario: A message of several lines
- **WHEN** a delivery's message holds three lines, the second empty
- **THEN** the delivery's record holds the three lines as given

### Requirement: A time is RFC 3339, kept as written

A time SHALL be RFC 3339 with its offset, kept as written, and any other text SHALL be refused naming the field (C.8; core.14).

#### Scenario: A time with no offset
- **WHEN** a confirmation's `at` is given as `2026-11-08T10:40:12`
- **THEN** it is refused naming `at`

### Requirement: The records are spelled in recutils

`weir-delivery` SHALL spell records as recutils text in the order given, `format` first and then each field in C.8's order, one `reader` field per reader of a delivery. It SHALL read the same text back into the same records, and the text it writes SHALL pass `recfix` (C.8; core.24, core.52).

#### Scenario: Four records spelled and read back
- **WHEN** a delivery for two readers, a hand-over, a confirmation and a known name are spelled in recutils and read back
- **THEN** the four records read back equal to those spelled, in order, and the text passes `recfix`

### Requirement: A record out of its format is refused

Reading recutils text SHALL refuse a record whose `format` the crate does not read, one lacking a field its format names, and one holding a field its format does not name, each naming the field (40; C.8; core.22).

#### Scenario: A record of a format the crate does not read
- **WHEN** recutils text holds a record whose `format` is `weir-delivery/2`
- **THEN** reading it is refused, naming `format` and the format

#### Scenario: A field the format does not name
- **WHEN** a `weir-confirmation/1` record holds a field `note`
- **THEN** reading it is refused, naming `note`

### Requirement: The records are spelled in JSON

`weir-delivery` SHALL spell each record as one JSON object on one line, `format` first and its fields as C.8 names them, a delivery's readers as a list `readers`. It SHALL read each line back into the same record, and SHALL refuse a field missing or one the format does not name, naming it (C.8; core.52).

#### Scenario: C.8's example read
- **WHEN** the four lines of C.8's example are read
- **THEN** they read as a delivery for `planner`, its hand-over by `notice`, its confirmation and the name `planner` known, and spell back as the same four lines

#### Scenario: A delivery with its readers left out
- **WHEN** a `weir-delivery/1` line has no `readers`
- **THEN** it is refused naming `readers`

### Requirement: A delivery's expiry is its recorded time and its span

A delivery's `expires_at` SHALL be its `recorded_at` and the span its deliverer gave, in `recorded_at`'s offset. A span SHALL be a whole number of seconds, at least one (104; C.8; core.49).

#### Scenario: A week's span
- **WHEN** a delivery is recorded at `2026-11-08T10:02:00-05:00` with a span of 604800 seconds
- **THEN** its `expires_at` is `2026-11-15T10:02:00-05:00`
