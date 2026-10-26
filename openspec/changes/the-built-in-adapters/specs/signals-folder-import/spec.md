## Purpose

The signals-folder import: reads crew's folders, the captures a partition's flywheel branch holds and the meeting corpus a blueprints repository holds, into import lines, one per capture, with the pointer, signals and moves each was read with. It is no adapter, and writes nothing: a host's import operation writes its lines (37 to 39, 63, 64; C.6; S8, S9).

## ADDED Requirements

### Requirement: One import line per capture folder, in the order of their names

`read_signals_folder` SHALL read a body's root: each folder under its `signals/` holding a `capture.md` is one capture, read in the order of the folders' names, and `moves.rec` at the root and in `signals/` hold its moves (63; C.6). It SHALL give one import line per capture it reads whole, `came_in` `import:crew-folders`. Anything under `signals/` that is not such a folder SHALL be passed over, and a root with no `signals/` folder SHALL be refused, naming it.

#### Scenario: A flywheel branch's tree
- **WHEN** a root holds `signals/2026-10-08-release-pendingyou-0-32-0/` and `signals/2026-10-09-swancloud-design-6d65f646/`, each with its `capture.md` and signals, `signals/README.md`, and `moves.rec`
- **THEN** two import lines are given, the release capture's first, and `README.md` is passed over

#### Scenario: A folder with no capture.md beside one that has one
- **WHEN** one capture folder lacks `capture.md` and another is whole
- **THEN** the whole one's line is given, and the other, being no capture, is passed over

### Requirement: A capture not read whole is named and stops no other

A capture folder `read_signals_folder` cannot read whole SHALL give no line, and SHALL be named, with the file and why, while every other capture is read (15, 28, 75a).

#### Scenario: One capture of three not read
- **WHEN** a body holds three captures and the second's `capture.md` names no `source`
- **THEN** the first's and the third's lines are given, and the second is named with `capture.md` and `source`

### Requirement: A release capture keeps its key and every other takes its folder's name

A capture whose `capture.md` names `source: release` and a `key` SHALL keep that key, `release/<name>/<version>`; every other capture SHALL take its folder's name as its key (38; C.6; core.25). A key the key rule refuses SHALL leave the capture unread and named, with the rule (4).

#### Scenario: A release capture
- **WHEN** `capture.md` names `source: release` and `key: release/pendingyou/0.32.0`, in the folder `2026-10-08-release-pendingyou-0-32-0`
- **THEN** the import line's key is `release/pendingyou/0.32.0`

#### Scenario: A session's capture
- **WHEN** `capture.md` names `source: crew-session` and `key: session/c433e3c0/6d65f646`, in the folder `2026-10-09-swancloud-design-6d65f646`
- **THEN** the import line's key is `2026-10-09-swancloud-design-6d65f646` and its source `crew-session`

### Requirement: The capture is read as crew wrote it

The import line's capture SHALL take `source` from `capture.md`'s `source`; `event_at` from its `at`, an RFC 3339 time, or else from its `event_date`, read as that day's start in UTC; `url` from its `url`; and `title` from the first line of its text that begins `# `, without the mark (5, 37). A `capture.md` naming no `source`, or neither `at` nor `event_date` as such a time or day, SHALL leave the capture unread and named.

#### Scenario: A release capture's fields
- **WHEN** `capture.md` holds `at: 2026-10-08T18:53:39Z`, `event_date: 2026-10-08`, `url: https://www.npmjs.com/package/pendingyou/v/0.32.0` and the heading `# pendingyou 0.32.0`
- **THEN** the line's capture holds `event_at` `2026-10-08T18:53:39Z`, that `url` and the title `pendingyou 0.32.0`

#### Scenario: A day and no time
- **WHEN** `capture.md` holds `event_date: 2026-10-02` and no `at`
- **THEN** the line's capture holds `event_at` `2026-10-02T00:00:00Z`

### Requirement: The pointer is crew's, and nothing it names is opened

`pointer` SHALL be `capture.md`'s `raw`, byte for byte, or absent when it names none, and no file it names SHALL be opened or copied (37; core.11, core.25).

#### Scenario: A pointer kept
- **WHEN** `capture.md` holds `raw: mac-studio:~/.local/state/swancloud/release-signals/raw/2026-10-08-release-pendingyou-0.32.0/`
- **THEN** the line's pointer is that string, and nothing at that path is read

#### Scenario: No pointer
- **WHEN** `capture.md` names no `raw`
- **THEN** the line holds no `pointer`

### Requirement: Signals are numbered from 1 in their files' order

A capture's signals SHALL be its files named `NN-<slug>.md`, in the order of `NN` and then of the name, numbered from 1 in that order (27; C.1).

#### Scenario: Signals numbered with a gap
- **WHEN** a capture holds `01-a.md` and `03-b.md`
- **THEN** they are signals 1 and 2

### Requirement: A signal's fields are read as crew writes them

