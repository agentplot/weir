# Design

## Context

`weir-kinds` at main, 5562af5, reads a kind's table in `read_kind`
(`src/kinds.rs`). It reads the fields `KIND_FIELDS` names, `version`,
`branch`, `starts_unready`, `lands`, `agents` and `stages`, and returns
every other key among `Kind::sections`, unread (56). A table holding
`agents` and none of `DECLARING_FIELDS` gives settings (core.41). A
stage names where a failed submission sends its place back, by
`on_fail`, and how many times, by `max_send_backs`, an integer of at
least 0 read as `Option<u64>`. A kind that lands with `stages = []` has
no stage to carry either. `NamedKind` (`src/identity.rs`) reads a kind's
name in its four forms, `<name>`, `<name>@<version>`, `<source>/<name>`
and `<source>/<name>@<version>`, checks each part as an identity's
parts are checked, and spells back as it was written (C.4; core.42).

This design reads the statement at blueprints 31f9f1c, where flume
record 0056's amendment is settled. It uses 52, 57 and 89 as amended
there, with C.4's `[kinds.train]` example and its paragraph on
defaults, and 56 and 58 as they stand:

- A kind that lands with no stages may name the kind its fixes run as,
  by a name its reader resolves, and how many fixes it runs before its
  place waits on its caller (52).
- A kind with stages that names either is refused naming the field (57).
- A package names no kind but its own and the one its kind names for
  its fixes, a name its reader resolves and no part of the package (89).
- C.4 spells the two `fix = "fix"` and `max_fixes = 2`, and gives
  neither a default. Each field left out is returned left out, and
  `fix` is returned as written.

flume's design for its change
`a-conflicting-base-place-gets-its-fix-launched`, decision 1, names the
fix kind "by a kind's name or a whole identity" (flume 23). It maps weir's two fields into its own `Kind`,
`fix: Option<String>` and `fix_limit: Option<u32>`, in its follow-up unit
a-base-places-fix-kind-is-read-from-weirs-format. Its decision 2
resolves the name at the base place's launch, and refuses the launch
where the name resolves to no kind, to one that does not land, or to one
with no stages. All of that is flume's, by its own rule (58).

Rules this design relies on, all in `docs/architecture/core.md`, the one
page whose area the change touches:

- core.2: no tool named. The fixtures name the kinds `train` and `fix`
  and the source `acme`, as C.4 does.
- core.30: one reader of kinds.
- core.31: a declaration checked whole. It is amended on this branch
  from 52, 57 and C.4 at 31f9f1c.
- core.32: the reader resolves the fix kind. Its details are amended on
  this branch.
- core.35: each case cites its clauses.
- core.37: a clause changed in the statement first.
- core.41: a settings table. Its details are amended on this branch.
- core.42: a kind's name.
- core.43 and core.44: a package and its hash. core.43's details are
  amended on this branch.

The design departs from none.

## Goals / Non-Goals

**Goals:**

- `fix` and `max_fixes` read on a kind that lands with no stages,
  returned as written, left out when left out, and refused naming the
  field where their shape is wrong or the kind has stages.
- Every reader in weir that says what a kind holds says it too: the
  crate's docs, `read-package`, `weir check` and the pack's kinds cases.

**Non-Goals:**

- What a fix is, when one is launched, and what the bound counts. Those
  are flume's (203; core.4, core.32).
- Resolving the name `fix` gives, or checking that the kind it names is
  declared, lands or has stages. That is the reader's (58, 89; core.32).
- The seventh deliverable, `plan`, and statechart kinds (54, 94, 95),
  each a change of its own.
- A release of weir. weir publishes none, and a reader pins a revision
  of its main.

## Decisions

### 1. The two fields are the kind's own, spelled as C.4 spells them

`Kind` gains `fix: Option<NamedKind>` and `max_fixes: Option<u64>`, each
`None` when left out. `KIND_FIELDS` names both, so neither is returned
among `Kind::sections`. `read_kind` reads them after the kind's stages,
since each is judged against them. They sit on the kind's table, beside
`lands`, as C.4's `[kinds.train]` writes them.

