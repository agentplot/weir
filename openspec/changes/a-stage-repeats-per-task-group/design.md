# Design

## Context

`read_stage` reads each field `STAGE_FIELDS` names and returns every
other key of the stage in its `sections`, so a `repeat` written today
is returned unread (56). The statement at blueprints d75cf53 names it:
a stage may hold whether it repeats per task group (53), spelled
`repeat = "per-task-group"` (C.4), and a stage that repeats and does
not deliver tasks ticked is refused naming the field (57). How a reader
works a repetition is its own (58).

Rules this design relies on: core.30 (one reader, sections returned
unread, nothing rewritten), core.31 (checked whole, refused naming the
field), core.2 (no tool named in the crate's code), core.35 (each pack
case cites its clauses), core.39 (the kinds project's checks). The area
is the kinds crate; there are no area pages.

## Decisions

### `repeat` is `Option<Repeat>`, one variant

`Stage::repeat: Option<Repeat>`, `None` when left out. `Repeat` is an
enum with the one variant `PerTaskGroup`, spelled `per-task-group`, its
`Display` the spelling as `Deliverable`'s is. An enum rather than a flag
keeps the spelling the format's and leaves room for the statement to
name another way to repeat without a second field. `repeat` joins
`STAGE_FIELDS`, so it no longer appears in `Stage::sections`.

### What is refused, and once

A string other than `per-task-group` is refused naming
`stages[n].repeat` and the one value it takes; a value of any other
shape is refused naming the field as a string the format spells. A
`repeat` that reads is checked against the stage's end: on a
conversation, or on a deliverable other than `tasks`, it is refused
naming `stages[n].repeat` and the stage's end (57). Where the stage's
own end is refused, its `deliverables` or `conversation` already named,
the `repeat` is not judged against it, so one fault is named once; the
declaration is refused whole either way. A refused `repeat` reads as
`None` in the value the refusal discards.

### The pack's case format names `repeat`

A case's stage gains `repeat`, the spelling, left out when the stage
does not repeat; the crate's reading is compared with it as it is with
the stage's end. Two cases join the pack: flume's change@2, its code
stage repeating, read twice the same (53, 57, 58; S10), and `repeat` on
a stage of commits refused naming the field (57).

## Risks / Trade-offs

- [A struct literal of `Stage` outside the crate fails to compile on
  the new field] → readers only read stages; no crate of the family
  builds one, and every existing field reads as before.
