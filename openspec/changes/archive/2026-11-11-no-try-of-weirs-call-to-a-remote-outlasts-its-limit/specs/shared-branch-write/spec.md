## ADDED Requirements

### Requirement: Each call to the remote is one try to its limit

The protocol SHALL run each git call that reaches a branch's remote, the fetch of its tip, the listing that says whether a branch not fetched exists, and the push, as one try through `weir-bounded` to `TRY_FOR`, ended at that limit with everything it started (69, C.3 as amended at blueprints ad3a1ae; core.53). Every git call that reaches no remote SHALL run to its end as before.

#### Scenario: A remote that never answers a fetch
- **WHEN** a branch's remote takes the call and never answers, and the branch's limit is 300 ms
- **THEN** reading the branch fails at the read step in under five seconds, naming `git fetch`, the remote's address and `300ms`, and the process the call started is gone

### Requirement: A stopped read fails the write and lets its lock go

A stopped fetch or listing SHALL fail the read at its read step, and the write that read it, and the machine's writers' lock SHALL be let go as on any failure (49; C.3 as amended at blueprints ad3a1ae; core.28, core.53).

#### Scenario: A write whose remote never answers
- **WHEN** a write is asked of a branch whose remote never answers, and the branch's limit is 300 ms
- **THEN** the write fails at the read step naming the stopped fetch, and the machine's writers' lock is let go

### Requirement: A stopped push fails the write and is never replayed

A stopped push SHALL fail the write at its move step. It is no refusal: the protocol SHALL neither read the branch again nor form the write again (44; C.3 as amended at blueprints ad3a1ae; core.26, core.53).

#### Scenario: A remote that never answers a push
- **WHEN** a commit is moved onto a branch whose remote never answers, and the branch's limit is 300 ms
- **THEN** the move fails at the move step, naming `git push`, the remote's address and `300ms`, and is not a refusal

### Requirement: A stopped call's failure names its call, its remote and its limit

A stopped call's failure SHALL be git's last line on standard error and `; `, where git wrote one, then `weir-bounded`'s words naming `git fetch`, `git ls-remote` or `git push`, the remote's address and the limit. Its standard output SHALL never be read as an answer (C.3 as amended at blueprints ad3a1ae; core.3, core.53).

#### Scenario: A stopped call's failure follows git's last line
- **WHEN** a stopped fetch's git wrote two lines on standard error, the last `fatal: x`, and then a stopped fetch's git wrote nothing there
- **THEN** the first failure is `fatal: x; ` then the stopped call's words, and the second is the words alone
