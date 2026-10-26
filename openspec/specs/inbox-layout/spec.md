# inbox-layout Specification

## Purpose
Fixes where each record of an inbox lies, how an inbox is read whole
from one tip, the format and major every record names, and that a write
only adds, so every host and tool reads and writes one inbox the same
way (40 to 43; C.2).

## Requirements

### Requirement: Each capture lies where its key fixes, with its signals beside it

A capture SHALL lie at `inbox/captures/<its key's segments as nested folders>/capture.rec`, and its own signal or its reading in `signals.rec` in the same folder (42; C.2). An inbox whose `capture.rec` names a key other than its folder's, or whose `signals.rec` has no `capture.rec` beside it, SHALL be refused naming the path (core.23).

#### Scenario: A capture's place

- **WHEN** the capture `release/mise/2026.10.10` is written with its own signal
- **THEN** the write adds `inbox/captures/release/mise/2026.10.10/capture.rec` and `inbox/captures/release/mise/2026.10.10/signals.rec`, and nothing else

#### Scenario: A capture out of its place

- **WHEN** a tip holds `inbox/captures/release/a/capture.rec` naming the key `release/b`
- **THEN** reading the inbox is refused, naming that path and its `key`

#### Scenario: Signals with no capture

- **WHEN** a tip holds `inbox/captures/release/a/signals.rec` and no `capture.rec` beside it
- **THEN** reading the inbox is refused, naming that path

### Requirement: Moves, batches and items lie where no key reaches

Moves SHALL lie in `inbox/moves/moves.rec`, each batch in `inbox/moves/batches/<batch>.rec`, and each item in `inbox/items/<name>/` as `item.rec`, `brief.md` and `sends.rec` (42; C.2). Every file the layout places SHALL be a name the key rule refuses as a key segment, and `captures`, `moves` and `items` the first segments it refuses, so no key reaches them (4).

#### Scenario: The layout's names are names no key holds

- **WHEN** each file name the layout places, and each of its three roots, is given to the capture crate's key rule
- **THEN** each file name is refused in any segment and each root as a first segment

#### Scenario: A move and an item where the layout puts them

- **WHEN** a join names a new item `shim-timeout`
- **THEN** the write adds the move to `inbox/moves/moves.rec` and the item's record at `inbox/items/shim-timeout/item.rec`

### Requirement: An inbox is read whole from one tip, and holds only what the layout places

An inbox SHALL be read from every file under `inbox/` of one tip, handed in by the caller, and every operation SHALL answer from that one read (43, 46). A file under `inbox/` the layout does not place SHALL be refused naming its path; files outside `inbox/` SHALL be passed over. Reading SHALL change nothing it was handed (I7).

#### Scenario: A stray file

- **WHEN** a tip holds `inbox/moves/notes.txt`
- **THEN** reading the inbox is refused, naming `inbox/moves/notes.txt`

#### Scenario: A file outside the inbox

- **WHEN** a tip holds `README.md` beside `inbox/`
- **THEN** reading the inbox passes over it

#### Scenario: An empty tip

- **WHEN** a tip holds no file under `inbox/`
- **THEN** reading gives an empty inbox, which holds no key and no item

### Requirement: Every record names its format and major

Every record SHALL name `format: weir-<record>/<major>` as its first field: `weir-capture/1`, `weir-signal/1`, `weir-move/1`, `weir-batch/1`, `weir-item/1` and `weir-send/1`, and a brief `weir-brief/1` on its first line (40; C.2; core.22). A record naming no format, or a format other than the one its place holds, SHALL be refused naming its path and record.

#### Scenario: A record with no format

- **WHEN** `moves.rec` holds a record with no `format` field
- **THEN** reading the inbox is refused, naming `inbox/moves/moves.rec` and the record's place in it

#### Scenario: A record of another format where it lies

- **WHEN** an item's `sends.rec` holds a record naming `weir-move/1`
- **THEN** reading the inbox is refused, naming the path and the format it holds

#### Scenario: Every record written names its format first

- **WHEN** any operation forms a write
- **THEN** every record in every `.rec` file it adds or appends names its format as its first field, and every brief it writes names `weir-brief/1` on its first line

### Requirement: A reader reads every earlier major and writes none it did not read

A reader SHALL read every major of a format up to the latest it knows, as it stands, and SHALL refuse an inbox holding a later major, naming the path and the format, so that it never writes into an inbox it could not read (40; core.22). Every record the crate writes SHALL be in a major it reads back.

#### Scenario: A later major

- **WHEN** `moves.rec` holds a record naming `weir-move/2`
- **THEN** reading the inbox is refused, naming the path and `weir-move/2` as a later major than the reader reads, and no write can be formed on that tip

#### Scenario: Written and read back

- **WHEN** every kind of record is written and the inbox read again
- **THEN** each reads back as it was written

### Requirement: A write only adds

A write SHALL add files, append records to the end of `moves.rec`, a batch's file or `sends.rec`, or rewrite `brief.md`, and SHALL do nothing else (41; core.21). A write that changes a `capture.rec`, a `signals.rec` or an `item.rec` already at the tip, or any record already in an appended file, SHALL be refused naming the path, whoever formed it.

#### Scenario: A capture rewritten

- **WHEN** a write handed to the check replaces a `capture.rec` the tip holds, even with the same capture marked read
- **THEN** it is refused, naming the path

#### Scenario: A move edited in place

- **WHEN** a write handed to the check gives `moves.rec` with its first move's target changed and a move appended
- **THEN** it is refused, naming `inbox/moves/moves.rec`

#### Scenario: A brief rewritten

- **WHEN** a write rewrites an item's `brief.md`
- **THEN** it passes the check

### Requirement: Every write is checked against the tip it is formed on

Every operation SHALL form its write from the inbox read at one tip and check it against that read: the inbox the write would leave SHALL itself read whole, and the write SHALL only add (46; core.24, core.26). Forming the same operation on a later tip SHALL make every check again there. The check SHALL be open to a write formed elsewhere.

#### Scenario: Formed again on a new tip

- **WHEN** a move for an unmoved signal is formed on one tip, and formed again on a later tip where another move for that signal now stands
- **THEN** the first forming gives a write, and the second is refused naming the standing move

#### Scenario: A write formed elsewhere

- **WHEN** a write adding a `moves.rec` record whose word is `promote` is handed to the check
- **THEN** it is refused, naming the path and the word

### Requirement: Every file a write forms passes recfix

Every `.rec` file a write adds or changes SHALL be accepted by GNU recutils' `recfix` (core.24; C.2). The crate's tests SHALL hold every file its operations form to `recfix`; the crate itself SHALL run no program (core.1).

#### Scenario: Each operation's files

- **WHEN** each operation's write in the crate's tests is handed to `recfix`, file by file
- **THEN** `recfix` accepts every file
