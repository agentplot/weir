# Design

## Context

flume and sluice each owe a reader messages and keep their own record
of them. flume records each end of a place once, with a list of its
deliveries in `EndHeld` (`crates/flume-seams/src/held.rs`): to whom,
when, and by `DeliveredBy::Notice`, `Wait` or `Inbox`. An end with no
delivery for a recipient is pending, and `flume inbox` collects and
marks it (flume 43, 62). sluice keeps an outcome pending until a push,
a harness's collect or a wait marks it delivered (sluice 51). Both
count a message done when it is handed over, and neither expires one.

Record 0006, accepted by the user on 2026-11-08, gives weir one form for
this, with a crate any tool calls over its own records and a pack that
holds a tool to it. Its form's rules, its name and the order of the work
are agentplot-design's, each cheap to change. The statement at
blueprints 006fc34 writes the form in:

- 100 to 107 (A.14): a delivery is recorded once under its key with its
  readers, owed to each until that reader confirms it by its key. A
  hand-over confirms nothing unless the reader confirms on receipt in
  the same call. Its expiry is fixed when recorded. An expired delivery
  is shown with its reader until confirmed, a name no reader has
  confirmed under is shown as such, and only a done delivery is let go
  of.
- 108 to 113 (B.7): `record`, `hand`, `confirm` and `owed` over a
  store's records, and a store's `read` and `add`.
- C.8: the four records and their two spellings. C.7: the `delivery`
  shape and the run that holds a store. 65, 67, 70, 86, 87 and I9
  follow.

The details of the form below are this change's own, written into the
statement at 006fc34, each cheap to change.

Rules this design relies on, all in `docs/architecture/core.md`, the one
page whose area the change touches:

- core.1 and core.51: the calls answer from their inputs, read no clock
  and hold no store.
- core.2: no tool named. Cases and tests name readers `planner`,
  `reviewer` and `stranger`, ways `notice`, `wait` and `session-start`,
  and keys `place/<id>/end/<n>`.
- core.3: no secret in a record.
- core.4: weir hands no delivery over and confirms none.
- core.22: each record names its format and major.
- core.24: what weir writes in recutils passes `recfix`.
- core.35: the `delivery` shape, its cases run against weir's crate, and
  A.14 counted. Amended on this branch.
- core.37: the clauses are in the statement first, at 006fc34.
- core.39 and core.40: the crate is a moon project of its own, one task
  per check.
- core.47 to core.52: the delivery's rules, written on this branch.

Where weir has no rule of its own, kit:testing.1 and kit:testing.3 bind
the new moon project's tasks. The design departs from no rule.

## Goals / Non-Goals

**Goals:**

- One crate, `weir-delivery`, holding the four records, both
  spellings, the four calls and the store seam.
- The pack's `delivery` shape: cases citing 100 to 107, run against
  the crate in weir's tests through a store of each spelling, and
  runnable by any deliverer over its own store.
- The pack's index at 006fc34, and its check counting A.14.

**Non-Goals:**

- flume and sluice moving onto the crate. Each moves in a change of its
  own, flume first (record 0006).
- How a deliverer hands a delivery over, when it hands one again, its
  span, and who may confirm for a reader. These are the deliverer's
  (section 8; core.4).
- Where a store lies, its files and how it lets go of a done delivery.
  These are the deliverer's (107; C.8).
- A `weir` command for deliveries. Section 9 names none, and the
  deliverers link the crate.
- A.13 and the `tick` shape, which weir's statechart change takes in.

## Decisions

### 1. One crate, `weir-delivery`, beside the others

`crates/weir-delivery` is a library crate with the moon project
`delivery`. It depends on `weir-capture` for `rec`, the one recutils
reader and writer, and on `time`, `serde` and `serde_json`. Modules:
`text` (the line types, the message, the time and the span), `record`
(the four records and `Record`), `rec` and `json` (the spellings),
`calls` (`record`, `hand`, `confirm`), `owed` and `store`. Its
`lib.rs` carries `#![deny(missing_docs)]`, as weir-cadence's does.

The way not taken is putting the delivery into `weir-signals`, the
inbox's crate. "Inbox" stays the captures' word (record 0006), a
deliverer needs none of the inbox, and a crate of its own lets sluice
take this one dependency on weir alone.

### 2. Four records, each written once

A delivery, a hand-over, a confirmation and a known name are four
records, `weir-delivery/1`, `weir-handover/1`, `weir-confirmation/1`
and `weir-known/1` (C.8). None changes once written. What happens to a
delivery is added beside it, as a move's replacement is (41). A store
on a shared branch only appends, so two writers meet at fewer lines,
and a store on a machine keeps the same records.

