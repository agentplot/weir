## MODIFIED Requirements

### Requirement: A move of any word is carried as it stands

A move SHALL be read whatever its word and whether or not it carries its consequence, so the import operation can leave its signal unmoved and report it (39; core.25); only a move whose `signal` is not a whole number or whose `move` is not a non-empty string SHALL refuse the line, naming the field.

#### Scenario: A word not of the six
- **WHEN** an import line carries a move `{"signal": 1, "move": "promote", "by": "curation", "at": "2026-10-12T00:00:00Z"}`
- **THEN** it reads with that move, word `promote`, unchanged

#### Scenario: A drop with no reason
- **WHEN** an import line carries a move `{"signal": 1, "move": "drop"}`
- **THEN** it reads with that move, no target and no note

#### Scenario: A move with its note
- **WHEN** an import line carries a move `{"signal": 1, "move": "attach", "target": "intent/x", "note": "the same ask as the standup's"}`
- **THEN** it reads with that move, its note `the same ask as the standup's` as written

## ADDED Requirements

### Requirement: A move holds its signal and its word, and keeps what the body wrote with it

`moves` SHALL be a list, possibly empty, whose entries hold `signal`, a whole number, and `move`, a non-empty word, and MAY hold `target`, `note`, `by` and `at`, strings kept as written, `note` the reason or note the body gave with the move, for the import operation to keep with it (31, 39; C.1, C.2).

#### Scenario: A move with its note
- **WHEN** an import line carries a move `{"signal": 1, "move": "attach", "target": "intent/x", "note": "the same ask as the standup's"}`
- **THEN** it reads with that move, its note `the same ask as the standup's` as written
