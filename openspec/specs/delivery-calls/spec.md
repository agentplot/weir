# delivery-calls Specification

## Purpose
`record`, `hand`, `confirm` and `owed`, the calls a deliverer makes over
the records it passes in, with what each returns or refuses, and the
store seam a deliverer implements (100 to 113).

## Requirements

### Requirement: The calls are functions over the records their caller passes

`record`, `hand`, `confirm` and `owed` SHALL each answer from the records, the arguments and the times their caller passes, and from nothing else: no clock, file, store or environment. A call that adds SHALL return the records to add and write none. The same records, arguments and times SHALL always give the same answer (69, 108 to 113; core.1, core.51).

#### Scenario: A call made twice
- **WHEN** `owed` is called twice with the same records and the same time
- **THEN** both answers are equal, and the records passed are unchanged

### Requirement: record returns the delivery to add

`record` SHALL take the records, a key, a message, the readers, a time and a span in seconds, and SHALL return the one `weir-delivery/1` record to add, `recorded_at` the time given and `expires_at` that time and the span (100, 104, 108; core.47).

#### Scenario: A delivery recorded for two readers
- **WHEN** `record` is called with no records, the key `place/7f3a/end/2`, readers `planner` and `reviewer`, the time `2026-11-08T10:02:00-05:00` and a span of 604800
- **THEN** it returns one `weir-delivery/1` record naming both readers in that order, recorded at that time and expiring at `2026-11-15T10:02:00-05:00`

### Requirement: record holds a key once

Where the records hold a delivery under the key, `record` SHALL add nothing and return the first delivery they hold under it, whatever the later call carries (101; core.47).

#### Scenario: One key recorded twice (S24)
- **WHEN** `record` is called with records holding a delivery under `place/7f3a/end/2`, the same key and another message
- **THEN** it adds nothing and returns the delivery the records hold, with its first message

### Requirement: record refuses a delivery out of its shape

`record` SHALL refuse, naming the field: a key, a message or a reader out of C.8's shape; no reader (`readers`); a reader named twice (`readers`, naming the reader); and a span under one second (`span`) (100, 104; core.47).

#### Scenario: A delivery with no reader
- **WHEN** `record` is called with no reader
- **THEN** it is refused naming `readers`

#### Scenario: A reader named twice
- **WHEN** `record` is called with the readers `planner` and `planner`
- **THEN** it is refused naming `readers` and `planner`

#### Scenario: A span of no seconds
- **WHEN** `record` is called with a span of 0
- **THEN** it is refused naming `span`

### Requirement: owed gives every delivery a reader has not confirmed

`owed` SHALL take the records and a time, and SHALL give each reader with something unconfirmed every delivery naming it that it has not confirmed, in the order recorded, with each hand-over of it to that reader. It SHALL leave out nothing a reader has not confirmed (105, 111; I9; core.49).

#### Scenario: Deliveries in the order recorded
- **WHEN** the records hold three deliveries for `planner`, recorded in the order `c`, `a`, `b`
- **THEN** `owed` gives them to `planner` in the order `c`, `a`, `b`

#### Scenario: A delivery one reader has confirmed
- **WHEN** the records hold a delivery for `planner` and `reviewer` and a confirmation by `planner` alone
- **THEN** `owed` gives the delivery owed to `reviewer`, not to `planner`, and does not list it done

### Requirement: owed marks a delivery past its expiry expired

`owed` SHALL mark a delivery expired for a reader once the time given is at or past its `expires_at`, and owed before. It SHALL give an expired delivery as it gives an owed one (104, 111; core.49).

#### Scenario: An expired delivery for a misspelt reader (S23)
- **WHEN** `owed` is called past the expiry of a delivery naming `planer`, which no confirmation and no `weir-known/1` record names
- **THEN** it gives `planer` with that delivery marked expired, and its name not known

### Requirement: owed says whether a reader's name is known

`owed` SHALL say of each reader whether its name is known, from `weir-known/1` records alone (106, 111; core.50).

#### Scenario: A name stays known after its deliveries are let go of
- **WHEN** the records hold a `weir-known/1` record for `planner` and one delivery owed to it, and no confirmation by it
- **THEN** `owed` gives `planner` with its name known

### Requirement: owed lists the deliveries done