The way not taken is one record per delivery rewritten with its
hand-overs and confirmations, as flume's `EndHeld` is. That breaks "a
delivery never changes once recorded" (100), and on a branch it turns
every hand-over into a rewrite of the record. A single format with a
field saying which of four it is was not taken either: C.2 gives each
record a format of its own.

### 3. The line types and the message

`Key`, `Reader` and `Way` are each a newtype over one line of text,
built only through `parse`, which refuses empty text, a line break
(`\n` or `\r`), and a space or tab at either end. Each refusal names
the field it was read for (`key`, `reader`, `by`). `Message` is text
that is not empty, any lines kept as given. The parse rules keep every
value spelled the same in recutils and in JSON (C.8).

### 4. A time is kept as written

`Time` holds an RFC 3339 time as written, checked with `time`'s
`Rfc3339` and kept with its parsed `OffsetDateTime`. Two times are
equal when their text is equal, as a capture's `event_at` is kept
(core.14). Order and expiry compare the instants. `expires_at` is
written by `time`'s `Rfc3339` formatting of `recorded_at` plus the span,
at `recorded_at`'s offset, so `2026-11-08T10:02:00-05:00` and 604800
seconds give `2026-11-15T10:02:00-05:00`. `Span` is a whole number of
seconds, at least 1, built through `Span::seconds`, which refuses 0
naming `span`.

### 5. A held key returns the delivery that stands

`record` over records holding a delivery under the key adds nothing.
It returns `Recorded::Held(delivery)`, the first delivery under the
key in the records, whatever the call's message, readers or span
(101). A deliverer may then call `record` at every pass over its own
state and learn whether it had recorded the message.

The way not taken is refusing a held key. A refusal would make each
deliverer track what it had recorded before calling, and 14 already
gives the family "held, writes nothing" for a key met twice.

### 6. The expiry is fixed when recorded

`record` takes the span and writes `expires_at`. `owed` reads
`expires_at` and the time its caller gives, and nothing else (104;
core.49). A delivery is expired for a reader once that time is at or
past its `expires_at`.

The way not taken is a span given to `owed` at each read. A setting
changed later would then change what had expired, and two deliverers
reading one store with different settings would disagree on it.

### 7. A known name is a record of its own

The first confirmation under a name returns a `weir-known/1` record
beside the confirmation, `known_at` its time. `confirm` and `hand`
(confirming on receipt) look for a `weir-known/1` naming the reader,
and return none where one exists. `owed` says a name is known from
these records alone (106; core.50).

The way not taken is reading knownness from confirmations. A deliverer
lets go of a done delivery with its confirmations (107), and the name
would then be forgotten.

### 8. A confirmation is the one way out

A delivery leaves what a reader is owed on that reader's confirmation
and on nothing else (102). An expired delivery stays in every answer of
`owed`, marked expired, until confirmed (104, 105). weir gives no call
that drops or dismisses one. A deliverer that lets an operator clear a
reader's expired deliveries does so by confirming them under that
reader's name, a choice of who may confirm, which is the deliverer's
(section 8).

The way not taken is a dismissal record. Record 0006 names the
reader's confirmation as what ends a delivery, and a second way out
would let a delivery end unseen.

### 9. The calls

The calls are in `calls.rs` and `owed.rs`. Every argument a caller
gives as text is parsed into its type inside the call, so a refusal
names the field:

```rust
pub fn record(records: &[Record], key: &str, message: &str, readers: &[&str],
              at: &str, span: u64) -> Result<Recorded, Refusal>;
pub enum Recorded { Add(Vec<Record>), Held(Delivery) }

pub fn hand(records: &[Record], key: &str, reader: &str, by: &str, at: &str,
            confirms: bool) -> Result<Handed, Refusal>;
pub enum Handed { Add(Vec<Record>), Confirmed(Time) }

pub fn confirm(records: &[Record], key: &str, reader: &str, at: &str)
               -> Result<Confirmed, Refusal>;
pub enum Confirmed { Add(Vec<Record>), Already(Time) }

pub fn owed(records: &[Record], now: &str) -> Result<Owed, Refusal>;
pub struct Owed { pub readers: Vec<ReaderOwed>, pub done: Vec<Key> }
pub struct ReaderOwed { pub reader: Reader, pub known: bool,
                        pub deliveries: Vec<OwedDelivery> }
pub struct OwedDelivery { pub delivery: Delivery, pub expired: bool,
                          pub handovers: Vec<Handover> }

pub struct Refusal { pub field: String, pub reason: String }
```

