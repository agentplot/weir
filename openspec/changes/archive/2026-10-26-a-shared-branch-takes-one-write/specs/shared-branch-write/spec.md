# Spec Delta

## Purpose

How every writer of a shared branch reads it and writes it through the one
protocol, so that no write is lost and no two writes move the branch from
one tip (statement 44 to 48, 50, 78 to 80; C.3).

## ADDED Requirements

### Requirement: A branch is read at one tip

Reading a branch SHALL give the commit it stands at, or that it has no tip when the branch does not exist, and every file at that commit SHALL be readable whole from it however the branch moves after (78, 80). Reading SHALL change nothing on the branch (I7).

#### Scenario: A tip read stays readable
- **WHEN** a writer reads a branch at commit A, and another write then moves it to B
- **THEN** every file at A still reads as it stood at A, and the branch stands at B

#### Scenario: A branch that does not exist
- **WHEN** a writer reads a branch the repository does not hold
- **THEN** the read gives no tip and no files, and the repository holds no new branch

### Requirement: A write is formed against the tip read and checked there

The protocol SHALL hand the write's caller the tip read and its files, and take back the files the write adds or replaces, nothing to write, or a refusal naming why (44, 46). A refusal SHALL be reported naming the tip it was made at, and the branch SHALL not move. Nothing to write SHALL be reported as such, and the branch SHALL not move.

#### Scenario: A check refuses the write at the tip
- **WHEN** a write's caller, handed the tip, refuses because a signal it moves already has a standing move
- **THEN** the write is reported refused with the caller's reason and the tip, no commit is made, and the branch stands where it was

#### Scenario: Nothing to write
- **WHEN** a write's caller finds the key it would take already held at the tip and gives back nothing to write
- **THEN** no commit is made, the branch stands where it was, and the write is reported as having written nothing

### Requirement: Every record file a write changes passes recfix before it lands

Every file ending `.rec` that a write adds or replaces SHALL pass `recfix`, run on the file's new text, before the write is committed; a file it refuses SHALL refuse the write, naming the file and `recfix`'s words, and the branch SHALL not move (46; C.2, C.3 step 2; core.24).

#### Scenario: A record recfix refuses
- **WHEN** a write adds `inbox/moves/moves.rec` holding a line that is no field of a record
- **THEN** the write is refused naming `inbox/moves/moves.rec` and what `recfix` said, and no commit is made

### Requirement: A write is one commit on the tip read, touching no working copy

A write SHALL be exactly one commit whose one parent is the tip read (none when the branch has no tip), whose tree is that tip's tree with the write's files added or replaced and nothing else changed, made through a temporary index in a git directory with no working tree (45, 47; C.3 step 3; core.27).

#### Scenario: Only the write's files change
- **WHEN** a write replaces `inbox/moves/moves.rec` on a tip holding fifty other files
- **THEN** the commit's parent is the tip, its tree differs from the tip's in that file alone, and no working tree on the writer's machine has changed

### Requirement: A write lands only while the branch stands at the tip read

The branch SHALL be moved to the write's commit only while it stands at the tip the write was formed against: on a remote by a push naming that tip as the expected old value and never with force, or, for a branch that did not exist, only while it still does not (44, 79; C.3 step 4; core.27).

#### Scenario: The branch stands where it was read
- **WHEN** a writer reads a branch at A and no one moves it before the push
- **THEN** the branch moves to the write's commit, whose parent is A

#### Scenario: The first write makes the branch
- **WHEN** a writer reads a branch that does not exist and writes to it
- **THEN** the branch exists at the write's commit, which has no parent

### Requirement: A moved branch has the write formed again on the new tip

When the branch has moved since the tip read, the protocol SHALL read the new tip and form the write again there, every check made again on it, then move the branch from that tip (44, 46; C.3 step 5; core.26). Two writers' writes from one tip SHALL never both land on it (45, 80).

#### Scenario: Two writes from one tip both land, one after the other
- **WHEN** two writers read one branch at A, the first lands its write, and the second's push is refused
- **THEN** the second write is formed again on the first's commit and lands there, and the branch holds both writes in two commits

#### Scenario: Two machines take one release at once (S5)
- **WHEN** two writers on two machines read one branch at A and each would add the capture of one key, and the first lands
- **THEN** the second, formed again on the first's commit, finds the key held there and writes nothing, and the branch holds one capture of that key

#### Scenario: Two moves of one signal in one second (S6)
- **WHEN** two writers read one branch at A and each would add a standing move for one signal, and the first lands
- **THEN** the second, formed again on the first's commit, is refused naming the move that stands, and the branch holds one move for that signal

### Requirement: A write is replayed at most the writer's bound, five unless set

The protocol SHALL form a write again on a moved branch at most as many times as the writer's bound, five when the writer sets none, so that a write is tried at most one time more than its bound; then it SHALL report the write refused, naming how many times it was tried and that the branch kept moving, and SHALL never try it again on its own (44; record 0002, ruling 4; core.26).

#### Scenario: A branch that keeps moving
- **WHEN** a writer sets the bound to two, and another write lands between each of its reads and its push
- **THEN** the write is tried three times, reported refused naming the three tries, and the branch holds none of it

#### Scenario: The bound left out
- **WHEN** a writer sets no bound, and another write lands between each of its reads and its push
- **THEN** the write is tried six times and then reported refused

### Requirement: A push refused while the branch stands still is a failure, not a replay

When a push is refused and a fresh read finds the branch still at the tip the write was formed against, the protocol SHALL report the write failed with git's words and SHALL not form it again (44).

#### Scenario: A remote that refuses every push
- **WHEN** a write is pushed to a remote that refuses it for a reason other than the branch having moved
- **THEN** the write is reported failed, naming what git said, after one try

### Requirement: A branch with no remote takes the same write

A branch kept in a repository on the writer's machine alone SHALL be written through the same steps, its commit made in that repository's git directory through a temporary index, and the branch moved with `update-ref` from the tip read as the expected old value; a branch that has left that tip SHALL refuse the move, and the write SHALL be formed again on the new tip as on a remote (48; C.3; S7; core.27).

#### Scenario: A write formed against the old tip (S7)
- **WHEN** an inbox lies on a branch of a repository with no remote, a writer reads it at A, and another write moves it to B before the first moves it
- **THEN** the move from A is refused, the write is formed again on B and lands there, and both writes stand

### Requirement: Every write names what it did and who asked

Every write SHALL be handed what it does, one line, and who asked for it, and its commit SHALL name both: its message's subject is what it did, and its message ends with a line naming who asked. A write handed an empty or multi-line account of either SHALL be refused before anything is read (50; C.3 step 3; core.29).

#### Scenario: The commit's message
- **WHEN** the operator at one machine asks for a write that takes the capture `release/acme/v1`
- **THEN** the commit's subject is what the writer said it did, and its message names the operator as who asked

#### Scenario: No one named
- **WHEN** a write is asked for with who asked left empty
- **THEN** it is refused naming that field, and nothing is read or written

### Requirement: No credential is passed on a command line

The protocol SHALL refuse a remote whose address holds a password, naming the address without it, so that no secret reaches a command line, a commit or a report; reaching a remote is left to git's own credential helpers (71; core.3).

#### Scenario: A password in the address
- **WHEN** a writer is handed the remote `https://bot:s3cret@example.org/inbox.git`
- **THEN** the write is refused before git runs, and the refusal does not hold `s3cret`