The way not taken is a table of their own, such as `[kinds.train.fix]`
holding a kind and a bound. C.4 spells two fields on the kind, and a
table would be a spelling the statement does not give (core.37).

### 2. `fix` is a kind's name, checked and spelled back as written

`fix` reads through `NamedKind`'s `FromStr`, in any of its four forms:
`fix`, `fix@2`, `acme/fix` or `acme/fix@1`. 52 has the fix kind named
"by a name its reader resolves", and flume 23 by "its name or its
identity". `NamedKind` spells back as it was written, and a version has
one spelling, so the name returns as written, as C.4 says.

A `fix` that is not a string is refused naming `kinds.<kind>.fix` with
"must be a kind's name written as a string, such as \"fix\" or
\"acme/fix@1\"". A string that names no kind is refused naming the
same field, the reason `NameRefusal`'s words, such as "`a b` names no
kind: its name `a b` holds ' ', …". That covers `""`, `"a b"`,
`"../fix"` and `"fix@0"`. Checking the name's shape is not resolving it
(core.32). It is 57's "a field of the wrong shape", and it holds 89's "no
part of the package" by construction, since no name is a path.

The way not taken is `Option<String>` with no check. A reader would then
meet `"../fix"` or `"a b"` only when it resolved the name, with no field
named. Each reader would also parse the name again in code of its own,
which core.30 rules out. A whole `Identity` alone is not taken either,
since 52 leaves the source and the version to the reader.

### 3. `max_fixes` is a whole number of at least 0, as `max_send_backs` is

`max_fixes` reads as `Option<u64>`. Any value but an integer of at least
0 is refused naming `kinds.<kind>.max_fixes` with "must be an integer of
at least 0". That is the reason and the rule of `max_send_backs`, the
bound a kind with stages gives (52). 0 reads: such a kind runs no fix,
and its place waits on its caller at the first conflict. What the bound
counts is flume's (58).

The way not taken is a floor of 1. 52 says "how many fixes it runs",
and 0 is a count, as `max_send_backs = 0` is.

### 4. Either field on a kind with stages is refused naming it

Where the kind's `stages` reads as one or more stages, `fix` and
`max_fixes` are each refused naming the field. This holds whether or not
the kind lands, and whatever the stages' own problems:

| field | reason |
|---|---|
| `fix` | "is on a kind with stages, and only a kind that lands with no stages names the kind its fixes run as; a kind with stages names its fix round by a stage's `on_fail`" |
| `max_fixes` | "is on a kind with stages, and only a kind that lands with no stages names how many fixes it runs; a kind with stages bounds its send-backs by a stage's `max_send_backs`" |

A field whose shape is wrong is named once, by its shape's reason, as a
`repeat` of the wrong value is named before its stage's end is judged.
In `read_stages`' answer, "one or more stages" is a non-empty `Vec`.
Where `stages` is left out, is `[]` on a kind that does not land, or is
not an array of tables, `stages` is named and the answer is empty. Then
neither field is judged against stages and neither is refused a second
time. A kind whose `lands` is refused and whose `stages = []` has no
stages, so a `fix` on it is not refused for stages. The one refusal
names its `lands`, as at 5562af5.

The problems come in the order the reader walks the kind: the stages',
then `fix`'s, then `max_fixes`'s (81).

The way not taken is refusing `fix` on every kind that is not a kind
that lands with no stages, `stages` left out among them. A kind whose
`stages` are already refused would then be named twice for one fault,
and decision 4 of the change that let a kind land with no stages
already judges `stages = []` against a refused `lands` this way.

### 5. Neither field needs the other

A kind may name `fix` without `max_fixes`, or `max_fixes` without `fix`.
57 lists no refusal of either alone. C.4 gives neither a default, and
what one means without the other is the reader's: flume's design gives a
bound left out flume's default for a stage's (58). Adding a refusal
would depart from 57 (core.31, core.37).

### 6. A table holding `agents` and either field is a kind