A call refuses at the first field that fails, in argument order: `key`,
`message`, `readers`, `reader`, `by`, `at`, `now`, `span`, then what
the records say of the key and the reader. Reasons:

| refusal | field | reason holds |
|---|---|---|
| no reader | `readers` | "names no reader" |
| a reader named twice | `readers` | "names `<reader>` twice" |
| a span of 0 | `span` | "is 0 seconds" |
| no delivery under the key | `key` | "no delivery is recorded under `<key>`" |
| a reader the delivery does not name | `reader` | "`<reader>` is not a reader of `<key>`" |

`hand` and `confirm` find a reader's confirmation of the key in the
records; where there is one, they return its `at` and add nothing.
`hand`'s `Add` holds the hand-over, and where it confirms, the
confirmation and the known name `confirm` would return.

`owed` walks the deliveries in the order the records hold them, the
first under each key alone (101). For each reader of each, in the
delivery's order of readers, it puts the delivery under that reader
unless the reader confirmed it. Readers come in the order first met.
A reader with nothing unconfirmed is left out. A hand-over or a
confirmation naming no delivery of the records names nothing owed, and
`owed` passes over it. `done` holds each key every reader of which has
confirmed, in the order recorded.

### 10. The spellings

`rec::write(&[Record]) -> String` and `rec::read(&str) -> Result<Vec<Record>, Refusal>`
spell through `weir_capture::rec`. Each record is `format` and then its
fields in C.8's order, one `reader` field per reader of a delivery, and
a message's later lines as `+` continuation lines. A record is read by
its `format`. A format the crate does not read, a field missing, a
field the format does not name, or a field given twice where C.8 gives
it once is refused, naming the field.

`json::write(&Record) -> String` gives one line, a JSON object with
`format` first and its fields in C.8's order, a delivery's readers as
`readers`. `json::read(&str) -> Result<Record, Refusal>` reads one
line back. It refuses a field missing or one the format does not name,
naming it, through serde structs with `deny_unknown_fields`, one per
format. `json::write_lines` and `json::read_lines` spell many records,
one per line.

### 11. The store seam

```rust
pub trait Store {
    type Error: std::fmt::Display;
    fn read(&self) -> Result<Vec<Record>, Self::Error>;
    fn add(&mut self, records: &[Record]) -> Result<(), Self::Error>;
}
```

`weir-delivery` defines the trait and implements it for nothing (112;
core.51, core.52). A deliverer implements it over its own store.

The way not taken is calls that take a store and write to it. Record
0006 makes the crate functions over records the caller passes in and
gets back. A caller then forms its own write, a branch's against its
tip, which weir's branch protocol already holds (44, 46).

### 12. The pack's `delivery` shape and its run

`Shape` gains `Delivery`, spelled `delivery`, after `Kinds`, and
`Shape::ALL` holds seven. `weir-pack` depends on `weir-delivery` and
gains a `delivery` module:

```rust
pub fn read_delivery_case(case: &Case) -> Result<DeliveryCase, String>;
pub fn run_delivery_case<S: Store>(case: &DeliveryCase, store: &mut S)
    -> Result<(), Vec<String>>;
pub fn conform_deliveries<S: Store>(pack: &Pack, make: impl FnMut() -> S)
    -> Vec<(String, Result<(), Vec<String>>)>;
```

`run_delivery_case` follows C.7. It adds `start.rec`'s records to the
store, reads them back and names each one not read back as written. It
calls the case's call over what it read, and adds what `Add` returns.
Then it reads the store and compares it, record by record, with the
starting records followed by `added.rec`. It names a record expected
and not read back as `not read back: <its JSON line>`, and a record
read back and not expected as `not expected: <its JSON line>`. It
compares the answer, held, already, refused or owed, with the case's,
naming what differs. `conform_deliveries` runs every delivery case of
the pack, each through a fresh store from `make`, and gives each case's
name and verdict. A deliverer calls it in its own tests with
`Pack::embedded()`.

The case format, with unknown fields refused:

```toml
# What the case holds.
satisfies = [102]

[call.confirm]          # one of record, hand, confirm, owed
key = "place/7f3a/end/2"
reader = "planner"
at = "2026-11-08T10:40:12-05:00"

# One of: added.rec beside the case; held = true (record);
# already = "<time>" (hand, confirm); [refused]; or, for owed:
done = ["place/7f3a/end/2"]
[[owed]]
reader = "reviewer"
known = false
[[owed.delivery]]
key = "place/7f3a/end/2"
expired = false
handed = ["notice"]     # the ways of its hand-overs to this reader, in order
```