Each signal SHALL be read with `kind` from its `kind`, `asserted_by` from its `who`, `subjects` from its `subject` and `argues_with` from its `claims`, each list `[a, b]` read as its entries and an absent one as empty, and `assertion` from the text's lines that are neither quoted, an attribution nor a heading, trimmed (25; C.6). A signal lacking a `kind` of the five, a `who`, an assertion or an excerpt SHALL leave its whole capture unread and named, with the file and why (28).

#### Scenario: A signal of no nature
- **WHEN** one signal's file of a capture names `kind: finding`
- **THEN** no line is given for that capture, and it is named with the file and `kind`

### Requirement: A signal's excerpt and position are read from its quote

A signal's `excerpt` SHALL be its lines beginning `>`, the mark and one space after it removed, joined by newlines, and its `position` the attribution crew writes beside it: the text after `" — ` on a one-line quote of the form `> "<words>" — <where>`, whose excerpt is then `<words>`, or else a line `— <where>` after the quote (25; C.6). A signal with no attribution SHALL take as its position the file's path in the body, `signals/<capture>/<file>`.

#### Scenario: A session's signal
- **WHEN** a signal's file holds `kind: constraint`, `who: swancloud-design`, the text `Claude Code does not fetch the tools again.` and the quote `> "HTTP reconnection successful after 16216ms (attempt 1)" — 22:21:18Z`
- **THEN** the signal reads with nature `constraint`, asserted by `swancloud-design`, that assertion, excerpt `HTTP reconnection successful after 16216ms (attempt 1)`, position `22:21:18Z` and no subjects

#### Scenario: A release's signal
- **WHEN** a signal's file holds `subject: [release, pendingyou]`, `claims: [packages/pendingyou/default.nix, flake.nix inputs.herdr-pendingyou-src]`, a quote of three `>` lines, and after it `— https://www.npmjs.com/package/pendingyou/v/0.32.0, published 2026-10-08T18:53:39Z`
- **THEN** its subjects are `release` and `pendingyou`, it argues with the two claims, its excerpt is the three lines without their marks, and its position is the text after `— `

### Requirement: Moves are carried by signal number

Each record of `moves.rec` naming a signal of a capture read SHALL be carried in that capture's import line, by the signal's number, a signal named by its full id, `<capture>/<NN>-<slug>`, or its short one, `<capture>/<NN>`; `by` from its `By`; and `at` from its `Date`, a day read as its start in UTC, or as written when it is not a day (31, 37; C.1).

#### Scenario: A route and a drop
- **WHEN** `moves.rec` holds a `route` of `<capture>/01-a` to `unit/fix-x` on `2026-10-06` by `swancloud-planner`, and a `drop` of `<capture>/02-b` with `Reason: the same defect as another`
- **THEN** the capture's line carries `{"signal": 1, "move": "route", "target": "unit/fix-x", "by": "swancloud-planner", "at": "2026-10-06T00:00:00Z"}` and a drop of signal 2 whose target is `the same defect as another`

### Requirement: crew's words are carried as the six, any other as it stands

A move's word and consequence SHALL be: `attach`, `challenge`, `answered` and `route` as they stand, the target from `Target`; `drop` as it stands, the target its `Reason`, or else its `Target`; and any other word as it stands, the target from `Target`, for the import operation to leave its signal unmoved (30, 39).

#### Scenario: A word not of the six
- **WHEN** a move's word is `promote`
- **THEN** the line carries it as `promote`, for the import operation to leave unmoved (S9)

### Requirement: crew's new-territory is a join into a proposed item named for its target

crew's `new-territory` SHALL be carried as a `join`, its target the name of a proposed item made from `Target`, lowercase ASCII letters and digits kept and every other run of characters one `-`, trimmed of `-` (39; section 10, question 3; core.25). A `new-territory` with no `Target`, or one leaving no name, SHALL be carried as a join with no target.

#### Scenario: crew's new-territory
- **WHEN** a move's word is `new-territory` with `Target: upstream: Claude Code's HTTP MCP reconnection`
- **THEN** the line carries a `join` whose target is `upstream-claude-code-s-http-mcp-reconnection`

#### Scenario: A new-territory with no target
- **WHEN** a move's word is `new-territory` and it holds no `Target`
- **THEN** the line carries a `join` with no target

### Requirement: A move of a signal outside the body is named, not carried

A record of `moves.rec` naming a signal of no capture read SHALL be named as not carried, with why, and SHALL not stop the rest (15).

#### Scenario: A move of a signal in another body
- **WHEN** a flywheel branch's `moves.rec` moves a signal whose capture lies in a blueprints repository
- **THEN** it is named as not carried, and every line is given

### Requirement: Reading one body twice gives the same lines

`read_signals_folder` SHALL read nothing but the body's files, so one body read twice, on any machine, gives the same lines byte for byte, and a host's import operation holds each capture by its key the second time (38; S8; core.14). It SHALL write nothing.

#### Scenario: One folder read twice
- **WHEN** one body is read twice
- **THEN** the two readings' lines are equal, byte for byte, and the body's files are as they were
