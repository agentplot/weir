# Spec Delta

## Purpose

Records the units of work curation makes of signals: an item, the
signals and weight read from its standing moves, its brief, and each
send of it with what became of it (34 to 36; C.2).

## ADDED Requirements

### Requirement: An item's signals are read from the standing moves that name it

An item's signals SHALL be exactly the signals whose standing join or attach names it, read from `moves.rec` whenever asked, and SHALL NOT be recorded in the item's record (34; core.20).

#### Scenario: A join replaced

- **WHEN** three signals are joined into `shim-timeout` and the join of one is then replaced by a drop
- **THEN** the item's signals are the other two, and its `item.rec` is unchanged

#### Scenario: An attach counts

- **WHEN** a signal is attached to the sent item `shim-timeout`
- **THEN** it is among the item's signals

### Requirement: An item's weight is computed, never stored

An item's weight SHALL be computed from its signals: how many, the sources of their captures, and the earliest and latest of their captures' event times; no record SHALL hold a weight (34; core.8, core.20).

#### Scenario: Weight from two sources

- **WHEN** an item holds three signals, two from release captures of 2026-10-01 and 2026-10-10 and one from a meeting capture of 2026-10-05
- **THEN** its weight is three signals, from `meeting` and `release`, spanning 2026-10-01 to 2026-10-10 as their event times give them

### Requirement: An item is made by a join or from an accepted conclusion

An item SHALL be recorded as `weir-item/1` with its name, whether a join or a conclusion made it, the conclusion where one did, and who named it and when (34; C.2). An item made from a conclusion SHALL name that conclusion; one with no conclusion named SHALL be refused.

#### Scenario: An item from a conclusion

- **WHEN** an item `pin-mise` is proposed from the conclusion `inquiry/toolchain#3`
- **THEN** the write adds `inbox/items/pin-mise/item.rec`, made by a conclusion and naming it, and the item is proposed with no signals

### Requirement: An item's name is one its inbox has never had

An item's name SHALL be one no item of the inbox has had, SHALL hold no `/`, and SHALL be neither empty, `.` nor `..`; any other SHALL be refused naming the name (35; core.20). A join naming an existing item grows it rather than making it.

#### Scenario: A name already had

- **WHEN** an item is proposed from a conclusion under the name of an item the inbox holds
- **THEN** it is refused, naming the name

#### Scenario: A nested name

- **WHEN** a join names the item `shim/timeout`
- **THEN** it is refused, naming the name

### Requirement: An item is proposed until first sent

An item SHALL be proposed while it has no send and sent once it has one (glossary; 36).

#### Scenario: The first send

- **WHEN** a proposed item is sent
- **THEN** it reads as sent from that tip on

### Requirement: An item's brief is the text handed for it, rewritten in place

An item's brief SHALL be `inbox/items/<name>/brief.md`, its first line naming `weir-brief/1` and the rest the text handed, unchanged; writing it again SHALL rewrite that file alone (36, 41; core.21). The crate SHALL compose no brief: it records the text curation or the operator hands (core.20).

#### Scenario: A brief written and rewritten

- **WHEN** a brief is written for `shim-timeout` and later written again with other text
- **THEN** `brief.md` holds the later text after its format line, and no other file changes

#### Scenario: A brief for no item

- **WHEN** a brief is written for an item the inbox does not hold
- **THEN** it is refused, naming the item

### Requirement: A send keeps the brief as handed

A send SHALL append to the `sends.rec` of each item it hands a `weir-send/1` record with the item, the target, the brief as it stood when sent, when it was sent, and the outcome `open`; the brief it keeps SHALL not change when `brief.md` later does (36; core.20). An item with no brief SHALL not be sent.

#### Scenario: A brief rewritten after a send

- **WHEN** `shim-timeout` is sent with one brief and its brief is then rewritten
- **THEN** its send still holds the brief as it was sent

#### Scenario: Several items to one target

- **WHEN** two items are sent together to one target
- **THEN** one write appends a send to each item's `sends.rec`, both with that target and time

#### Scenario: An item with no brief

- **WHEN** an item with no `brief.md` is sent
- **THEN** it is refused, naming the item

### Requirement: What became of a send is added beside it

A send's outcome SHALL be added as a record after it in the same `sends.rec`, naming the send by its target and when it was sent, and giving the outcome: a landing, a conclusion, or abandoned (36, 41; C.2). The send itself SHALL stay as written. An outcome for no send, or for a send already settled, SHALL be refused.

#### Scenario: A send that landed

- **WHEN** the send of `shim-timeout` lands as a commit
- **THEN** `sends.rec` gains a record naming that send with the outcome a landing and what landed, and the send's own record is unchanged

#### Scenario: An outcome for a settled send

- **WHEN** an outcome is added for a send whose outcome is already abandoned
- **THEN** it is refused, naming the send and its outcome
