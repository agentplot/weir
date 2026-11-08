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

### Requirement: A stage reads with its name, prompt and end
A stage SHALL read with its name, its prompt as written, and either its
one deliverable or that it is a conversation (53). It SHALL read its
standing instructions' path as written (none when left out, and refused
naming the field when it is not a non-empty string), its agents with
their settings, the names of what it makes besides its deliverable
(none when left out), whether it waits (false when left out), whether it
repeats per task group (none when left out), and its `quiet`, `on_fail`
and `max_send_backs`, each of the last three returned left out when
left out (52, 53, 57, 58, C.4).

#### Scenario: A stage reads with its settings
- **WHEN** a stage declares `name = "review"`, a prompt, `conversation = true`, `wait = true`, `agents = [{ beside = "operator" }]`, `on_fail = "code"` and `max_send_backs = 3`
- **THEN** it reads as a conversation named `review` that waits on its caller's word, with one agent holding `beside = "operator"`, sending a failed place back to `code` at most three times

#### Scenario: Settings left out are returned left out
- **WHEN** a stage declares no `quiet`, `on_fail`, `max_send_backs` or `repeat`
- **THEN** it reads with each of the four left out, and no value stands in for any of them

#### Scenario: A stage reads what it makes
- **WHEN** a stage declares `makes = ["preview", "page"]`, and another stage declares no `makes`
- **THEN** the first reads making `preview` then `page`, with no section unread, and the second making nothing

#### Scenario: A stage reads its standing instructions
- **WHEN** a stage declares `standing = "prompts/bump-code-standing.md"`, and another stage declares no `standing`
- **THEN** the first reads with that path as written, with no section unread, and the second with none

#### Scenario: Standing instructions of the wrong shape
- **WHEN** a stage declares `standing = ""` or `standing = 3`
- **THEN** the declaration is refused naming that stage's `standing` field

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
A declaration SHALL be refused whole when any of these holds:

- a kind that does not land has no stage;
- a kind's table that declares a kind leaves its `stages` out, whether
  or not it lands;
- a stage has no name or no prompt;
- a field the format names has the wrong shape, a kind's `agents` table
  among them, a stage's `makes` that is not a list of non-empty names,
  and a `landing_fix` that is no identity with its version;
- a deliverable is not of the six;
- a stage repeats per task group and does not deliver tasks;
- two stages of one kind share a name;
- an `on_fail` names no stage of its kind;
- a kind with stages names its landing fix, `landing_fix`, or how many
  landing fixes it runs, `max_landing_fixes`, whether or not it lands.

These follow 51, 52 and 57, as amended at blueprints 51b6316 and
8eec6dd, and C.4. A kind's table that gives settings (57) is no kind
with no stage. A `stages = []` beside a `lands` that is refused SHALL be
judged against no `lands`, and not refused a second time. A
`landing_fix` or a `max_landing_fixes` SHALL be judged against the
kind's stages only where they read as one or more stages: where
`stages` is left out, is `[]` on a kind that does not land, or is not an
array of tables, `stages` is named and neither is refused for the kind's
stages. A `landing_fix` or a `max_landing_fixes` of the wrong shape
SHALL be named once, by its shape. The refusal SHALL name every field
that fails, in the order the reader walks the declaration, a kind's
stages before its `landing_fix` and its `landing_fix` before its
`max_landing_fixes`, each by its path and with its reason, and SHALL
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

#### Scenario: A kind with stages that names its fix is refused
- **WHEN** a kind that lands, and another that does not, each with one stage `code` delivering commits, declare `landing_fix = "acme/fix@1"` and `max_landing_fixes = 2`
- **THEN** the declaration is refused naming each kind's `landing_fix` and then its `max_landing_fixes`, each reason saying only a kind that lands with no stages names it

#### Scenario: A fix of the wrong shape beside stages
- **WHEN** a kind with one stage declares `landing_fix = 3`
- **THEN** the one refusal names that kind's `landing_fix` once, with the reason that it must be a kind's identity with its version written as a string

