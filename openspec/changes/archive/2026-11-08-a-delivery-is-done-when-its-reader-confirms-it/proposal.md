# Proposal

## Why

Record 0006 was accepted by the user on 2026-11-08, on their card. It
gives weir one form for a message a tool owes a reader, with one crate
any tool calls over its own records and a pack that holds a tool to it.
flume and sluice each keep their own record of what a reader is owed
today. Each counts a message done the moment it is handed over, so a
reader that stops before it acts never sees the message again, and
nothing owed ever expires.

The statement at blueprints 006fc34, on the branch
`design/weir-a-delivery-is-done-when-its-reader-confirms-it`, writes the
form into weir:

- A.14, 100 to 107: a delivery is recorded once under its key with its
  readers (100, 101). It stays owed to each reader until that reader
  confirms it by its key (102). A hand-over confirms nothing unless the
  reader confirms on receipt in the same call (103). Its expiry is fixed
  when it is recorded, and an expired delivery is shown with its reader
  and never dropped (104, 105). A name no reader has confirmed under is
  shown as such (106). Only a done delivery is let go of (107).
- B.7, 108 to 113: `record`, `hand`, `confirm` and `owed` are functions
  over a store's records. A deliverer's store is held to the pack
  through its `read` and `add`.
- C.8 spells four records, `weir-delivery/1`, `weir-handover/1`,
  `weir-confirmation/1` and `weir-known/1`, in recutils on a shared
  branch and in JSON on a machine.
- 65, 67, 70, 86, 87 and C.7 add the pack's `delivery` shape, count
  A.14, and keep weir from handing a delivery over or confirming one.

core.4, core.35 and core.47 to core.52 say it as rules, on this branch.

## What Changes

- A new crate, `weir-delivery`, with its moon project `delivery`. It
  holds the four records, their recutils and JSON spellings, the four
  calls, and the `Store` seam a deliverer implements.
- `weir-pack` gains the `delivery` shape. It reads a delivery case and
  runs every delivery case through a store a caller hands it, as C.7
  says. Its tests run the cases through a store of each spelling, and
  fail a store that drops a record.
- `pack/delivery/` holds the cases, citing 100 to 107 between them.
- The pack's index moves to blueprints 006fc34 and takes in A.14 and
  B.7. The pack's check counts A.14 beside A.1 to A.10 and 71.
- `Cargo.toml`, `.moon/workspace.yml` and the README name the new
  crate.

## Capabilities

### New Capabilities

- `delivery-records`: the four records of C.8, their fields, their
  recutils and JSON spellings, each read back as written.
- `delivery-calls`: `record`, `hand`, `confirm` and `owed` over the
  records a caller passes in, with what each returns or refuses.
- `delivery-cases`: the pack's delivery cases, their format, and the run
  that holds weir's crate and any deliverer's store to them.

### Modified Capabilities

- `conformance-pack`: seven shapes, `delivery` among them; the index at
  006fc34 with A.14 and B.7; the check counting A.14.

## Impact

- New: `crates/weir-delivery/` with its `moon.yml`, and
  `pack/delivery/`.
- `crates/weir-pack`: `Shape::Delivery`, a `delivery` module, the
  counted sections, a dependency on `weir-delivery`, and its moon
  project's inputs and `dependsOn`.
- `pack/clauses.toml`: the commit, and the sections A.14 and B.7.
- `Cargo.toml` members and its crate list, `.moon/workspace.yml`, and
  the README's table of crates.
- flume and sluice move onto the crate in changes of their own, as
  record 0006 orders it. sluice takes its first dependency on weir for
  that change. Neither moves in this one.
