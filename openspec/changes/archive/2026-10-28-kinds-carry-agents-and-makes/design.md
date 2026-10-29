# Design

## Context

`read_kinds` walks `[kinds]`, reading each table as a kind; every key a
kind or a stage holds that the format does not name is returned in its
`sections`. The statement at blueprints 4d1f88b names a kind's `agents`
table (52, C.4) and reads a kind's table holding `agents` and nothing
the format names besides as settings given to a kind declared
elsewhere, never as a kind with no stage (57, C.4). The runner of
places reads a stage's `makes` (its statement's C.2 and 129, which name
`makes` among the kind format's own names).

Rules this design relies on: core.30 (one reader, sections returned
unread, nothing rewritten), core.31 (checked whole, refused naming the
field), core.32 (no resolution of names or layering of settings in this
crate), core.2 (no tool named in the crate or its fixtures), core.39
(the kinds project's checks). The area is the kinds crate; there are no
area pages.

## Decisions

### A kind's agents table is `KindAgents`, settings as written

`Kind::agents: KindAgents { every_stage: Table, stages: BTreeMap<String,
Table> }`: `every_stage` is every key of the `agents` table but
`stages`, and `stages` each `agents.stages.<stage>` table, all as
written, as a stage's own `agents` are today (53). The format checks
only that the table, its `stages` and each stage's settings are tables
(57); `program`, `model`, `instructions`, `env` and any other setting
are the runner's (C.4: "settings as written"). Left out, it is empty.
`agents` joins the kind fields the format names, so it no longer
appears in `Kind::sections`.

### A settings-only table is `KindSettings`, in `Declaration::settings`

A kind's table holding `agents` and none of `version`, `branch`,
`starts_unready`, `lands`, `stages` and `elaboration` gives settings
(57, C.4). It is returned in `Declaration::settings`, by name, as
`KindSettings { name, agents, sections }`, apart from `kinds`, so a
reader tells a declaration from settings by where it lies. An
`elaboration` section counts as declaring a kind, being the format's own
section (55): a table holding it and `agents` and no stages is refused
as a kind with no stage. A table holding neither `agents` nor stages is
refused as before.

Whether a stage named under `agents.stages` exists, for a settings-only
table or a declared kind, is not checked here: C.4 leaves it to the
reader for the settings-only table (58), and the runner refuses
settings for a stage the kind lacks at its launch, so the crate holds
both tables to one rule.

### `env` is read as written

The statement gives the refusal of an environment in a repository's
file to the runner, at its launch, since the format cannot know which
declaration a file is (58). The crate returns `env` with the other
settings.

### `makes` is a list of names

`Stage::makes: Vec<String>`, in the order written, empty when left out;
anything but an array of strings, or a name that is the empty string,
is refused naming the field (57). weir's statement does not yet name
`makes` in 53 or C.4; the runner's statement names it as the kind
format's own (its C.2), and this crate reads it so. 53 and C.4 are the
statement's to amend.

## Risks / Trade-offs

- [A struct literal of `Kind`, `Stage` or `Declaration` outside the
  crate fails to compile on the new fields] → readers only read these;
  no crate of the family builds them, and every existing field reads as
  before.