#### Scenario: A fix beside stages already refused
- **WHEN** a kind declares `lands = true` and `landing_fix = "acme/fix@1"` and leaves `stages` out, and another declares `stages = []`, `landing_fix = "acme/fix@1"` and `max_landing_fixes = 2` and does not land
- **THEN** the refusal names each kind's `stages` once, and neither kind's `landing_fix` or `max_landing_fixes`

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
with its deliverable or that it is a conversation, and exit 0. It
SHALL list a kind that lands with no stages with the line
`  lands with no stages` after it (51). Below that it SHALL print the
line `  landing fix <identity>` where the kind names its landing fix,
the identity as written, and `  landing fixes at most <n>` where it
names its bound. Neither line SHALL be printed where its field is left
out (52). When the declaration is refused, it SHALL name each failing
field with its reason on standard error and exit non-zero. It SHALL
leave the file as it was.

#### Scenario: A declaration that reads
- **WHEN** `weir check` is given a file declaring `fix` and `change`
- **THEN** it prints `fix@1` and `change@1` with their stages and deliverables, and exits 0

#### Scenario: A kind that lands with no stages is listed
- **WHEN** `weir check` is given a file declaring `train` with `lands = true` and `stages = []`
- **THEN** it prints `train@1` and then `  lands with no stages`, and exits 0

#### Scenario: A kind that names its fix is listed
- **WHEN** `weir check` is given a file declaring `train` with `lands = true`, `stages = []`, `landing_fix = "acme/fix@1"` and `max_landing_fixes = 2`
- **THEN** it prints `train@1`, `  lands with no stages`, `  landing fix acme/fix@1` and `  landing fixes at most 2`, in that order, and exits 0

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
it lands with none. Each kind gives its landing fix and how many it
runs where it names them, and neither where it does not. Each stage
gives its name, its deliverable or that it is a conversation, and its
`repeat` where it repeats. Beside the kinds go the sections returned
unread and the names of the kinds it gives settings to without declaring
them. The second is the refusal, each field named with a phrase its
reason holds. Each case cites the clauses it exercises (67). The kinds
crate's tests SHALL read every case and fail naming each one whose
kinds, settings, identity, hash or refusal differ, a kind's
`landing_fix` and `max_landing_fixes` among them (65, 87; core.36).

The pack SHALL hold a case citing each of 51 to 58, among them:

- flume's own declaration read twice the same, and flume's change@2
  read twice the same with its code stage repeating (S10);
- an elaboration kind read with its section returned unread (S11);
- a stage with two deliverables, and a kind that does not land
  declaring no stage, each refused naming the field (S12);
- a kind that lands declaring `stages = []`, read with no stages, and a
  kind that lands with its `stages` left out, refused naming the field
  (51, 57 as amended at blueprints 51b6316);
- a kind that lands with no stages naming its landing fix and its
  bound, read with both, as a declaration and as a package; a kind with
  stages naming both, refused naming each; and a landing fix that is no
  identity, refused naming it (52, 57 and 89 as amended at blueprints
  8eec6dd);
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

#### Scenario: A kind that names its fix, and one with stages that may not
- **WHEN** one case's declaration gives `train` `lands = true`, `stages = []`, `landing_fix = "acme/fix@1"` and `max_landing_fixes = 2`, a second gives `bump` one stage, `lands = true`, `landing_fix = "acme/fix@1"` and `max_landing_fixes = 2`, and a third holds a package whose one file is a `kind.toml` declaring `train` as the first does, held at `acme/train@1`
- **THEN** the first expects `train` at version 1 with no stages, `landing_fix` `acme/fix@1` and `max_landing_fixes` 2; the second the refusal naming `kinds.bump.landing_fix` and then `kinds.bump.max_landing_fixes`; the third the identity `acme/train@1`, the hash of that one file and `train` with `landing_fix` `acme/fix@1` and `max_landing_fixes` 2; and the crate gives each

