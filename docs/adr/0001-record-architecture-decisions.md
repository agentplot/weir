---
number: 1
title: Record architecture decisions
status: proposed
date: 2026-10-26
decision-makers:
- Chuck Swanberg
---

# Record architecture decisions

Status: proposed to the user on 2026-10-26, as `briefs.15` has it; accepted only on their yes.

Deciders: the user, in weir's statement (plan step F1, ruled through 2026-10-26).
Sources: `agentplot/blueprints:design/weir/requirements.md` at 955c25a; `agentplot/blueprints:design/capture/design.md` and `plan.md`; `agentplot/blueprints:design/requirements-style.md`; sluice's, tailrace's and flume's `docs/adr/0001` as the model.

## Context and problem statement

weir starts with a statement of invariants and no code. Seven crates will be built side by side from it, by agents that read the repository and not the conversations that shaped the statement. A decision made in a conversation and nowhere else is lost to the next agent, and a rule with no record of why it holds is argued again at every change. Where do the decisions live, and how does a rule reach a session?

## Considered options

1. Decisions in the statement alone, cited by clause number from code and commits.
2. Decisions in the statement, and the rules a change must keep in `docs/architecture/`, each naming the clause it enforces, with the decisions behind the rules as MADR records in `docs/adr/`, one each, imported into every session by `CLAUDE.md`.
3. Rules in `CLAUDE.md` itself, with no records.

## Decision outcome

Option 2, as sluice, tailrace and flume keep theirs. The statement is the narrative and is never restated here; `docs/architecture/core.md` holds the rules a reviewer points at, each with the clause it enforces and what it rules out; `docs/adr/` holds one record per decision a rule rests on, proposed until the user's yes and named by the rules that rest on it. A change that bears on a clause cites it by number, and a clause that no longer holds is changed in the statement first (core.37). Nothing commercial appears in this repository (core.38).

### Consequences

- Every session in the kit has the rules, since `CLAUDE.md` imports `core.md`.
- A rule drawn from the statement waits on no yes; a rule drawn from a record stands as the record's status says.
- `check-adrs` holds every record to one number, the one its name says.
