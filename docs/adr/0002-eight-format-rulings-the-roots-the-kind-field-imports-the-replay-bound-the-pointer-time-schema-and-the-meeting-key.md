---
number: 2
title: "Eight format rulings: the roots, the kind field, imports, the replay bound, the pointer, time, the schema and the meeting key"
status: accepted
date: 2026-10-26
decision-makers:
- Chuck Swanberg
---

# Eight format rulings: the roots, the kind field, imports, the replay bound, the pointer, time, the schema and the meeting key

Status: accepted by the user on 2026-10-26, on their card ("Take my eight picks as they stand": approved; "Publish the design": approved).

Deciders: the user, 2026-10-26, on madswan-design's card of 2026-10-22.
Sources: `agentplot/blueprints:design/weir/requirements.md` section 10 and clauses 4, 17, 44, 61, 62, C.1, C.2, C.5, C.6, at 955c25a; `design/millpond/requirements.md` 63, C.1, C.11; `design/capture/design.md`, the key rule.

## Context and problem statement

weir's statement left eight questions to the operator in its section 10. Each fixes a record format that seven crates and three other tools read, so a unit's construct must not decide them, and they had to be settled before the first bolt. Each answer keeps what crew and millpond already write today, so importing the operator's curated signals stays faithful and nothing on main changes shape.

## Considered options

For each question, the statement's alternatives as section 10 posed them; the recommended answer in each case is the one the family's tools already produce.

## Decision outcome

1. The inbox has three roots, `captures/`, `moves/` and `items/`, and `items` is a third reserved first segment beside `captures` and `moves` in the key rule (clause 4, C.1, C.2; millpond C.1; the capture design).
2. The signal record's field stays spelled `kind`, as crew's and millpond's records write it; the statement's own word for it is nature, and Part C says the field is `kind`. millpond keeps its glossary.
3. crew's `new-territory` move imports as a join into a proposed item named for the move's target, so the operator's curation stands and nothing is judged again (clause 39; millpond 63, C.11).
4. The writer sets the replay bound, five unless it says otherwise; a write refused that many times is reported and never retried on its own (clause 44).
5. A pointer names the machine by the hostname the machine reports for itself, its short name, as `<hostname>:<path>`, which is what crew writes today (clause 17).
6. Cadence reads the machine's local time, as cron and launchd do (C.5).
7. An adapter's configuration shape is a JSON Schema, draft 2020-12, beside its declaration (C.1).
8. The meeting file's key is `meeting/file/<sha256 of the file>`; its event time is the meeting's start where the file says it, else the date its name begins with, else its last write, as the folder drop's (C.6).

### Consequences

- Rules core.7, core.11, core.25, core.26, core.33 and core.34 rest on this record.
- The statement carries each ruling under its question in section 10, and Part C is amended to match; millpond's statement and the capture design's key rule are amended with it (blueprints 955c25a).
