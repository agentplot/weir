# shared-branch-lock Specification

## Purpose
The lock that keeps one machine's writers of a shared branch in turn: what
it names, how a writer waits for it and breaks a stale one, and that it
never stands in for the check at the tip (statement 49; C.3).

## Requirements

### Requirement: A write holds the lock from its read to its landing

A writer SHALL take the lock in the git directory it writes through before it reads the tip, hold it through every try of the write, and let it go when the write lands, writes nothing, is refused or fails, so that one machine's writers sharing that git directory write in turn (49; C.3; core.28).

#### Scenario: Two writers on one machine
- **WHEN** two writers sharing one cache each ask for a write to one branch at the same moment
- **THEN** one holds the lock through its write while the other waits, each write is formed on the tip the other left, and both land with no push refused

#### Scenario: The lock is let go after a refusal
- **WHEN** a write is refused by its caller's check
- **THEN** no lock stands in the git directory afterwards

### Requirement: The lock names its holder and when it was taken

The lock SHALL name who holds it, the writer's account of who asked, and when it was taken, a time read from the clock the caller hands, in RFC 3339 (49; core.28). Taking it SHALL succeed for one writer only when two try at once.

#### Scenario: Reading a held lock
- **WHEN** a write is being formed
- **THEN** the lock in the git directory names that write's asker and the time it was taken

### Requirement: A writer waits for a held lock and breaks a stale one

A writer that finds the lock held SHALL wait until it is let go, or until it is older than the age the writer's caller sets, and then break it once, however many writers find it stale, and take it; the write's report SHALL name a lock it broke, with its holder and time (49; C.3).

#### Scenario: A lock let go while a writer waits
- **WHEN** a writer finds the lock held, taken a moment ago with the stale age a minute, and its holder lets it go a fifth of a second later
- **THEN** the writer takes it and its write lands, breaking nothing

#### Scenario: A stale lock
- **WHEN** a writer finds a lock taken two minutes ago with the stale age a minute
- **THEN** the writer breaks it, takes it, lands its write, and its report names the holder and time of the lock it broke

### Requirement: The lock never stands in for the check at the tip

Holding the lock SHALL never let a write skip the move from the tip read: a writer holding it SHALL still push, or move a branch with no remote, only from the tip it read, and a write from another machine SHALL refuse it as any moved branch does (44, 49; core.28).

#### Scenario: Another machine writes while the lock is held
- **WHEN** a writer holds its machine's lock and has read the branch at A, and a writer on another machine, with its own cache, lands a write moving the branch to B
- **THEN** the first writer's push from A is refused, and its write is formed again on B
