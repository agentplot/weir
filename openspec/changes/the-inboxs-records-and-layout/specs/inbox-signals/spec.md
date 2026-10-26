# Spec Delta

## Purpose

Records what is read from a capture and what curation decides about it:
a capture's signals and its one reading, a signal's one standing move,
and the batches moves are proposed in (25 to 33; C.2).

## ADDED Requirements

### Requirement: A signal holds every field, and its nature is one of five

A signal SHALL be recorded as `weir-signal/1` with its identity, its nature in the field `kind`, who asserted it, what it asserts, its subjects and what it argues with (each a list, possibly empty), its excerpt and where it stands (25; C.2; record 0002). A signal lacking a field, or whose nature is not constraint, ask, question, commitment or reaction, SHALL be refused naming the field.

#### Scenario: A signal of an unknown nature

- **WHEN** a reading holds a signal whose nature is `complaint`
- **THEN** the reading is refused, naming that signal and its `kind`

#### Scenario: Empty lists

- **WHEN** a signal with no subjects and nothing it argues with is written and read back
- **THEN** it reads back with both lists empty

### Requirement: A capture's own signal is recorded in the same write as the capture

The capture and own signal the capture crate hands over SHALL be written in one write as `capture.rec` and a `signals.rec` holding the one signal `<key>#1`, its excerpt as cut and, where it was cut, a `cut` field giving the lines and bytes left out as `<n> lines and <b> bytes` (19, 20, 27; core.13). A capture with no own signal SHALL be written with no `signals.rec`.

#### Scenario: A cut own signal

- **WHEN** the capture crate hands over `release/mise/2026.10.10` with an own signal cut leaving out 812 lines and 39 020 bytes
- **THEN** one write adds `capture.rec` and `signals.rec`, the signal `release/mise/2026.10.10#1` with `cut: 812 lines and 39020 bytes`

#### Scenario: An unread capture

- **WHEN** the capture crate hands over a capture with no own signal
- **THEN** the write adds `capture.rec` alone, marked unread

#### Scenario: A held key

- **WHEN** the capture crate hands over a capture under a key the inbox holds
- **THEN** it is refused as held and nothing is written (14)

### Requirement: Signals are numbered in reading order

A capture's signals SHALL be numbered from 1 in the order they were read, and a signal's identity SHALL be `<key>#<n>` (27; C.2). An inbox whose `signals.rec` numbers otherwise, or names another capture's key, SHALL be refused naming the path.

#### Scenario: Three signals read

- **WHEN** a reader's reading of `meeting/file/ab12` holds three signals
- **THEN** they are written as `meeting/file/ab12#1`, `#2` and `#3`, in the order handed

#### Scenario: A gap in the numbers

- **WHEN** a tip's `signals.rec` holds `#1` and `#3`
- **THEN** reading the inbox is refused, naming the path

### Requirement: A capture has at most one reading

A reading SHALL be written only for a capture the inbox holds that has neither an own signal nor a reading; any other SHALL be refused naming the capture (28; core.17). A reading of no signals SHALL be written as a `signals.rec` holding no record, so the capture is no longer unread.

#### Scenario: A second reading

- **WHEN** a reading is written for a capture that already has one
- **THEN** it is refused, naming the capture, and nothing is written

#### Scenario: A reading of a capture with an own signal

- **WHEN** a reading is written for a capture whose offer carried an own excerpt
- **THEN** it is refused, naming the capture

#### Scenario: A reading of nothing

- **WHEN** a reader reads a capture and finds no signal
- **THEN** the write adds a `signals.rec` with no record, and the capture is no longer unread

### Requirement: A reading is written whole or refused whole

A reading SHALL be written whole, or refused whole naming each signal that could not be taken and why, writing nothing (28; core.17).

#### Scenario: One bad signal of three

- **WHEN** a reading holds three signals and the second lacks who asserted it
- **THEN** the reading is refused naming the second signal and `asserted_by`, and no signal is written

### Requirement: A signal's excerpt stands verbatim in its capture's raw material

A reader's signal SHALL be taken only when its excerpt stands, byte for byte, in one of the raw material files the writer hands in for that capture; otherwise the reading SHALL be refused naming the signal (26; core.16). The own signal's excerpt is the own excerpt as cut and is not looked for in raw material.

#### Scenario: A paraphrase

- **WHEN** a reading's signal has an excerpt that stands in none of the capture's raw files, though its words are close
- **THEN** the reading is refused, naming that signal

#### Scenario: Verbatim across lines

- **WHEN** a signal's excerpt is two whole lines of a raw transcript, as they stand there
- **THEN** it is taken

### Requirement: A signal has at most one standing move

A move for a signal with no standing move SHALL stand once written. A move for a signal that has a standing move, and that is not a replacement, SHALL be refused naming the standing move (29; I6; core.18). A move for a signal the inbox does not hold SHALL be refused.

#### Scenario: Two moves within a second

- **WHEN** a drop and a join for one signal are each formed on one tip, the drop lands, and the join is formed again on the new tip
- **THEN** the join is refused, naming the standing drop, its maker and when (S6)

