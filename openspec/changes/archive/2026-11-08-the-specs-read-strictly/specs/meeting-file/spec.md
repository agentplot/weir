## MODIFIED Requirements

### Requirement: The event time is the meeting's start where the file states it

The offer's `event_at` SHALL be the meeting's start where the file says it, the first of these it says: a `start` field, then a `date` field, of its front matter; else a date that opens its first line, the first line not blank after any front matter; else the date the file's name begins with, read as the folder drop reads it; else its last write, in UTC to the second (5, 62; C.6). A value or a line that reads as no time SHALL pass to the next way.

#### Scenario: A file that states its start
- **WHEN** the file opens with `---`, `start: 2026-10-02T14:00:00-04:00`, `---`
- **THEN** the offer's `event_at` is `2026-10-02T14:00:00-04:00`

#### Scenario: A date field and no start
- **WHEN** the file opens with `---`, `title: standup`, `date: "2026-10-02"`, `---`
- **THEN** the offer's `event_at` is `2026-10-02T00:00:00Z`

#### Scenario: A start before a date
- **WHEN** the file's front matter holds `date: 2026-10-01` and `start: 2026-10-02T14:00:00Z`
- **THEN** the offer's `event_at` is `2026-10-02T14:00:00Z`

#### Scenario: A start that is not a time
- **WHEN** the file's front matter holds `start: Tuesday` and no `date`, its text's first line is `Weekly sync`, and its name begins `2026-10-02-`
- **THEN** the offer's `event_at` is `2026-10-02T00:00:00Z`

#### Scenario: A date that opens the first line
- **WHEN** the file `call.txt` has no front matter and its first line is `2026-10-02 Weekly sync`
- **THEN** the offer's `event_at` is `2026-10-02T00:00:00Z`

#### Scenario: A time that opens the first line after the front matter
- **WHEN** the file's front matter holds only `title: standup`, a blank line follows it, and the next line is `2026-10-02T14:00:00-04:00 Weekly sync`
- **THEN** the offer's `event_at` is `2026-10-02T14:00:00-04:00`

#### Scenario: A transcript's header
- **WHEN** the file `2026-10-02-standup.vtt` opens with the line `WEBVTT`
- **THEN** the offer's `event_at` is `2026-10-02T00:00:00Z`

#### Scenario: A file that says nothing of when
- **WHEN** the file `call.txt` has no front matter, its first line is `Weekly sync`, and it was last written at 2026-10-03T09:15:42Z
- **THEN** the offer's `event_at` is `2026-10-03T09:15:42Z`

## ADDED Requirements

### Requirement: A meeting file's front matter is the block it opens with

A meeting file's front matter SHALL be a block the file opens with, a line `---`, lines `<name>: <value>` and a closing `---` (62; C.6).

#### Scenario: A file that states its start
- **WHEN** the file opens with `---`, `start: 2026-10-02T14:00:00-04:00`, `---`
- **THEN** the offer's `event_at` is `2026-10-02T14:00:00-04:00`