`[call.record]` takes `key`, `message`, `readers`, `at` and `span`;
`[call.hand]` takes `key`, `reader`, `by`, `at` and `confirms`;
`[call.owed]` takes `now`. `[refused]` holds `field` and `reason_holds`.
A case holding no expectation, or two, fails its read naming the case.
`start.rec` and `added.rec` are recutils, which treefmt.nix already
excludes under `pack/**/*.rec`, so no new file type joins the tree.

### 13. The cases

Each lies under `pack/delivery/`, and between them they cite every
clause of 100 to 107:

| case | call | expects | cites |
|---|---|---|---|
| `a-delivery-recorded-for-two-readers` | record | `added.rec`, expiring a week on | 100, 104 |
| `a-delivery-with-no-reader` | record | refused `readers` | 100 |
| `a-reader-named-twice` | record | refused `readers` | 100 |
| `a-delivery-with-no-message` | record | refused `message` | 100 |
| `a-span-of-no-seconds` | record | refused `span` | 104 |
| `one-key-recorded-twice` | record | held | 101 |
| `a-reader-confirms-by-its-key` | confirm | confirmation and known name | 102, 106 |
| `a-second-confirmation` | confirm | already | 102 |
| `a-confirmation-of-a-key-no-delivery-is-under` | confirm | refused `key` | 102 |
| `a-confirmation-by-a-reader-not-named` | confirm | refused `reader` | 102 |
| `a-hand-over-confirms-nothing` | hand | `added.rec`, a hand-over alone | 103 |
| `handed-and-still-owed` | owed | owed with its hand-over | 102, 103, 105 |
| `a-reader-confirms-on-receipt` | hand | hand-over, confirmation and known name | 103, 106 |
| `a-hand-over-to-a-reader-that-confirmed` | hand | already | 103 |
| `a-hand-over-to-a-reader-not-named` | hand | refused `reader` | 103 |
| `an-expired-delivery-is-shown` | owed | expired, name not known | 104, 105, 106 |
| `an-expired-delivery-confirmed` | confirm | confirmation and known name | 102, 104 |
| `owed-in-the-order-recorded` | owed | `c`, `a`, `b` | 105 |
| `a-name-stays-known` | owed | known with no delivery of its own left | 106 |
| `a-delivery-done-when-every-reader-confirms` | owed | done | 107 |
| `a-delivery-not-done-while-a-reader-has-not-confirmed` | owed | owed to one | 107 |

`an-expired-delivery-is-shown` holds one delivery for `planer`, expiring
`2026-11-09T10:02:00-05:00`, asked at `2026-11-12T09:00:00-05:00`, so
the S25 store of task 4.4, dropping deliveries expiring before
`2026-11-10T00:00:00Z`, fails it.

### 14. The index and the counted sections

`pack/clauses.toml` names commit `006fc34`, and takes in A.14,
`[100 … 107]`, after A.12, and B.7, `[108 … 113]`, after B.6. Its
header says that 94 to 99, statechart kinds and the engine, stay out
until weir's statechart change takes them in with their cases.
`COUNTED_SECTIONS` holds A.14 beside A.1 to A.10, and 71 stays counted
beside them.

The way not taken is taking in 94 to 99 now. 94 and 95 stand in A.8,
which the check counts, and weir reads no statechart kind, so the check
would fail for want of their cases. Leaving A.14 out until then was not
taken either: 67 counts it, and the cases are here.

## Risks / Trade-offs

- The statement's commit 006fc34 is on a blueprints branch the
  orchestrator pushes. Until it lands on blueprints' main, the index
  names a commit main lacks. If the branch is rebased before it lands,
  the index and the rules' citations move to the landed commit in the
  bolt that holds this change.
- A store that rewrites a record's spelling, a time's offset or a
  message's line ends, fails the pack, since records compare as
  written. A deliverer's store keeps records as the crate spells them.
- `hand` with `confirms` returns three records that a store must add
  together. `add` takes every one or none (112). A store that cannot
  add atomically breaks 103, and the pack's run cannot see that.

## Migration Plan

Nothing moves in flume or sluice in this change. flume moves `EndHeld`'s
deliveries onto the crate in a change of its own, and sluice its
outcomes after it, each pinning weir at the revision of weir's main
that lands this change. weir publishes no release and tags none.
