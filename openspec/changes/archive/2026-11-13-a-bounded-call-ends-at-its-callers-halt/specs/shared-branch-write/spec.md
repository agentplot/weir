## ADDED Requirements

### Requirement: A read or a write ends its tries at its caller's halt

A branch SHALL be given its caller's halt by `with_halt`, and each try of a read or a write of that branch, the fetch, the listing and the push, SHALL take that halt beside its limit and end at whichever comes first. A branch given no halt SHALL run as before (C.3; record 0008 as amended 2026-11-12; core.53).

#### Scenario: A halt ends a fetch its remote never answers
- **WHEN** a branch's remote takes the call and never answers, the branch's limit is the family's, and its caller's halt is set 300 ms after the read begins
- **THEN** reading the branch fails at the read step in under five seconds, naming `git fetch`, the remote's address and that its caller asked it stopped, and naming no limit, and the process the call started is gone

#### Scenario: A read begun with its halt set reaches no remote
- **WHEN** a branch whose remote never answers is read with its caller's halt already set
- **THEN** the read fails at the read step, naming `git fetch` and that its caller asked it stopped, and the remote was never called

### Requirement: A halted read fails the write and lets its lock go

A halted fetch or listing SHALL fail the read at its read step, and the write that read it, and the machine's writers' lock SHALL be let go as on any failure (49; C.3; record 0008 as amended 2026-11-12; core.28, core.53).

#### Scenario: A write halted while it reads lets its lock go
- **WHEN** a write is asked of a branch whose remote never answers, the branch's limit is the family's, and its caller's halt is set 300 ms after the write begins
- **THEN** the write fails at the read step in under five seconds, naming the halted fetch, and the machine's writers' lock is let go

### Requirement: A halted push fails the write and is never replayed

A halted push SHALL fail the write at its move step. It is no refusal: the protocol SHALL neither read the branch again nor form the write again (44; C.3; record 0008 as amended 2026-11-12; core.26, core.53).

#### Scenario: A halt ends a push its remote never answers
- **WHEN** a commit is moved onto a branch whose remote never answers, the branch's limit is the family's, and its caller's halt is set 300 ms after the move begins
- **THEN** the move fails at the move step in under five seconds, naming `git push`, the remote's address and that its caller asked it stopped, it is not a refusal, and the process the call started is gone

### Requirement: A branch with no remote takes no halt

A branch with no remote makes no try, and its calls SHALL run to their end whatever its halt says (48; record 0008 as amended 2026-11-12; core.53).

#### Scenario: A branch with no remote takes no halt
- **WHEN** a write is asked of a branch with no remote, given a halt already set
- **THEN** it lands

### Requirement: A halted call's failure names its call and its remote

A halted call's failure SHALL be git's last line on standard error and `; `, where git wrote one, then `weir-bounded`'s words for a halted call naming `git fetch`, `git ls-remote` or `git push` and the remote's address. Its standard output SHALL never be read as an answer (C.3; record 0008 as amended 2026-11-12; core.3, core.53).

#### Scenario: A halted call's failure follows git's last line
- **WHEN** a halted push's git wrote two lines on standard error, the last `fatal: x`, and then a halted push's git wrote nothing there
- **THEN** the first failure is `fatal: x; ` then the halted call's words, and the second is the words alone
