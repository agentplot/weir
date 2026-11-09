## MODIFIED Requirements

### Requirement: One import line per capture folder, in the order of their names

`read_signals_folder` SHALL read a body's root: each folder under its `signals/` holding a `capture.md` is one capture, read in the order of the folders' names (63; C.6). It SHALL give one import line per capture it reads whole, `came_in` `import:crew-folders`. Anything under `signals/` that is not such a folder SHALL be passed over, and a root with no `signals/` folder SHALL be refused, naming it.

#### Scenario: A drive branch's tree
- **WHEN** a root holds `signals/2026-10-08-release-pendingyou-0-32-0/` and `signals/2026-10-09-swancloud-design-6d65f646/`, each with its `capture.md` and signals, `signals/README.md`, and `moves.rec`
- **THEN** two import lines are given, the release capture's first, and `README.md` is passed over

#### Scenario: A blueprints repository's corpus with the drive branch's moves
- **WHEN** a blueprints repository's root holds `signals/2026-10-02-standup/` with `capture.md`, `01-a.md` and `02-b.md`, and a `signals/moves.rec` moving `2026-10-02-standup/02-b`, and it is read naming a drive branch's `moves.rec` that moves `2026-10-02-standup/01-a`
- **THEN** one import line is given, carrying the move of signal 1 from the drive branch's file, signal 2 carrying no move, so it arrives unmoved

#### Scenario: A blueprints repository's corpus read alone
- **WHEN** the same root is read naming no moves file
- **THEN** its line carries no move

#### Scenario: A folder with no capture.md beside one that has one
- **WHEN** one capture folder lacks `capture.md` and another is whole
- **THEN** the whole one's line is given, and the other, being no capture, is passed over

## ADDED Requirements

### Requirement: A body's moves are those of one moves.rec

The moves `read_signals_folder` reads with a body's captures SHALL be those of one `moves.rec`, the file the caller names or else the one at the root, a `signals/moves.rec` never read (63; C.6).

#### Scenario: A blueprints repository's corpus with the drive branch's moves
- **WHEN** a blueprints repository's root holds `signals/2026-10-02-standup/` with `capture.md`, `01-a.md` and `02-b.md`, and a `signals/moves.rec` moving `2026-10-02-standup/02-b`, and it is read naming a drive branch's `moves.rec` that moves `2026-10-02-standup/01-a`
- **THEN** one import line is given, carrying the move of signal 1 from the drive branch's file, signal 2 carrying no move, so it arrives unmoved

#### Scenario: A blueprints repository's corpus read alone
- **WHEN** the same root is read naming no moves file
- **THEN** its line carries no move