#### Scenario: A landing fix that is no identity is a case of its own
- **WHEN** a case's declaration gives `train` `lands = true`, `stages = []` and `landing_fix = "fix"`
- **THEN** the case expects the refusal naming `kinds.train.landing_fix`, its reason holding `is no identity`, and the crate gives it

#### Scenario: A case whose kind differs in its fix is named
- **WHEN** a case expects a kind with `landing_fix = "acme/fix@1"` and the kind read names `acme/other@1`, or expects a kind with no `max_landing_fixes` and the kind read names 2
- **THEN** the crate's tests give a difference naming the case, the kind's `landing_fix` or `max_landing_fixes` read and the one the case expects

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
`branch`, `starts_unready`, `lands`, `stages`, `landing_fix`,
`max_landing_fixes` and `elaboration` SHALL declare no kind: it SHALL be
returned apart from the kinds as settings given to a kind of that name,
its `agents` read as a kind's are and every other key returned unread,
and SHALL never be refused as a kind with no stage (57, C.4). A table
that holds `stages`, `stages = []` among them, or `landing_fix` or
`max_landing_fixes`, SHALL be read as a kind and never as settings (57
as amended at blueprints 51b6316 and 8eec6dd). Whether a stage it names
exists, and which declaration it joins, SHALL be left to the reader
(58).

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

#### Scenario: Agents beside a fix with no stages
- **WHEN** one kind's table holds `[kinds.train.agents]` with `model = "opus"` and `landing_fix = "acme/fix@1"`, and another's holds `[kinds.yard.agents]` with `model = "opus"` and `max_landing_fixes = 2`, neither with `stages`
- **THEN** the declaration is refused naming `kinds.train.stages` and `kinds.yard.stages`, each with the reason that a kind declares its stages, and gives no settings for either

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

### Requirement: A kind's key is a name
`read-kinds` SHALL refuse a kind's table, or a table of settings, under
a key that is not one or more ASCII letters, digits, `_` and `-`,
naming the field by its key as TOML quotes it, and SHALL return no kind,
so every kind it reads has an identity (57, 88; C.4 as amended at
blueprints 361a599; record 0004).

#### Scenario: A kind under a key that is not a name
- **WHEN** a declaration holds `[[kinds."a/b".stages]]` with a name, a prompt and `deliverables = ["commits"]`
- **THEN** it is refused naming `kinds."a/b"`, the reason saying a kind's name is one or more ASCII letters, digits, `_` and `-`

#### Scenario: Settings under a key that is not a name
- **WHEN** a declaration holds `[kinds."review@2".agents]` with `model = "opus"`, beside a kind `bump` it declares
- **THEN** it is refused naming `kinds."review@2"`, and returns no kind

#### Scenario: Names of every allowed character
- **WHEN** a declaration declares `code_review`, `review-2` and `R2`
- **THEN** it reads with all three kinds

### Requirement: The pack holds package and lock cases
A package's case SHALL hold the package's files, and its links where it
has any, written in the case, the identity it is held at, and a lock to
check it against where it names one; and either its kind with its
identity and content hash, or the refusal. A lock's case SHALL hold the
lock and either its identities with their hashes, or the refusal (C.7
as amended at blueprints 361a599). The pack SHALL hold a case citing
each of 88 to 93.

#### Scenario: A package laid out in two folders (S17)
- **WHEN** the kinds crate's tests read a package's case
- **THEN** they lay its files and links out in two folders at different paths, read each at the case's identity, and fail naming the case where the two answers differ or either differs from the case's

