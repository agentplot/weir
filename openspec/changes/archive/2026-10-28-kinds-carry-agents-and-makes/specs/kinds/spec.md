# Spec Delta

## ADDED Requirements

### Requirement: A kind's agents table reads with the kind
A kind's `agents` table SHALL read with the kind: every key of it but
`stages` as the settings for every stage's agents, and each
`agents.stages.<stage>` table as the settings for that stage's agents,
all as written (52, C.4). A kind with no `agents` table SHALL read with
none. A stage's own `agents` SHALL read beside it as before (53). An
`agents` table, its `stages` table, or a stage's table there, that is
not a table SHALL be refused naming the field (57).

#### Scenario: A kind's agents table for every stage and for one
- **WHEN** a kind declares `[kinds.bump.agents]` with `program`, `model`, `instructions` and `env`, and `[kinds.bump.agents.stages.review]` with `model = "opus"`
- **THEN** the kind reads with those four settings for every stage, `model = "opus"` for `review`, and no section unread

#### Scenario: An inline agents table
- **WHEN** a kind declares `agents = { model = "haiku" }`
- **THEN** it reads as the same table written as a header would

#### Scenario: A stage's own agents beside the kind's
- **WHEN** a kind declares an `agents` table and its stage `code` declares `agents = [{ model = "haiku" }]`
- **THEN** both read, the kind's with the kind and the stage's with the stage

#### Scenario: An agents table of the wrong shape is refused
- **WHEN** a kind declares `agents = ["claude"]`, `agents = 3`, `agents.stages = 3` or `agents.stages.code = "opus"`
- **THEN** the declaration is refused naming that field

### Requirement: A kind's table holding agents and no stages gives settings
A `[kinds.<kind>]` table holding `agents` and none of `version`,
`branch`, `starts_unready`, `lands`, `stages` and `elaboration` SHALL
declare no kind: it SHALL be returned apart from the kinds as settings
given to a kind of that name, its `agents` read as a kind's are and
every other key returned unread, and SHALL never be refused as a kind
with no stage (57, C.4). Whether a stage it names exists, and which
declaration it joins, SHALL be left to the reader (58).

#### Scenario: Settings for a kind the declaration does not declare
- **WHEN** a declaration holds `[kinds.change.agents]` with `model = "sonnet"` and `[kinds.change.agents.stages.verify]` with `model = "opus"`, beside a kind `tidy` it declares
- **THEN** it reads with the one kind `tidy`, and settings for `change`: `model = "sonnet"` for every stage and `model = "opus"` for `verify`

#### Scenario: Settings carry every field as written
- **WHEN** a settings-only table gives `program = "claude"` and `env = { ACCOUNT = "work" }`, with an unknown key `owner` beside `agents`
- **THEN** the settings hold both as written, and `owner` is returned unread beside them

#### Scenario: A table with neither agents nor stages
- **WHEN** a kind's table holds only `owner = "docs"`
- **THEN** the declaration is refused naming that kind's `stages`

## MODIFIED Requirements

### Requirement: A stage reads with its name, prompt and end
A stage SHALL read with its name, its prompt as written, and either its
one deliverable or that it is a conversation (53). It SHALL read its
agents with their settings, the names of what it makes besides its
deliverable (none when left out), whether it waits (false when left
out), and its `quiet`, `on_fail` and `max_send_backs`, each of the last
three returned left out when left out (52, 53, 58, C.4).

#### Scenario: A stage reads with its settings
- **WHEN** a stage declares `name = "review"`, a prompt, `conversation = true`, `wait = true`, `agents = [{ beside = "operator" }]`, `on_fail = "code"` and `max_send_backs = 3`
- **THEN** it reads as a conversation named `review` that waits on its caller's word, with one agent holding `beside = "operator"`, sending a failed place back to `code` at most three times

#### Scenario: Settings left out are returned left out
- **WHEN** a stage declares no `quiet`, `on_fail` or `max_send_backs`
- **THEN** it reads with each of the three left out, and no value stands in for any of them

#### Scenario: A stage reads what it makes
- **WHEN** a stage declares `makes = ["preview", "page"]`, and another stage declares no `makes`
- **THEN** the first reads making `preview` then `page`, with no section unread, and the second making nothing

### Requirement: A declaration is checked whole and refused naming the field
A declaration SHALL be refused whole when any kind has no stage, any
stage has no name or no prompt, any field the format names has the wrong
shape, a kind's `agents` table among them and a stage's `makes` that is
not a list of non-empty names, any deliverable is not of the six, two
stages of one kind share a name, or an `on_fail` names no stage of its
kind (57, C.4). A kind's table that gives settings (57) is no kind with
no stage. The refusal SHALL name every field that fails, each by its
path and with its reason, and SHALL return no kind (81).

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

The conformance pack SHALL hold cases of the kinds shape at `pack/kinds/<case>/case.toml` (C.4; C.7): each a declaration, written in the case, and either the kinds it reads into, each with its version and its stages in order, each stage's name and its deliverable or that it is a conversation, the sections returned unread, and the names of the kinds it gives settings to without declaring them; or the refusal, each field named with a phrase its reason holds; citing the clauses it exercises (67). The kinds crate's tests SHALL read every case and fail naming each one whose kinds, settings or refusal differ (65, 87; core.36). The pack SHALL hold a case citing each of 51 to 58, among them flume's own declaration read twice the same (S10), an elaboration kind read with its section returned unread (S11), a stage with two deliverables and a declaration with no stage, each refused naming the field (S12), and a kind's `agents` table beside settings given to a kind declared elsewhere (52, 57). A field the case format does not name SHALL fail the case's read, naming the case and the field.

#### Scenario: Every kinds case is answered as it says
- **WHEN** each kinds case of the pack is read by the kinds crate
- **THEN** every declaration reads into the kinds and settings its case expects, or is refused naming the fields its case expects

#### Scenario: A stage with two deliverables (S12)
- **WHEN** a case's declaration gives a stage `deliverables = ["commits", "tasks"]`
- **THEN** the case expects the refusal naming that stage's `deliverables`, and the crate gives it

#### Scenario: Settings for a kind declared elsewhere
- **WHEN** a case's declaration gives `change` an `agents` table and no stages, beside `bump` with an `agents` table and stages
- **THEN** the case expects `bump` with no section unread and settings for `change`, and the crate gives them

#### Scenario: A misspelt field of the case
- **WHEN** a kinds case names `kind` in place of `expect`
- **THEN** reading it fails, naming the case and the field
