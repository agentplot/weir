# Spec Delta

## ADDED Requirements

### Requirement: The report is written and read as weir-report/1

A report of a run, an intake or an import SHALL be written as one JSON object on one line naming `"format": "weir-report/1"`, holding the keys taken in the order of their lines, the count held, each refusal with its line, its key where it had one and its reason, the listing's count where a completion was read, each failure the completion named with its `what` and `why`, the cursor kept where one was, and whether the run was complete (23, 76; C.7; core.22). The same object SHALL read back into the report it was written from, and an object naming another format, or lacking a field, SHALL be refused naming why.

#### Scenario: A run's report written and read back
- **WHEN** a run's report holds two keys taken, one held, a refusal of line 3 for key `moves/x`, a listing of fourteen with one failure, the cursor `c2` kept, and complete
- **THEN** it is written as one line naming `weir-report/1`, and reading that line gives the same report

#### Scenario: Another format
- **WHEN** the object `{"format": "weir-report/2", ...}` is read
- **THEN** it is refused, naming the format
