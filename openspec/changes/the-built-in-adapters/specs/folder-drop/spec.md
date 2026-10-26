## Purpose

The folder drop: a built-in adapter that offers one capture per file in a folder, keyed by the folder and the file's content, and speaks the contract as any adapter does (61, 64; C.6).

## ADDED Requirements

### Requirement: One capture per file, keyed by the folder and the file's content

The folder drop SHALL offer one capture per distinct content among its folder's files, keyed `folder/<the folder's own name>/<sha256 of the file>`, the hash the capture crate's (61; C.6; core.34). A file whose content an earlier file of the run had SHALL be offered no second time. A folder whose own name the key rule refuses SHALL fail the run before anything is offered, saying why (4).

#### Scenario: A file dropped twice, the second time under another name
- **WHEN** the folder `inbox-drop` holds `notes.txt`, the folder drop runs, `notes.txt` is copied to `notes-again.txt`, and the folder drop runs again
- **THEN** both runs offer the key `folder/inbox-drop/<sha256 of notes.txt>`, the second run offering it once, and a host taking both runs' lines writes one capture (S14)

#### Scenario: Two files with one content in one run
- **WHEN** the folder holds `a.txt` and `b.txt` with the same bytes
- **THEN** one offer is printed, titled `a.txt`, and the completion lists two

#### Scenario: The same file keys the same on every run
- **WHEN** one folder is listed twice, on two raw places
- **THEN** the two runs print the same lines

### Requirement: The folder's files are those directly in it

The folder drop SHALL list the files directly in its folder, in the order of their names, a symbolic link read as the file it names, and SHALL pass over subfolders and names beginning with `.` (61).

#### Scenario: Hidden files and subfolders
- **WHEN** the folder holds `.DS_Store`, a subfolder `old` and `report.pdf`
- **THEN** one offer is printed, for `report.pdf`

### Requirement: The event time is the date the name begins with, or else the last write

An offer's `event_at` SHALL be the date a file's name begins with, `YYYY-MM-DD` naming a day of the calendar, read as that day's start in UTC, `2026-10-02T00:00:00Z`; or else the file's last write, in UTC to the second (5, 61; C.6; core.8). It SHALL never be the time of listing.

#### Scenario: A dated name
- **WHEN** a file is named `2026-10-02-standup.vtt`
- **THEN** its offer's `event_at` is `2026-10-02T00:00:00Z`, whenever it was written

#### Scenario: An undated name
- **WHEN** a file is named `standup.vtt` and was last written at 2026-10-03T09:15:42.7Z
- **THEN** its offer's `event_at` is `2026-10-03T09:15:42Z`

#### Scenario: A name that begins with no day
- **WHEN** a file is named `2026-13-40-x.txt`
- **THEN** its offer's `event_at` is its last write

### Requirement: The file is the raw material, with no excerpt

Each offer SHALL carry source `folder`, title the file's name, the file copied byte for byte into the raw place as `<sha256>/<file name>`, named in `raw`, and no excerpt (6, 61; C.6). The bytes copied SHALL be the bytes hashed.

#### Scenario: The raw copy
- **WHEN** the folder drop runs with raw place `/r/1` over `notes.txt`
- **THEN** `/r/1/<sha256>/notes.txt` holds the file's bytes, the offer's `raw` is `["<sha256>/notes.txt"]`, and it carries no `excerpt`

### Requirement: The run ends with its completion, a failed file stopping no other

The folder drop SHALL end every run that could list its folder with one completion counting the files listed and naming each file that could not be read or copied, with why, and SHALL go on to the next file after one fails (9, 10; C.1). It SHALL print no cursor and SHALL take a `--cursor` without reading it, since it works without one and a host holds each key once (8).

#### Scenario: One file unreadable
- **WHEN** the folder holds `a.txt`, `b.txt`, which cannot be read, and `c.txt`
- **THEN** offers for `a.txt` and `c.txt` are printed, then `{"done": {"listed": 3, "failed": [{"what": "b.txt", "why": "…"}]}}`
