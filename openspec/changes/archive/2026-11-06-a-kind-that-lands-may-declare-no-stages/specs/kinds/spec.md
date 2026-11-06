# Spec Delta

## MODIFIED Requirements

### Requirement: A declaration reads into its kinds
A declaration SHALL read into every kind under its `kinds` table, each
with its version, its stages in the order written, its branch prefix,
whether its first stage starts unready and whether it lands (51, 52,
C.4). A version left out SHALL read as 1, a branch left out as the
kind's name, and `starts_unready` and `lands` left out as false. A kind
that lands and declares `stages = []` SHALL read as a kind with no
stages. Its version, branch, `starts_unready`, `agents` table and
sections SHALL read as any kind's do, and none SHALL be refused because
it has no stages (51, 52 and 57 as amended at blueprints 51b6316).

#### Scenario: A kind reads with every field
- **WHEN** a declaration holds `kinds.bump` with `version = 2`, `branch = "bump"`, `starts_unready = false`, `lands = true` and two stages, `code` then `review`
- **THEN** the read holds one kind, `bump`, at version 2, with branch prefix `bump`, not starting unready, landing, and its stages `code` then `review` in that order

#### Scenario: Fields left out read as the format says
- **WHEN** a kind declares only its stages
- **THEN** it reads at version 1, with its own name as its branch prefix, not starting unready and not landing

#### Scenario: A declaration with no kinds
- **WHEN** a declaration holds no `kinds` table
- **THEN** it reads with no kinds and is not refused

#### Scenario: A kind that lands with no stages
- **WHEN** a declaration holds `[kinds.train]` with `branch = "train"`, `lands = true` and `stages = []`
- **THEN** the read holds one kind, `train`, at version 1, with branch prefix `train`, landing, with no stages and no section unread

#### Scenario: A kind with no stages holds what any kind holds
- **WHEN** a kind declares `lands = true`, `stages = []`, `version = 2`, `starts_unready = true`, an `agents` table with `model = "opus"`, and `owner = "release"`
- **THEN** it reads at version 2, starting unready, landing, with no stages, the `agents` table as written, and `owner` returned unread

### Requirement: A declaration is checked whole and refused naming the field
A declaration SHALL be refused whole when any of these holds:

- a kind that does not land has no stage;
- a kind's table that declares a kind leaves its `stages` out, whether
  or not it lands;
- a stage has no name or no prompt;
- a field the format names has the wrong shape, a kind's `agents` table
  among them and a stage's `makes` that is not a list of non-empty
  names;
- a deliverable is not of the six;
- a stage repeats per task group and does not deliver tasks;
- two stages of one kind share a name;
- an `on_fail` names no stage of its kind.

These follow 51, 52 and 57, as amended at blueprints 51b6316, and C.4.
A kind's table that gives settings (57) is no kind with no stage. A
`stages = []` beside a `lands` that is refused SHALL be judged against
no `lands`, and not refused a second time. The refusal SHALL name every
field that fails, each by its path and with its reason, and SHALL
return no kind (81).

#### Scenario: A kind with no stage is refused
- **WHEN** a kind that does not land, one with `lands` left out and another with `lands = false`, declares `stages = []`
- **THEN** the declaration is refused naming each kind's `stages` field, the reason saying a kind that does not land has at least one stage (S12)

#### Scenario: A kind's stages left out are refused, whether it lands or not
- **WHEN** a kind declares `version = 1` and no stages, and another declares `lands = true` and no stages
- **THEN** the declaration is refused naming each kind's `stages` field, the reason saying a kind declares its stages, one or more, or none as `stages = []` where it lands

#### Scenario: Empty stages beside a lands of the wrong shape
- **WHEN** a kind declares `lands = "yes"` and `stages = []`
- **THEN** the one refusal names that kind's `lands` and not its `stages`

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
- **WHEN** one kind that does not land has no stage and another kind's stage has no prompt
- **THEN** the one refusal names both fields, and no kind is returned

#### Scenario: Text that is not TOML is refused
- **WHEN** a declaration's text does not parse as TOML
- **THEN** it is refused saying where in the text it fails and why

### Requirement: A kind's table holding agents and no stages gives settings
A `[kinds.<kind>]` table holding `agents` and none of `version`,
`branch`, `starts_unready`, `lands`, `stages` and `elaboration` SHALL
declare no kind: it SHALL be returned apart from the kinds as settings
given to a kind of that name, its `agents` read as a kind's are and
every other key returned unread, and SHALL never be refused as a kind
with no stage (57, C.4). A table that holds `stages`, `stages = []`
among them, SHALL be read as a kind and never as settings (57 as
amended at blueprints 51b6316). Whether a stage it names exists, and
which declaration it joins, SHALL be left to the reader (58).

#### Scenario: Settings for a kind the declaration does not declare
- **WHEN** a declaration holds `[kinds.change.agents]` with `model = "sonnet"` and `[kinds.change.agents.stages.verify]` with `model = "opus"`, beside a kind `tidy` it declares
- **THEN** it reads with the one kind `tidy`, and settings for `change`: `model = "sonnet"` for every stage and `model = "opus"` for `verify`

