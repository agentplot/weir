## MODIFIED Requirements

### Requirement: Every line is exactly one of three

A line SHALL be read as an offer, a cursor or a completion when it is one JSON object holding exactly one of the keys `capture`, `cursor` and `done`; any other line SHALL be refused as malformed, naming its reason (1; C.1; core.5). A top-level key the contract does not name SHALL be ignored, not refused (core.6). A line holding `import` SHALL be refused as malformed, its reason saying an import line is read by a host's import operation and never taken, whatever else it holds (63, 64; C.1).

#### Scenario: One key of three reads as that line
- **WHEN** the lines `{"capture": {…}}`, `{"cursor": "2026-10-17T04:30:05Z"}` and `{"done": {"listed": 14, "failed": []}}` are read
- **THEN** they read as an offer, a cursor and a completion, in that order

#### Scenario: A line with none or two of the keys is malformed
- **WHEN** the line `{"cursor": "a", "done": {"listed": 0, "failed": []}}` is read, and then `{"note": 1}`, and then `not json`
- **THEN** each is refused as malformed with a reason naming what it lacks or holds twice

#### Scenario: An unknown top-level key is ignored
- **WHEN** the line `{"cursor": "c1", "trace": "x"}` is read
- **THEN** it reads as the cursor `c1`

#### Scenario: An import line is not an adapter's line
- **WHEN** the line `{"import": {…}}` is read, and then `{"import": {…}, "capture": {…}}`
- **THEN** each is refused as malformed, the reason naming it an import line for a host's import operation, and `take` names it among the run's refusals and goes on to the next line
