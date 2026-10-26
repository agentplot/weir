# kinds Specification

## Purpose
The one kind format of the family: how a tool, a repository or a launch
declares the kinds of work it runs, and how every reader reads and
checks such a declaration the same way (51 to 58, 81, 82, C.4).

## Requirements

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
agents with their settings, whether it waits (false when left out), and
its `quiet`, `on_fail` and `max_send_backs`, each of the last three
returned left out when left out (52, 53, 58, C.4).

#### Scenario: A stage reads with its settings
- **WHEN** a stage declares `name = "review"`, a prompt, `conversation = true`, `wait = true`, `agents = [{ beside = "operator" }]`, `on_fail = "code"` and `max_send_backs = 3`
- **THEN** it reads as a conversation named `review` that waits on its caller's word, with one agent holding `beside = "operator"`, sending a failed place back to `code` at most three times

#### Scenario: Settings left out are returned left out
- **WHEN** a stage declares no `quiet`, `on_fail` or `max_send_backs`
- **THEN** it reads with each of the three left out, and no value stands in for any of them

### Requirement: A stage's quiet time is one whole number of s, m or h
A stage's `quiet` SHALL be a string holding one whole number followed
by `s`, `m` or `h`, and SHALL read as the duration it names (57, C.4).
Any other value SHALL be refused naming the field.

#### Scenario: A stage's quiet time reads as a duration
- **WHEN** three stages declare `quiet = "45m"`, `quiet = "90s"` and `quiet = "2h"`
- **THEN** they read as forty-five minutes, ninety seconds and two hours

#### Scenario: A quiet time of more than one part is refused
- **WHEN** a stage declares `quiet = "1h30m"`
- **THEN** the declaration is refused naming that stage's `quiet` field

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
shape, any deliverable is not of the six, two stages of one kind share a
name, or an `on_fail` names no stage of its kind (57, C.4). The refusal
SHALL name every field that fails, each by its path and with its reason,
and SHALL return no kind (81).

#### Scenario: A kind with no stage is refused
- **WHEN** a kind declares no stages, or `stages = []`
- **THEN** the declaration is refused naming that kind's `stages` field (S12)

#### Scenario: A stage with no prompt is refused
- **WHEN** the second stage of `bump` has a name and no prompt
- **THEN** the declaration is refused naming `kinds.bump.stages[1].prompt`

#### Scenario: A field of the wrong shape is refused
- **WHEN** a kind declares `version = "two"`, `version = 0` or `lands = "yes"`, or a stage declares `quiet = 45` or `quiet = "1.5h"`
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
refused naming the field (55, 57). An ending left out SHALL be returned
left out (58, C.4).

#### Scenario: Both readers read one elaboration kind
- **WHEN** a declaration holds `writeback` with stages and an elaboration section declaring `inputs`, `conclusion`, `ending = "self-closing"`, `transcripts = false` and `lands`
- **THEN** a read of its kinds gives `writeback`'s stages with the section returned unread, and a read of its elaboration gives every field the section declares, and neither fails on the other's part (S11)

#### Scenario: A malformed elaboration section
- **WHEN** an elaboration section declares `ending = "sometimes"`
- **THEN** a read of the kinds still succeeds, and a read of the elaboration is refused naming `kinds.writeback.elaboration.ending`

#### Scenario: An elaboration section with no ending
- **WHEN** an elaboration section declares its inputs and no `ending`
- **THEN** a read of the elaboration gives its inputs with the ending left out, and is not refused

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

### Requirement: The pack holds the kind format's cases

The conformance pack SHALL hold cases of the kinds shape at `pack/kinds/<case>/case.toml` (C.4; C.7): each a declaration, written in the case, and either the kinds it reads into, each with its version and its stages in order, each stage's name and its deliverable or that it is a conversation, and the sections returned unread; or the refusal, each field named with a phrase its reason holds; citing the clauses it exercises (67). The kinds crate's tests SHALL read every case and fail naming each one whose kinds or refusal differ (65, 87; core.36). The pack SHALL hold a case citing each of 51 to 58, among them flume's own declaration read twice the same (S10), an elaboration kind read with its section returned unread (S11), and a stage with two deliverables and a declaration with no stage, each refused naming the field (S12). A field the case format does not name SHALL fail the case's read, naming the case and the field.

#### Scenario: Every kinds case is answered as it says
- **WHEN** each kinds case of the pack is read by the kinds crate
- **THEN** every declaration reads into the kinds its case expects, or is refused naming the fields its case expects

#### Scenario: A stage with two deliverables (S12)
- **WHEN** a case's declaration gives a stage `deliverables = ["commits", "tasks"]`
- **THEN** the case expects the refusal naming that stage's `deliverables`, and the crate gives it

#### Scenario: A misspelt field of the case
- **WHEN** a kinds case names `kind` in place of `expect`
- **THEN** reading it fails, naming the case and the field
