# Spec Delta

## Purpose

The one kind format of the family: how a tool, a repository or a launch
declares the kinds of work it runs, and how every reader reads and
checks such a declaration the same way (51 to 58, 81, 82, C.4).

## ADDED Requirements

### Requirement: A declaration reads into its kinds
A declaration SHALL read into every kind under its `kinds` table, each
with its version, its stages in the order written, its branch prefix,
whether its first stage starts unready and whether it lands (51, 52,
C.4). A version left out SHALL read as 1, a branch left out as the
kind's name, and `starts_unready` and `lands` left out as false.

#### Scenario: A kind reads with every field
- **WHEN** a declaration holds `kinds.bump` with `version = 2`, `branch = "bump"`, `starts_unready = false`, `lands = true` and two stages, `code` then `review`
- **THEN** the read holds one kind, `bump`, at version 2, with branch prefix `bump`, not starting unready, landing, and its stages `code` then `review` in that order

#### Scenario: Fields left out read as the format says
- **WHEN** a kind declares only its stages
- **THEN** it reads at version 1, with its own name as its branch prefix, not starting unready and not landing

#### Scenario: A declaration with no kinds
- **WHEN** a declaration holds no `kinds` table
- **THEN** it reads with no kinds and is not refused

### Requirement: A stage reads with its name, prompt and end
A stage SHALL read with its name, its prompt as written, and either its
one deliverable or that it is a conversation (53). It SHALL read its
agents with every setting each holds, whether it waits on its caller's
word (false when left out), how long its agent may be quiet, the stage a
failed submission sends a place back to (`on_fail`) and how many times
a place may go back (`max_send_backs`), the last three only when
declared.

#### Scenario: A stage reads with its settings
- **WHEN** a stage declares `name = "review"`, a prompt, `conversation = true`, `wait = true`, `agents = [{ beside = "operator" }]`, `on_fail = "code"` and `max_send_backs = 3`
- **THEN** it reads as a conversation named `review` that waits on its caller's word, with one agent holding `beside = "operator"`, sending a failed place back to `code` at most three times

#### Scenario: A stage's quiet time reads as a duration
- **WHEN** a stage declares `quiet = "45m"`, and another `quiet = "1h30m"`
- **THEN** the first reads as forty-five minutes and the second as ninety

### Requirement: A deliverable is one of six spellings
A stage that is not a conversation SHALL declare exactly one
deliverable, spelled `commits`, `change`, `tasks`, `archived`,
`report:<path>` or `committed:<path>`, and a conversation SHALL declare
none (51, 54, C.4).

#### Scenario: Each spelling reads as its form
- **WHEN** six stages declare `commits`, `change`, `tasks`, `archived`, `report:out/conclusion.md` and `committed:docs/verify.md`
- **THEN** they read as commits, a change written, tasks ticked, a change archived, a report at `out/conclusion.md` and a committed report at `docs/verify.md`

#### Scenario: Two deliverables are refused
- **WHEN** a stage declares `deliverables = ["commits", "tasks"]`
- **THEN** the declaration is refused naming that stage's `deliverables` field (S12)

#### Scenario: A spelling not of the six is refused
- **WHEN** a stage declares `deliverables = ["commit"]`, or `deliverables = ["report:"]`
- **THEN** the declaration is refused naming that stage's `deliverables` field

#### Scenario: A stage with neither a deliverable nor a conversation
- **WHEN** a stage declares no `deliverables` and no `conversation = true`
- **THEN** the declaration is refused naming that stage's `deliverables` field

#### Scenario: A conversation with a deliverable
- **WHEN** a stage declares `conversation = true` and `deliverables = ["commits"]`
- **THEN** the declaration is refused naming that stage's `deliverables` field

### Requirement: A declaration is checked whole and refused naming the field
A declaration SHALL be refused whole when any kind has no stage, any
stage has no name or no prompt, any field the format names has the wrong
shape, or any deliverable is not of the six (57). The refusal SHALL name
every field that fails, each by its path and with its reason, and SHALL
return no kind (81).

#### Scenario: A kind with no stage is refused
- **WHEN** a kind declares no stages, or `stages = []`
- **THEN** the declaration is refused naming that kind's `stages` field (S12)

#### Scenario: A stage with no prompt is refused
- **WHEN** the second stage of `bump` has a name and no prompt
- **THEN** the declaration is refused naming `kinds.bump.stages[1].prompt`

