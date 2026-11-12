## MODIFIED Requirements

### Requirement: The pack keeps an index of the statement's clauses

The pack SHALL hold, at `pack/clauses.toml`, the clause numbers of the statement in boomdrivedev/blueprints `design/weir/requirements.md`, each under the section it stands in (A.1 to A.12, A.14, B.1 to B.7), naming the commit of the statement it was read from, blueprints 006fc34. It SHALL hold every number of that commit but 94 to 99, statechart kinds and the engine, which weir's statechart change takes in with their cases.

#### Scenario: The index read
- **WHEN** the index is read
- **THEN** it names the statement's commit, every section of Parts A and B but A.13, and under each the numbers that section holds: 75a under B.2, 100 to 107 under A.14, and 108 to 113 under B.7 among them
