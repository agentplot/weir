---
number: 6
title: Flume's and sluice's inboxes share one delivery form and one crate in weir
status: accepted
date: 2026-11-08
decision-makers:
- Chuck Swanberg
- agentplot-design
---

# Flume's and sluice's inboxes share one delivery form and one crate in weir

Status: accepted by the user on 2026-11-08, on their card.

Deciders: the user, 2026-11-08, in agentplot-design's pane, on how Flume and sluice hand a reader what it waits for: "it seems like we're maybe inventing multiple similar solutions", "Is it actually tracking what was read?", "Is it done well?", and "So would we move a shared crate under Weir so that it's loose and Flume could use the same inbox system?". Then, on agentplot-design's card of the same day, which offered this direction: "Runner as one more recipient now; one shared inbox in weir next". The name, the form's rules below, and the order of the work are agentplot-design's, each cheap to change.

Sources: boomdrivedev/blueprints `design/weir/requirements.md`, section 1 ("An inbox written by one tool is read by the next with no import") and its vocabulary, where "inbox" is the captures' inbox; `design/flume/requirements.md` 7, 43, 62 and 132; `design/sluice/requirements.md` 51; boomdrivedev/flume `crates/flume-seams/src/held.rs` (`EndHeld`, `Delivery`, `DeliveredBy`) and `crates/flume-core/src/wait.rs` (`inbox`); boomdrivedev/flume record 0060, rewritten on the same answer; this repository's `core.1`, `core.22` and `core.30`.

## Context and problem statement

Flume and sluice each keep their own record of what a reader is owed, and each hands it over by its own `inbox`. Flume records each end of a place once, with a list of its deliveries, to whom, when and by what: a notice into a pane, a wait, or the inbox. An end is pending for a recipient that has none. sluice keeps the same idea for a submission's outcome, delivered to its submitter's session. Both count a message done the moment it is handed over, so a reader that stops before it acts on what it was given never sees it again. A notice counts once the terminal took it. Nothing owed expires, so a misspelt or departed reader holds its messages for good.

## Considered options

1. **One delivery form, with one crate, in weir (chosen).** weir fixes what a tool owes a reader and how a delivery is recorded and confirmed. A crate implements it as functions over the tool's own records, and a conformance pack holds any tool to it. Flume and sluice each keep their own storage.
2. **Each tool keeps its own and fixes it alone.** Not taken: the same rules written and tested twice, and the user's words against it.
3. **A message broker.** Not taken: the family's tools run with no service of their own (sluice `core.31`; Flume's settle), and a broker is one.

## Decision outcome

Option 1.

- **The name.** weir calls the form a *delivery*: a message one tool owes one reader. "Inbox" stays the captures' word in weir. The commands people type, `flume inbox` and `sluice inbox`, keep their names.
- **The form's rules.**
  - A message is recorded once, under a key, with each of its readers.
  - It stands owed to a reader until that reader confirms it, naming its key.
  - Handing it over, by an inbox, a wait or a notice, confirms nothing by itself. A reader that confirms on receipt, as an agent's session start may, says so in the same call.
  - An owed message expires after a time the tool's host sets, and an expired one is shown, with its reader, wherever the tool shows what is owed. It is never dropped silently.
  - A reader is named as the tool names it, and a name no reader has ever confirmed under is shown as such.
- **The crate.** Functions over records the caller passes in and gets back, holding no state and naming no tool, as `core.1` asks. Its records name their format, `weir-delivery/<major>` (`core.22`).
- **Each tool's storage stays its own.** Flume's place records on the host and sluice's records on the branch differ for their own reasons, and each tool plugs its store into the crate.
- **The order.**
  1. Flume's runner becomes one more recipient now, on Flume's present inbox (boomdrivedev/flume record 0060).
  2. weir's form, crate and pack.
  3. Flume, then sluice, move onto the crate, each in a change of its own. sluice takes its first dependency on weir for it.

## Consequences

- The statement gains weir's clauses for the form in the change that builds it, and Flume's 43 and 62 and sluice's 51 move to cite them as each tool adopts it.