#### Scenario: A field of the wrong shape is refused
- **WHEN** a kind declares `version = "two"`, `version = 0` or `lands = "yes"`, or a stage declares `quiet = 45`
- **THEN** the declaration is refused naming that field

#### Scenario: A send-back naming no stage of its kind is refused
- **WHEN** a stage of `bump` declares `on_fail = "deploy"` and `bump` has no stage named `deploy`
- **THEN** the declaration is refused naming that stage's `on_fail` field

#### Scenario: Two stages of one kind share a name
- **WHEN** two stages of one kind are both named `code`
- **THEN** the declaration is refused naming the second one's `name` field

#### Scenario: Every failing field is named
- **WHEN** one kind has no stage and another kind's stage has no prompt
- **THEN** the one refusal names both fields, and no kind is returned

#### Scenario: Text that is not TOML is refused
- **WHEN** a declaration's text does not parse as TOML
- **THEN** it is refused saying where in the text it fails and why

### Requirement: A section the reader does not know never fails the read
A table or field the format does not name SHALL be returned unread and
unrefused, beside the declaration, the kind or the stage that holds it
(56, 82). Nothing in such a section SHALL make a declaration refused.

#### Scenario: A tool's own table beside the kinds
- **WHEN** a declaration holds a `[profiles]` table beside its kinds
- **THEN** its kinds read as they would without it, and the `profiles` table is returned as written

#### Scenario: An unknown field on a stage
- **WHEN** a stage holds a field the format does not name, such as `colour = 3`
- **THEN** the stage reads, and the field is returned as written beside it

### Requirement: An elaboration kind's section is read only by whoever runs elaborations
A kind's `elaboration` section SHALL read, for a reader of places, as a
section it does not know (55, 56): the kind's stages read and a
malformed section never refuses them. Read as an elaboration, the
section SHALL give its inputs, the sections a conclusion adds, its
ending, one of `self-closing`, `standing` and `with-operator`, whether
earlier transcripts are handed, and what lands, checked whole and
refused naming the field (55, 57).

#### Scenario: Both readers read one elaboration kind
- **WHEN** a declaration holds `writeback` with stages and an elaboration section declaring `inputs`, `conclusion`, `ending = "self-closing"`, `transcripts = false` and `lands`
- **THEN** a read of its kinds gives `writeback`'s stages with the section returned unread, and a read of its elaboration gives every field the section declares, and neither fails on the other's part (S11)

#### Scenario: A malformed elaboration section
- **WHEN** an elaboration section declares `ending = "sometimes"`
- **THEN** a read of the kinds still succeeds, and a read of the elaboration is refused naming `kinds.writeback.elaboration.ending`

#### Scenario: A kind with no elaboration section
- **WHEN** a kind has no `elaboration` table
- **THEN** a read of its elaboration says it has none, and is not refused

### Requirement: The same declaration always reads the same
Reading a declaration SHALL depend on its text alone: the same text
SHALL give the same kinds or the same refusal every time, and reading
SHALL change nothing (58, 82, I7).

#### Scenario: Two kinds read the same twice
- **WHEN** a declaration holds `fix` at version 1, with one stage `code` delivering commits, starting unready and landing, and `change` at version 1, with `construct`, `code` and `verify` delivering a change written, tasks ticked and a change archived
- **THEN** each kind reads with exactly those stages and deliverables, and a second read gives a result equal to the first (S10)

### Requirement: weir check reads a declaration
`weir check <declaration>` SHALL read the file it names as a
declaration, the elaboration sections included. When the declaration
reads, it SHALL list each kind with its version and its stages, each
with its deliverable or that it is a conversation, and exit 0. When it
is refused, it SHALL name each failing field with its reason on standard
error and exit non-zero. It SHALL leave the file as it was.

#### Scenario: A declaration that reads
- **WHEN** `weir check` is given a file declaring `fix` and `change`
- **THEN** it prints `fix@1` and `change@1` with their stages and deliverables, and exits 0

#### Scenario: A declaration that is refused
- **WHEN** `weir check` is given a file whose stage declares two deliverables
- **THEN** it prints that stage's `deliverables` field with its reason on standard error, prints no kind, and exits non-zero

#### Scenario: Checking changes nothing
- **WHEN** `weir check` reads a file
- **THEN** the file's bytes after the check are the bytes before it
