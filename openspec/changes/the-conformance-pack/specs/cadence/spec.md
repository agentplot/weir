# Spec Delta

## MODIFIED Requirements

### Requirement: The pack holds cadence's cases

The conformance pack SHALL hold cases of cadence's shape (C.7), one
folder each at `pack/cadence/<case>/case.toml`, as every case of the
pack lies: each a schedule, a last run or none, a now with the zone it is read in, and
whether the thing is due, citing the clauses it exercises (67). Every
case SHALL be answered by cadence as it says, the pack SHALL hold a
case citing 59 and a case citing 60, and a case that leaves out a field
of the shape, names one the shape does not hold, or gives a now with no
zone SHALL fail naming its case.

#### Scenario: Every case is answered as it says

- **WHEN** each cadence case of the pack is given to cadence
- **THEN** cadence says due for every case that says due, and not due
  for every case that says not due

#### Scenario: The two clauses are cited

- **WHEN** the cadence cases of the pack are read
- **THEN** at least one cites 59 and at least one cites 60

#### Scenario: A case that never ran

- **WHEN** a cadence case gives no last run
- **THEN** it is read as a thing that has never run, and cadence says
  due

#### Scenario: A case of the wrong shape

- **WHEN** a cadence case names `last-run` in place of `last_run`, or
  gives `now` as `2026-11-03T09:20:00-05:00` with no zone, or cites no
  clause
- **THEN** the case fails, naming its case and the field

#### Scenario: One folder per case

- **WHEN** the cadence cases of the pack are listed
- **THEN** each is a folder under `pack/cadence/` holding its
  `case.toml`, and no case is a file of its own there