#### Scenario: A move for no signal

- **WHEN** a move names `release/x#4` and the capture has three signals
- **THEN** it is refused, naming `release/x#4`

### Requirement: Only a replacement changes a standing move

A replacement SHALL name who made it and the move it replaces, by that move's number among its signal's moves in `moves.rec`, counting from 1, and SHALL be taken only when that move is the one standing (29; C.2). It is appended; the move it replaces stays as it was (41).

#### Scenario: A drop replaced by a join

- **WHEN** the operator replaces the standing drop of `release/x#1`, its first move, with a join
- **THEN** `moves.rec` gains the join with `replaces: 1` and the operator as `by`, the drop stays as written, and the join stands

#### Scenario: Replacing a move that no longer stands

- **WHEN** a replacement names move 1 of a signal whose move 2 now stands
- **THEN** it is refused, naming move 2 as the one standing

### Requirement: A move is one of six, with its consequence

A move SHALL be drop with its reason, answered with the claim or record that settled it, challenge with the claim it weighs against, join with the item it makes or grows, attach with the sent item it adds to, or route with the chore or ask it offers, and SHALL name who made it and when (30, 31). Any other word, or a missing consequence, SHALL be refused naming it.

#### Scenario: A seventh word

- **WHEN** a move's word is `promote`
- **THEN** it is refused, naming the word

#### Scenario: A drop without a reason

- **WHEN** a drop names no reason
- **THEN** it is refused, naming the missing consequence

#### Scenario: A join that makes an item

- **WHEN** a join names `shim-timeout`, which no item has had
- **THEN** the move stands and the item `shim-timeout` is made, proposed, by that join, named by the move's maker at the move's time

#### Scenario: An attach to an item not sent

- **WHEN** an attach names an item that has no send
- **THEN** it is refused, naming the item as not sent

#### Scenario: A join to a sent item

- **WHEN** a join names an item that has been sent
- **THEN** it is refused, naming the item as sent, for which an attach is the move

### Requirement: A move keeps the note given with it

A move given a reason or note SHALL keep it as its `note`, as given, in its `weir-move/1` record, and a drop given no note of its own SHALL carry its reason as its note, so a drop's reason is its consequence and its note at once (39; C.2). A note SHALL never stand for a missing consequence. A move record with no `note` SHALL read as a move with none.

#### Scenario: A challenge with a note

- **WHEN** a challenge of `release/x#1` names its claim and the note `the pin was chosen for the shim`
- **THEN** its record holds that `note` as given, and reads back with it

#### Scenario: A drop's reason is its note

- **WHEN** a drop of `release/x#2` gives the reason `fixed by the tile cache` and no note
- **THEN** its record holds that reason as its `target` and again as its `note`

#### Scenario: A note is not a consequence

- **WHEN** an answered move names no claim or record but gives a note
- **THEN** it is refused, naming the missing consequence

### Requirement: A batch's moves do not stand while it is proposed

A proposed batch SHALL be written whole to `inbox/moves/batches/<batch>.rec` under a name no batch of the inbox has had and holding no `/`, with who proposed it and when, and each move naming the batch; while proposed its moves SHALL NOT stand and its signals stay unmoved (31, 32; core.19).

#### Scenario: A proposed batch

- **WHEN** curation proposes batch `2026-10-25-a` of four moves on unmoved signals
- **THEN** the write adds the batch's file alone, and each of the four signals still has no standing move

#### Scenario: A batch name used before

- **WHEN** a batch is proposed under a name the inbox has had
- **THEN** it is refused, naming the batch

### Requirement: A batch is taken whole or refused whole

Proposing a batch SHALL refuse it whole, naming each move that could not be taken, when any move is invalid or two name one signal. Standing a batch SHALL append its moves to `moves.rec` and its decision to its file in one write, checked at that tip, or refuse it whole naming each move that can no longer be taken (32; core.19).

#### Scenario: A batch stands

- **WHEN** the operator accepts a proposed batch of four moves whose signals are still unmoved
- **THEN** one write appends the four moves to `moves.rec`, each naming the batch, and the batch's decision, `stands: yes` with who accepted it and when; the four moves now stand

#### Scenario: One signal moved meanwhile

- **WHEN** a batch is accepted after one of its signals was moved directly
- **THEN** the batch is refused whole, naming that move and the move that stands, and none of its moves stand

#### Scenario: A batch refused by the operator

- **WHEN** the operator refuses a proposed batch
- **THEN** its file gains `stands: no` with who and when, and none of its moves ever stand

#### Scenario: A batch decided twice

- **WHEN** a batch that already stands is accepted again
- **THEN** it is refused, naming the batch's decision

### Requirement: An unmoved signal is never discarded

No operation SHALL delete or omit a signal, however old, and every unmoved signal SHALL be listed with its age read from its capture's event time (33; core.8, core.19).

#### Scenario: An old unmoved signal

- **WHEN** a signal whose capture's event time is two years before now has no standing move
- **THEN** it is listed among the unmoved signals with that event time, and no operation removes it
