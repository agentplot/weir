# Proposal

## Why

`root:specs` runs `openspec validate --strict` over the specs. On main
at 52b9552, which landed the delivery change, it refuses
`conformance-pack`. The delivery change named the statement's commit
and the clauses statechart kinds leave out in *The pack keeps an index
of the statement's clauses*. That made the requirement's text run over
500 characters, and `--strict` fails it. sluice sent the landing of
bolt/weirs-gate-validates-its-specs back for it as #13.

## What Changes

- *The pack keeps an index of the statement's clauses* is split into
  two requirements that together say what it said, each 500 characters
  or fewer, as the-specs-read-strictly split the 23 before it. The
  requirement split keeps its name and its one scenario. The part added
  beside it carries a copy of that scenario, word for word.
- No behavior changes. No clause, rule, record, code, test or pack case
  changes.

## Capabilities

### New Capabilities

None.

### Modified Capabilities

- `conformance-pack`: the index of the statement's clauses.

## Impact

- `openspec/specs/conformance-pack/spec.md`, once this change is
  archived.
- Nothing a crate reads or builds changes. No scenario's words change,
  and the copy is proved by the test that proves the scenario it
  copies.
