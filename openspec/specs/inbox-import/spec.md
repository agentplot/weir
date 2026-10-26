# inbox-import Specification

## Purpose
A host's import operation: carries captures read before their inbox
existed into it, each with the pointer, signals and moves it was read
with, held by key like any other capture (37 to 39, 63, 75a; S8, S9).

## Requirements

### Requirement: An imported capture is written as the body gave it

Each imported capture whose key the inbox does not hold SHALL be written with its key, source, event time, title and link as the import line gives them, the pointer as the body gave it or none, came in `import:<body>`, the importing machine and when it was imported (22, 37, 63, 75a). No raw material SHALL be banked or copied, and no pointer of the host's own written (37; core.25).

#### Scenario: A release capture imported

- **WHEN** an import line carries `release/mise/2026.10.10` with the pointer `mac-studio:~/.local/state/crew/x` and came in `import:crew-folders`
- **THEN** its `capture.rec` holds that key, that pointer and `came_in: import:crew-folders`, and nothing is banked

#### Scenario: A capture with no pointer

- **WHEN** an import line carries no pointer
- **THEN** the capture is written with no pointer

### Requirement: Imported signals and moves stand as read

An imported capture's signals SHALL be written as its reading, numbered from 1 in the body's order, with no excerpt looked for in raw material; a capture with no signals SHALL be written unread (26, 28, 37). Each imported move of one of the six with its consequence SHALL be appended to `moves.rec` and stand, as made by whom and when the body says (37).

#### Scenario: Signals and moves as read

- **WHEN** an import line carries three signals and a drop of signal 2 with its reason
- **THEN** the capture's `signals.rec` holds `#1` to `#3` as read, and `moves.rec` gains the drop of `#2`, standing

#### Scenario: An excerpt found in nothing

- **WHEN** an imported signal's excerpt stands in no raw material the importing machine holds
- **THEN** it is written all the same

### Requirement: An imported move keeps its note as read

A reason or note the body gave with any imported move SHALL be kept with it as its note, as read, and an imported drop given no note SHALL carry its reason as its note (39; C.2).

#### Scenario: A move's note kept as read

- **WHEN** an import line carries an attach of signal 1 with the note `same root cause as the October outage`, and a drop of signal 2 with its reason and no note
- **THEN** the attach's record holds that note as read, and the drop's record holds its reason as its target and as its note

### Requirement: An imported capture is held by key like any other

An imported capture whose key the inbox holds SHALL write nothing, its signals and moves with it, and SHALL be counted held; importing one body twice SHALL write nothing the second time, and a later offer under an imported key SHALL be held (14, 38; core.10, core.25). The import SHALL write captures only through the import operation, never through `take`, and `take` SHALL hold a key an import wrote as it holds any other (64).

#### Scenario: One body imported twice

- **WHEN** a partition's import lines are imported, and the same lines imported again on the new tip
- **THEN** the second import forms no write, and its report counts every capture held (S8)

#### Scenario: An offer under an imported key

- **WHEN** a capture was imported under `release/mise/2026.10.10` and an adapter later offers that key
- **THEN** the offer is held

### Requirement: An imported new-territory is a join into a proposed item named for its target

An imported move whose word is `new-territory` SHALL be read as a join into the proposed item its target names, then checked and written as any imported join, its record's word `join` (39; core.25). One whose target is empty, is not an item's name, or names a sent item SHALL not be taken: its signal SHALL arrive unmoved, and the report SHALL name the move with the word it came with and why.

#### Scenario: Two signals into new territory

- **WHEN** two imported captures each carry a `new-territory` move whose target is `shim-timeouts`, which the inbox does not hold
- **THEN** `moves.rec` gains two joins naming `shim-timeouts`, one `item.rec` for `shim-timeouts` is written, proposed, made by a join, with no `brief.md`, and both signals are its signals

#### Scenario: New territory with no target

- **WHEN** an imported `new-territory` move names no target
- **THEN** the move is not taken, its signal arrives unmoved, and the report names it as `new-territory` with the missing consequence

#### Scenario: New territory named for a path

- **WHEN** an imported `new-territory` move's target is `intent/shim-timeouts`
- **THEN** the move is not taken, and the report names it with the target as not an item's name

### Requirement: An imported move not of the six arrives unmoved

An imported move whose word is not one of the six and is not `new-territory`, that lacks its consequence, that names no signal of its capture, or that is a second move for one signal, SHALL not be taken; its signal SHALL arrive unmoved, and the import's report SHALL name the move and why (29, 30, 39; core.25).

#### Scenario: A word not of the six

- **WHEN** an imported move's word is `promote`
- **THEN** the move is not taken, its signal has no standing move, and the report names the signal, the word and why (S9)

#### Scenario: A join with no item

- **WHEN** an imported join names no item
- **THEN** the move is not taken and the report names it

### Requirement: An imported join makes or grows a proposed item with no brief

An imported join SHALL make the item it names, proposed with no brief and named by the move's maker at the move's time, or grow it when the inbox holds it proposed (39; core.25). A join naming a sent item, or an attach naming an item not sent, SHALL not be taken and SHALL be reported.

#### Scenario: Two joins into one item

- **WHEN** two imported captures each carry a join into `pin-mise`, which the inbox does not hold
- **THEN** one `item.rec` for `pin-mise` is written, made by a join, with no `brief.md`, and both signals are its signals

### Requirement: An imported capture is taken whole or not at all

An imported capture that names no key, or a key the key rule refuses, that names no source, or whose event time is missing or not RFC 3339 with its offset, SHALL be refused whole, its signals and moves with it, and named in the report with the field and why (4, 13, 75a). So SHALL one whose signals do not number from 1 in order, whose signal lacks a field or names a nature not of the five, or whose came-in is not `import:<body>` (15, 28, 75a). Every other capture of the import SHALL be taken.

#### Scenario: A capture that names no key

- **WHEN** an import of three captures holds one whose key is empty, and one whose key's first segment is `moves`
- **THEN** one capture is written, and the report names the other two refused, each with `key` and why

#### Scenario: One bad capture among many

- **WHEN** an import of twenty captures holds one whose second signal names no nature
- **THEN** nineteen are written, and the report names the one refused, its signal and `kind`

### Requirement: An import is one write, with its report

An import SHALL be formed as one write on one tip, checked there as every write is, and SHALL report the captures written, the count held, each capture refused and each move not taken, each with why (45, 46, 75a).

#### Scenario: The report

- **WHEN** an import of five captures meets one held key and one move of another word
- **THEN** one write adds four captures, and the report names the four, counts one held and names the move not taken
