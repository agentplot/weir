## ADDED Requirements

### Requirement: A delivery case holds a store's starting records and one call

A delivery case SHALL hold, in its folder under `pack/delivery/`, `case.toml` naming its clauses in `satisfies` and one call of `record`, `hand`, `confirm` or `owed` with its arguments, and `start.rec`, the store's starting records in recutils, which may be absent (C.7, C.8). A field the case format does not name SHALL fail the case's read, naming the case and the field.

#### Scenario: A misspelt call
- **WHEN** a delivery case names `[call.confrim]`
- **THEN** reading it fails, naming the case and `confrim`

### Requirement: A delivery case names one thing that must come of its call

A delivery case SHALL name exactly one of: `added.rec`, the records the call adds in order; that the key is held, for `record`; when the reader first confirmed, for `hand` and `confirm`; the refusal, by its field and a phrase of its reason; or, for `owed`, each reader with its deliveries, each by its key, owed or expired, with the ways of its hand-overs, whether the name is known, and the keys done (C.7).

#### Scenario: A case that expects both records and a refusal
- **WHEN** a delivery case holds `added.rec` and a refusal
- **THEN** reading it fails, naming the case

### Requirement: The pack runs a delivery case through a store its caller hands it

`weir-pack` SHALL run a delivery case through an empty store its caller hands it, as C.7 says: add the starting records, read them back, make the call with `weir-delivery` over what it read, add what the call returns, and read the store back. It SHALL compare the store with the starting records followed by `added.rec`, record by record, or the answer with the case's, naming each difference (65, 87, 112, 113; core.52).

#### Scenario: The cases through a store of each spelling
- **WHEN** weir-pack's tests run every delivery case through a store holding its records as a list, one holding them as recutils text, and one holding them as JSON lines
- **THEN** every case passes through each store

### Requirement: The pack fails a store that does not read back what it added

The run SHALL name, for each case a store fails, every record it added and did not read back as written, and every record it read back that the case does not expect. It SHALL run every delivery case of a pack, each through a fresh store its caller makes, so a deliverer runs it over its own store (87, 113; S25; core.35).

#### Scenario: A store that drops expired deliveries (S25)
- **WHEN** weir-pack's tests run the delivery cases through a store that leaves out of its reads every delivery whose `expires_at` is before `2026-11-10T00:00:00Z`
- **THEN** the run fails `an-expired-delivery-is-shown`, naming the `weir-delivery/1` record it did not read back

#### Scenario: A store that loses known names
- **WHEN** weir-pack's tests run the delivery cases through a store that adds no `weir-known/1` record
- **THEN** the run fails `a-name-stays-known`, naming the `weir-known/1` record it did not read back

### Requirement: The delivery cases hold A.14

The pack SHALL hold delivery cases exercising between them a delivery recorded and refused, a key held, a confirmation taken, repeated and refused, a hand-over with and without a confirmation on receipt, an expired delivery shown and confirmed, the order recorded, a name known and not, and a delivery done and not done. Every clause of 100 to 107 SHALL be cited by at least one delivery case (67; core.35).

#### Scenario: The cases named
- **WHEN** the delivery cases of the pack are read
- **THEN** a case of each thing this requirement names is among them, each reads, and every clause of 100 to 107 is cited by one

### Requirement: The delivery cases run from fixed inputs alone

Every delivery case SHALL run from its own files alone, its times written in it, with no clock, no network and no source, and SHALL give the same verdict on every machine (68; core.35).

#### Scenario: A case read from its folder
- **WHEN** a delivery case is run
- **THEN** it reads only the files of its folder, and every time it uses is one the case writes
