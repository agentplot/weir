# Spec Delta

## ADDED Requirements

### Requirement: A stage reads whether it repeats per task group
A stage's `repeat` SHALL read with the stage: `per-task-group`, worked
one group of its change's tasks at a time, the one value the format
gives it, and none when left out (53, C.4). How a reader works a
repetition SHALL be left to the reader (58).

#### Scenario: A stage that repeats per task group
- **WHEN** a stage delivering `tasks` declares `repeat = "per-task-group"`, and the kind's other stages declare no `repeat`
- **THEN** that stage reads repeating per task group, with no section unread, and every other stage reads with no `repeat`

#### Scenario: The spelling reads back
- **WHEN** a stage's `repeat` reads as per task group
- **THEN** it is spelled `per-task-group`, as the declaration wrote it

### Requirement: A repeat the format does not allow is refused
A stage's `repeat` SHALL be refused naming the field when it is any
value but `per-task-group`, or when the stage is a conversation or its
deliverable is not `tasks` (57, C.4). A stage whose own end is refused
SHALL have that end named and its `repeat` judged against no end.

#### Scenario: Another value of repeat
- **WHEN** a stage delivering `tasks` declares `repeat = "per-task"`, `repeat = true` or `repeat = ["per-task-group"]`
- **THEN** the declaration is refused naming that stage's `repeat` field

#### Scenario: Repeat on a stage that does not deliver tasks
- **WHEN** a stage delivering `commits`, or a conversation, declares `repeat = "per-task-group"`
- **THEN** the declaration is refused naming that stage's `repeat` field, with its end in the reason

#### Scenario: Repeat beside an end already refused
- **WHEN** a stage declares `repeat = "per-task-group"` and `deliverables = ["commit"]`
- **THEN** the one refusal names that stage's `deliverables` and not its `repeat`

## MODIFIED Requirements

### Requirement: A stage reads with its name, prompt and end
A stage SHALL read with its name, its prompt as written, and either its
one deliverable or that it is a conversation (53). It SHALL read its
agents with their settings, the names of what it makes besides its
deliverable (none when left out), whether it waits (false when left
out), whether it repeats per task group (none when left out), and its
`quiet`, `on_fail` and `max_send_backs`, each of the last three
returned left out when left out (52, 53, 58, C.4).

#### Scenario: A stage reads with its settings
- **WHEN** a stage declares `name = "review"`, a prompt, `conversation = true`, `wait = true`, `agents = [{ beside = "operator" }]`, `on_fail = "code"` and `max_send_backs = 3`
- **THEN** it reads as a conversation named `review` that waits on its caller's word, with one agent holding `beside = "operator"`, sending a failed place back to `code` at most three times

#### Scenario: Settings left out are returned left out
- **WHEN** a stage declares no `quiet`, `on_fail`, `max_send_backs` or `repeat`
- **THEN** it reads with each of the four left out, and no value stands in for any of them

#### Scenario: A stage reads what it makes
- **WHEN** a stage declares `makes = ["preview", "page"]`, and another stage declares no `makes`
- **THEN** the first reads making `preview` then `page`, with no section unread, and the second making nothing

### Requirement: A declaration is checked whole and refused naming the field
A declaration SHALL be refused whole when any kind has no stage, any
stage has no name or no prompt, any field the format names has the wrong
shape, a kind's `agents` table among them and a stage's `makes` that is
not a list of non-empty names, any deliverable is not of the six, a
stage repeats per task group and does not deliver tasks, two stages of
one kind share a name, or an `on_fail` names no stage of its kind (57,
C.4). A kind's table that gives settings (57) is no kind with no stage.
The refusal SHALL name every field that fails, each by its path and with
its reason, and SHALL return no kind (81).

#### Scenario: A kind with no stage is refused
- **WHEN** a kind declares no stages, or `stages = []`
- **THEN** the declaration is refused naming that kind's `stages` field (S12)

#### Scenario: A stage with no prompt is refused
- **WHEN** the second stage of `bump` has a name and no prompt
- **THEN** the declaration is refused naming `kinds.bump.stages[1].prompt`

#### Scenario: A field of the wrong shape is refused
- **WHEN** a kind declares `version = "two"`, `version = 0` or `lands = "yes"`, or a stage declares `quiet = 45`, `quiet = "1.5h"`, `makes = "preview"`, `makes = ["preview", 3]` or `makes = [""]`
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

### Requirement: The pack holds the kind format's cases

The conformance pack SHALL hold cases of the kinds shape at `pack/kinds/<case>/case.toml` (C.4; C.7): each a declaration, written in the case, and either the kinds it reads into, each with its version and its stages in order, each stage's name, its deliverable or that it is a conversation, and its `repeat` where it repeats, the sections returned unread, and the names of the kinds it gives settings to without declaring them; or the refusal, each field named with a phrase its reason holds; citing the clauses it exercises (67). The kinds crate's tests SHALL read every case and fail naming each one whose kinds, settings or refusal differ (65, 87; core.36). The pack SHALL hold a case citing each of 51 to 58, among them flume's own declaration read twice the same and flume's change@2 read twice the same with its code stage repeating (S10), an elaboration kind read with its section returned unread (S11), a stage with two deliverables and a declaration with no stage, each refused naming the field (S12), a kind's `agents` table beside settings given to a kind declared elsewhere (52, 57), and a `repeat` on a stage of commits refused naming the field (57). A field the case format does not name SHALL fail the case's read, naming the case and the field.

#### Scenario: Every kinds case is answered as it says
- **WHEN** each kinds case of the pack is read by the kinds crate
- **THEN** every declaration reads into the kinds and settings its case expects, or is refused naming the fields its case expects

#### Scenario: A stage with two deliverables (S12)
- **WHEN** a case's declaration gives a stage `deliverables = ["commits", "tasks"]`
- **THEN** the case expects the refusal naming that stage's `deliverables`, and the crate gives it

#### Scenario: Settings for a kind declared elsewhere
- **WHEN** a case's declaration gives `change` an `agents` table and no stages, beside `bump` with an `agents` table and stages
- **THEN** the case expects `bump` with no section unread and settings for `change`, and the crate gives them

#### Scenario: flume's change@2 (S10)
- **WHEN** a case's declaration gives `change` at version 2 with `construct`, `code` and `verify`, its `code` delivering `tasks` with `repeat = "per-task-group"`
- **THEN** the case expects `change@2` with `code` repeating per task group and the other two stages not, and the crate gives it, the same twice

#### Scenario: A misspelt field of the case
- **WHEN** a kinds case names `kind` in place of `expect`
- **THEN** reading it fails, naming the case and the field
