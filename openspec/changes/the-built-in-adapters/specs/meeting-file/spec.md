## Purpose

The meeting file: a built-in adapter that offers one capture for one meeting file, with the transcript as its raw material, and speaks the contract as any adapter does (62, 64; C.6; section 10, question 4).

## ADDED Requirements

### Requirement: One capture keyed by the file's content

The meeting file SHALL offer one capture for the file it is given, keyed `meeting/file/<sha256 of the file>`, the hash the capture crate's, with source `meeting`, title the file's name, the file copied byte for byte into the raw place as `<sha256>/<file name>` and named in `raw`, and no excerpt (62; C.6; core.34). The same bytes under any name SHALL key the same.

#### Scenario: A transcript offered
- **WHEN** the meeting file runs over `standup.vtt` with raw place `/r/1`
- **THEN** it prints one offer keyed `meeting/file/<sha256 of standup.vtt>`, source `meeting`, raw `["<sha256>/standup.vtt"]`, no excerpt, then `{"done": {"listed": 1, "failed": []}}`

#### Scenario: One transcript under two names
- **WHEN** the meeting file runs over `a.vtt` and then over a copy of it named `b.vtt`
- **THEN** both offers carry one key

### Requirement: The event time is the meeting's start where the file states it

The offer's `event_at` SHALL be the meeting's start where the file states it, as a `start:` field in RFC 3339 with its offset in a block the file opens with, a line `---`, lines `<name>: <value>` and a closing `---`, kept as written; else the date the file's name begins with, read as the folder drop reads it; else its last write, in UTC to the second (5, 62; C.6; section 10, question 4).

#### Scenario: A file that states its start
- **WHEN** the file opens with `---`, `start: 2026-10-02T14:00:00-04:00`, `---`
- **THEN** the offer's `event_at` is `2026-10-02T14:00:00-04:00`

#### Scenario: A start that is not a time
- **WHEN** the file's opening block holds `start: Tuesday`, and its name begins `2026-10-02-`
- **THEN** the offer's `event_at` is `2026-10-02T00:00:00Z`

#### Scenario: A file that says nothing of when
- **WHEN** the file `call.txt` has no opening block and was last written at 2026-10-03T09:15:42Z
- **THEN** the offer's `event_at` is `2026-10-03T09:15:42Z`

### Requirement: A file that cannot be read is named in the completion

When the file cannot be read or copied, the meeting file SHALL print no offer and end with a completion listing one and naming the file with why (9, 10; C.1). It SHALL print no cursor and SHALL take a `--cursor` without reading it (8).

#### Scenario: An unreadable file
- **WHEN** the meeting file runs over a file it cannot read
- **THEN** it prints only `{"done": {"listed": 1, "failed": [{"what": "<file name>", "why": "…"}]}}`