#### Scenario: Settings carry every field as written
- **WHEN** a settings-only table gives `program = "claude"` and `env = { ACCOUNT = "work" }`, with an unknown key `owner` beside `agents`
- **THEN** the settings hold both as written, and `owner` is returned unread beside them

#### Scenario: A table with neither agents nor stages
- **WHEN** a kind's table holds only `owner = "docs"`
- **THEN** the declaration is refused naming that kind's `stages`

#### Scenario: Agents beside stages declared as none
- **WHEN** a kind's table holds `[kinds.train.agents]` with `model = "opus"`, `lands = true` and `stages = []`
- **THEN** it reads as the kind `train` with no stages and that `agents` table, and gives no settings for `train`

### Requirement: weir check reads a declaration
`weir check <declaration>` SHALL read the file it names as a
declaration, the elaboration sections included. When the declaration
reads, it SHALL list each kind with its version and its stages, each
with its deliverable or that it is a conversation, and exit 0. It
SHALL list a kind that lands with no stages with the line
`  lands with no stages` after it (51). When the declaration is refused,
it SHALL name each failing field with its reason on standard error and
exit non-zero. It SHALL leave the file as it was.

#### Scenario: A declaration that reads
- **WHEN** `weir check` is given a file declaring `fix` and `change`
- **THEN** it prints `fix@1` and `change@1` with their stages and deliverables, and exits 0

#### Scenario: A kind that lands with no stages is listed
- **WHEN** `weir check` is given a file declaring `train` with `lands = true` and `stages = []`
- **THEN** it prints `train@1` and then `  lands with no stages`, and exits 0

#### Scenario: A declaration that is refused
- **WHEN** `weir check` is given a file whose stage declares two deliverables
- **THEN** it prints that stage's `deliverables` field with its reason on standard error, prints no kind, and exits non-zero

#### Scenario: Checking changes nothing
- **WHEN** `weir check` reads a file
- **THEN** the file's bytes after the check are the bytes before it

### Requirement: The pack holds the kind format's cases

The conformance pack SHALL hold cases of the kinds shape at
`pack/kinds/<case>/case.toml` (C.4; C.7), each of a declaration, a
package or a lock. A declaration's case holds the declaration, written
in the case. It expects one of two answers. The first is the kinds it
reads into, each with its version and its stages in order, none where
it lands with none. Each stage gives its name, its deliverable or that
it is a conversation, and its `repeat` where it repeats. Beside the
kinds go the sections returned unread and the names of the kinds it
gives settings to without declaring them. The second is the refusal,
each field named with a phrase its reason holds. Each case cites the
clauses it exercises (67). The kinds crate's tests SHALL read every
case and fail naming each one whose kinds, settings, identity, hash or
refusal differ (65, 87; core.36).

The pack SHALL hold a case citing each of 51 to 58, among them:

- flume's own declaration read twice the same, and flume's change@2
  read twice the same with its code stage repeating (S10);
- an elaboration kind read with its section returned unread (S11);
- a stage with two deliverables, and a kind that does not land
  declaring no stage, each refused naming the field (S12);
- a kind that lands declaring `stages = []`, read with no stages, and a
  kind that lands with its `stages` left out, refused naming the field
  (51, 57 as amended at blueprints 51b6316);
- a kind's `agents` table beside settings given to a kind declared
  elsewhere (52, 57);
- a `repeat` on a stage of commits refused naming the field (57).

A field the case format does not name SHALL fail the case's read,
naming the case and the field.

#### Scenario: Every kinds case is answered as it says
- **WHEN** each kinds case of the pack is read by the kinds crate
- **THEN** every declaration reads into the kinds and settings its case expects, every package into its kind, identity and hash, and every lock into its entries, or each is refused naming the fields its case expects

#### Scenario: A stage with two deliverables (S12)
- **WHEN** a case's declaration gives a stage `deliverables = ["commits", "tasks"]`
- **THEN** the case expects the refusal naming that stage's `deliverables`, and the crate gives it

#### Scenario: A kind that lands with no stages and its two neighbours
- **WHEN** one case's declaration gives `train` `lands = true` and `stages = []`, a second gives a kind `stages = []` with `lands` left out, and a third gives a kind `lands = true` with `stages` left out
- **THEN** the first expects `train` at version 1 with no stages, the second and third each the refusal naming that kind's `stages`, and the crate gives each

#### Scenario: Settings for a kind declared elsewhere
- **WHEN** a case's declaration gives `change` an `agents` table and no stages, beside `bump` with an `agents` table and stages
- **THEN** the case expects `bump` with no section unread and settings for `change`, and the crate gives them

#### Scenario: flume's change@2 (S10)
- **WHEN** a case's declaration gives `change` at version 2 with `construct`, `code` and `verify`, its `code` delivering `tasks` with `repeat = "per-task-group"`
- **THEN** the case expects `change@2` with `code` repeating per task group and the other two stages not, and the crate gives it, the same twice

#### Scenario: A misspelt field of the case
- **WHEN** a kinds case names `kind` in place of `expect`
- **THEN** reading it fails, naming the case and the field

#### Scenario: A case of more than one form
- **WHEN** a kinds case holds both a `declaration` and a `lock`
- **THEN** reading it fails, naming the case and saying it holds one of a declaration, a package and a lock