#### Scenario: The cases the statement gives
- **WHEN** the pack's kinds cases are listed
- **THEN** they hold flume's change@1 and change@2 each read as a package of its own (S10); a prompt named `../prompts/code.md` refused, the same package with the prompt inside it read, and its prompt a link to another file inside it refused naming the link (S17); a package changed by a byte refused against a lock naming the kind and both hashes (S18); a package held at another version refused naming both identities; a lock read with two versions of a kind; a lock recording one identity twice refused; and a declaration naming a kind by a key that is not a name refused naming it

#### Scenario: The cases cite 88 to 93
- **WHEN** the clauses the kinds cases cite are gathered
- **THEN** each of 88, 89, 90, 91, 92 and 93 is among them

### Requirement: A kind that lands with no stages may name its landing fix and its bound
A kind SHALL read `landing_fix`, the kind a place runs when the merge
queue sends its branch back, and `max_landing_fixes`, how many landing
fixes it runs before its place waits on its caller, and SHALL return
neither among its sections (52 and C.4 as amended at blueprints
8eec6dd; record 0005). Each left out SHALL be returned left out, with
no default, and either SHALL read without the other (58). Which kind
`landing_fix` names, and what that kind holds, SHALL be the reader's to
say (58, 89; core.32). `fix` and `max_fixes` are no fields the format
names, and SHALL be returned among a kind's sections as any such key is
(56).

#### Scenario: A kind that lands with no stages names its fix
- **WHEN** a declaration holds `[kinds.train]` with `branch = "train"`, `lands = true`, `stages = []`, `landing_fix = "acme/fix@1"` and `max_landing_fixes = 2`
- **THEN** the read holds one kind, `train`, at version 1, landing, with no stages, its `landing_fix` the identity `acme/fix@1` spelling back as written, with source `acme`, name `fix` and version 1, its `max_landing_fixes` 2, and no section unread

#### Scenario: A fix and its bound left out are returned left out
- **WHEN** a kind declares `lands = true` and `stages = []` and neither `landing_fix` nor `max_landing_fixes`
- **THEN** it reads with both left out, and no value stands in for either

#### Scenario: A bound without a fix
- **WHEN** a kind that lands with no stages declares `max_landing_fixes = 0` and no `landing_fix`
- **THEN** it reads with `max_landing_fixes` 0 and `landing_fix` left out

#### Scenario: The fields' old spellings are sections
- **WHEN** a kind that lands with no stages declares `fix = "fix"` and `max_fixes = 2`
- **THEN** it reads with `landing_fix` and `max_landing_fixes` left out, and `fix` and `max_fixes` returned unread among its sections

### Requirement: A kind's landing fix is an identity with its version and its bound a count
A kind's `landing_fix` SHALL be an identity with its version,
`<source>/<name>@<version>`, each part checked as an identity's is, and
SHALL spell back as written (C.4 as amended at blueprints 8eec6dd;
core.42; record 0005). A kind's `max_landing_fixes` SHALL be an integer
of at least 0. A `landing_fix` that is not a string, or a string that
is no identity with its version, a bare name among them, or a
`max_landing_fixes` that is not such an integer, SHALL be refused naming
the field (57).

#### Scenario: A fix in every form of a kind's name
- **WHEN** four kinds that land with no stages declare `landing_fix = "fix"`, `landing_fix = "fix@2"`, `landing_fix = "acme/fix"` and `landing_fix = "acme/fix@1"`
- **THEN** the declaration is refused naming the `landing_fix` of the first three and not of the fourth, each reason saying it is no identity, the first two that it names no source and the third that it names no version

#### Scenario: A fix or a bound of the wrong shape
- **WHEN** a kind that lands with no stages declares `landing_fix = 3`, `landing_fix = ""`, `landing_fix = "a b"`, `landing_fix = "../fix"` or `landing_fix = "acme/fix@0"`, or `max_landing_fixes = -1`, `max_landing_fixes = "2"` or `max_landing_fixes = 1.5`
- **THEN** the declaration is refused naming that kind's `landing_fix` or `max_landing_fixes` field, a `landing_fix` string that is no identity with a reason saying it is no identity