`DECLARING_FIELDS` gains `fix` and `max_fixes`. A `[kinds.<kind>]`
table holding `agents` and either is no settings table, since it holds
a field the format names besides `agents` (57, C.4; core.41). It reads
as a kind, and with its `stages` left out it is refused naming them, by
the reason at 5562af5 that a kind declares its stages. At 5562af5 such
a table gave settings, with `fix` returned unread among them.

The way not taken is keeping it as settings with `fix` unread. A
declaration giving settings to a kind declared elsewhere would then
carry a fix kind no reader looks at, and a kind that meant to land with
no stages and forgot `stages = []` would read as settings without a
word.

### 7. A package names its fix kind by name

`read-package` needs no new check. A kind that lands with no stages
names no part, and `fix` is no path, so the parts the package checks are
unchanged (89; core.43). The package reads with its `fix` as written,
whether or not it holds the kind named. Its content hash is C.4's over
its files, so it covers the name as `kind.toml`'s bytes and no file of
the kind named (90; core.44). The refusal of a `kind.toml` that gives
settings to another kind stays: settings are a table under that kind's
name, which 89 still rules out.

### 8. `weir check` lists the fix kind and the bound

Under `  lands with no stages`, `weir check` lists `  fixes run as
<fix>` where the kind names its fix, the name spelled as written, and
`  fixes at most <n>` where it names its bound. Neither line is printed
where its field is left out. The way not taken is `  fix: fix`, which
reads as a stage named `fix` delivering `fix`.

### 9. The pack's cases

The case format's expected kind, `ExpectKind` in
`crates/weir-kinds/tests/pack.rs`, gains `fix`, the name as spelled, and
`max_fixes`. Each is left out where the kind names none, and
`kind_differences` holds both against the kind read, naming the case.
Every case at 5562af5 expects neither, and no kind in them names either,
so each still answers as it says. Three cases join:

- `a-kind-that-lands-with-no-stages-names-its-fix` declares C.4's
  `[kinds.train]` with `branch`, `lands = true`, `stages = []`,
  `fix = "fix"` and `max_fixes = 2`. It expects `train` at version 1
  with no stages, `fix` `fix` and `max_fixes` 2 (52, 57).
- `a-kind-with-stages-that-names-its-fix` declares `bump`, landing,
  with one stage `code` delivering commits, `fix = "fix"` and
  `max_fixes = 2`. It expects the refusal naming `kinds.bump.fix` and
  then `kinds.bump.max_fixes`, each reason holding `only a kind that
  lands with no stages names` (57).
- `a-package-of-a-kind-that-names-its-fix` holds `kind.toml` alone,
  declaring `train` as the first case does, held at `acme/train@1`. It
  expects that identity, the hash of that one file and `train` with
  `fix` and `max_fixes` (52, 89, 92).

A reader that leaves `fix` among a kind's sections fails the first case
and the third. A reader that takes `fix` on any kind fails the second.

### 10. The clause index stays at a11f6d5

`pack/clauses.toml` moves when a clause is added, removed or
renumbered. 31f9f1c amends 52, 57, 89 and C.4 and adds, removes or
renumbers no clause of weir's. Moving the index there would take in 94
to 99, whose cases are the statechart change's.

## Risks / Trade-offs

- A declaration written before this change, with `fix` or `max_fixes`
  as a section of its own on a kind with stages, now fails to read. 56
  let that declaration name either as a section the format did not
  know, and 57 as amended refuses it now. No case or fixture of weir's
  holds one, and no declaration in flume's repository at its main names
  either.
- `Kind` gains two public fields, so a caller that builds a `Kind` by a
  struct literal no longer compiles until it gives both. weir builds
  none outside `read_kind`. flume maps weir's `Kind` into its own and
  builds none of weir's, and it moves with its pin.

## Migration Plan

flume moves its `weir-kinds` pin in `Cargo.toml` to the revision of
weir's main that lands this change, in its follow-up unit
a-base-places-fix-kind-is-read-from-weirs-format. weir publishes no
release and tags none, so the pin is that revision. Nothing else moves.
