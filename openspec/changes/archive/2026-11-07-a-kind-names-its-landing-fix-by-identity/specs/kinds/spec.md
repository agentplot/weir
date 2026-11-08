## RENAMED Requirements

- FROM: `### Requirement: A kind that lands with no stages may name its fix kind and its bound`
- TO: `### Requirement: A kind that lands with no stages may name its landing fix and its bound`

- FROM: `### Requirement: A kind's fix is a kind's name and its bound a count`
- TO: `### Requirement: A kind's landing fix is an identity with its version and its bound a count`

## MODIFIED Requirements

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