`owed` SHALL list every delivery every one of whose readers has confirmed it, in the order recorded, and SHALL give it to no reader (107, 111).

#### Scenario: A delivery done when every reader confirms (S22)
- **WHEN** the records hold a delivery for `planner` and `reviewer`, a hand-over to `planner`, and a confirmation by each
- **THEN** `owed` gives neither reader that delivery, and lists it done

### Requirement: confirm returns the confirmation to add

`confirm` SHALL take the records, a key, a reader and a time, and SHALL return a `weir-confirmation/1` record to add. Where no `weir-known/1` record of the records names the reader, it SHALL return one beside it, `known_at` the time given. It SHALL confirm an expired delivery as an owed one (102, 104, 106, 110; core.48, core.50).

#### Scenario: A reader confirms by its key
- **WHEN** `confirm` is called for `planner` on a delivery naming it, the name not yet known
- **THEN** it returns a `weir-confirmation/1` and a `weir-known/1` record for `planner`

#### Scenario: An expired delivery confirmed
- **WHEN** `confirm` is called for `planner` past the delivery's expiry
- **THEN** it returns the confirmation, and `owed` over the records with it added gives the delivery to `planner` no more

### Requirement: confirm takes a reader's confirmation once

Where the records hold the reader's confirmation of the key, `confirm` SHALL add nothing and say when the reader first confirmed (102; core.48).

#### Scenario: A second confirmation (S24)
- **WHEN** `confirm` is called for `planner` on a delivery whose confirmation by `planner` the records hold
- **THEN** it adds nothing and says when `planner` first confirmed

### Requirement: confirm refuses a key or a reader the records do not hold

`confirm` SHALL refuse a key no delivery of the records is under, naming `key`, and a reader the delivery does not name, naming `reader` (102; core.48).

#### Scenario: A confirmation of a key no delivery is under
- **WHEN** `confirm` is called with the key `place/0000/end/1`, which no delivery of the records is under
- **THEN** it is refused naming `key` and `place/0000/end/1`

### Requirement: hand returns a hand-over, which confirms nothing by itself

`hand` SHALL take the records, a key, a reader, a way, a time and whether the reader confirms on receipt, and SHALL return a `weir-handover/1` record to add. A hand-over SHALL leave the delivery owed to its reader (102, 103, 109; core.48).

#### Scenario: A hand-over by a notice
- **WHEN** `hand` is called for a delivery owed to `planner`, by `notice`, not confirming on receipt
- **THEN** it returns one `weir-handover/1` record, and `owed` over the records with it added still gives the delivery owed to `planner`, with that hand-over

### Requirement: A reader confirms on receipt in the call that hands it over

Where the reader confirms on receipt, `hand` SHALL return, after the hand-over, what `confirm` returns for that reader in the same answer (103; core.48).

#### Scenario: A reader that confirms on receipt
- **WHEN** `hand` is called for a delivery owed to `reviewer`, a name no reader has confirmed under, by `session-start`, confirming on receipt
- **THEN** it returns a `weir-handover/1`, a `weir-confirmation/1` and a `weir-known/1` record, in that order

### Requirement: hand hands nothing to a reader that confirmed or is not named

Where the records hold the reader's confirmation of the key, `hand` SHALL add nothing and say when the reader first confirmed. It SHALL refuse a key no delivery is under (`key`), a reader the delivery does not name (`reader`) and a way out of C.8's shape (`by`) (103; core.48).

#### Scenario: A hand-over to a reader that has confirmed
- **WHEN** `hand` is called for a reader whose confirmation of the delivery the records hold
- **THEN** it adds nothing and says when that reader confirmed

#### Scenario: A hand-over to a reader the delivery does not name
- **WHEN** `hand` is called for `stranger` on a delivery naming `planner` alone
- **THEN** it is refused naming `reader` and `stranger`

### Requirement: A deliverer's store is a seam of two calls

`weir-delivery` SHALL define the store a deliverer implements as two calls: `read`, giving every record the store holds in the order added, and `add`, holding the records given after the others, every one or none, each saying why where it failed. `weir-delivery` SHALL implement no store (112, 113; core.51, core.52).

#### Scenario: A store implemented outside the crate
- **WHEN** a test implements the store over a list of its own
- **THEN** the pack's run takes it as a store, adding to it and reading it back
