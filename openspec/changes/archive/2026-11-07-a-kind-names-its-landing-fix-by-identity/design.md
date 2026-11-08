# Design

## Context

The bolt `weir-kinds-name-their-fix` holds the archived change
`a-kind-with-no-stages-names-its-fix`, built from the statement at
blueprints 31f9f1c. In `weir-kinds` at ab6f141, `read_kind`
(`src/kinds.rs`) reads `fix` into `Kind::fix: Option<NamedKind>` and
`max_fixes` into `Kind::max_fixes: Option<u64>`. It refuses either on a
kind whose stages read as one or more. `KIND_FIELDS` and
`DECLARING_FIELDS` name both. `fix` takes a kind's name in any of its
four forms. `Identity` (`src/identity.rs`) reads
`<source>/<name>@<version>` whole. It refuses text that leaves out the
source or the version with "is no identity: it names no source, and an
identity is `<source>/<name>@<version>`", or "names no version", and
refuses a part that fails as `NamedKind` does.

Record 0005, accepted by the user on 2026-11-07, renames the fields and
has the landing fix named by identity with its version. The statement
at blueprints 8eec6dd says so in 52, 57, 89, 94 and C.4:

- A kind that lands with no stages may name its landing fix, the kind a
  place runs when the merge queue sends its branch back, by that kind's
  identity with its version. It may also name how many landing fixes it
  runs before its place waits on its caller (52).
- A kind with stages that names either is refused naming the field. So
  is a landing fix named by anything but an identity with its version
  (57).
- A package names no kind but its own and the landing fix its kind
  names, an identity and no part of the package (89).
- A statechart kind that lands may name both (94).
- C.4 spells them `landing_fix = "flume/fix@1"` and
  `max_landing_fixes = 2`, gives neither a default, and returns
  `landing_fix` as written.

Rules this design relies on, all in `docs/architecture/core.md`, the one
page whose area the change touches:

- core.2: no tool named. The fixtures and cases name the source `acme`
  and the kinds `train` and `fix`, not C.4's `flume`.
- core.30: one reader of kinds, and a section it does not know returned
  unread and unrefused.
- core.31: a declaration checked whole. It is amended on this branch
  from 52, 57 and C.4 at 8eec6dd.
- core.32: the reader resolves the landing fix. Its details are amended
  on this branch.
- core.35: each case cites its clauses.
- core.37: a clause changed in the statement first.
- core.41: a settings table. Its details are amended on this branch.
- core.42: a kind's identity.
- core.43 and core.44: a package and its hash. core.43's details are
  amended on this branch.

The design departs from none.

## Goals / Non-Goals

**Goals:**

- `landing_fix` and `max_landing_fixes` read on a kind that lands with
  no stages, in place of `fix` and `max_fixes`. Each is returned as
  written and left out when left out. Each is refused naming the field
  where its shape is wrong or the kind has stages.
- `landing_fix` taken only as an identity with its version.
- Every reader in weir that says what a kind holds says it too: the
  crate's docs, `read-package`, `weir check` and the pack's kinds cases.

**Non-Goals:**

- What a landing fix is, when the merge queue sends a branch back, and
  what the bound counts. Those are flume's (flume 203; core.4, core.32).
- Resolving the identity `landing_fix` gives, or checking that the kind
  it names is held, lands or has stages. That is the reader's (58, 89;
  core.32).
- A statechart kind's landing fix (94). weir reads no statechart kind
  yet, and its statechart change reads both fields there.
- The clause index. It moves with the statechart change (decision 10).

## Decisions

### 1. The fields are renamed as C.4 spells them

`Kind::fix` becomes `Kind::landing_fix`, and `Kind::max_fixes` becomes
`Kind::max_landing_fixes`. `KIND_FIELDS` and `DECLARING_FIELDS` name
`landing_fix` and `max_landing_fixes` in place of `fix` and `max_fixes`.
`read_kind` reads them after the kind's stages, as before, since each is
judged against them.

The way not taken is keeping `fix` and `max_fixes` as a second spelling
of each. Record 0005 renames the fields so that the name says the merge
queue. A second spelling would also give a declaration two ways to say
one thing, and C.4 spells one.

### 2. `fix` and `max_fixes` are no longer fields the format names

With the rename, `fix` and `max_fixes` are keys the format does not
name. On a kind's table each is returned unread among `Kind::sections`,
and never fails the read (56; core.30). A table holding `agents` and
`fix`, with nothing else the format names, gives settings again, `fix`
returned unread among them (core.41).

The way not taken is refusing `fix` with a reason that points to
`landing_fix`. 56 says a reader never fails on a field it does not know,
and core.30 returns such a section unrefused. The old spelling never
landed on weir's main, and no declaration in flume's repository at its
main names it.

### 3. `landing_fix` is an identity with its version

`landing_fix` reads through `Identity`'s `FromStr` into
`Option<Identity>`. `Identity` takes only `<source>/<name>@<version>`,
each part checked as core.42 says, and spells back as it was read. So
the identity returns as written, as C.4 says. One version of a kind then
always means one landing fix (record 0005).

A `landing_fix` that is not a string is refused naming
`kinds.<kind>.landing_fix` with "must be a kind's identity with its
version written as a string, such as \"acme/fix@1\"". A string that is
no identity is refused naming the same field, with `NameRefusal`'s
words. Those words say which part is missing or fails:

| `landing_fix` | reason |
|---|---|
| `"fix"` | "`fix` is no identity: it names no source, and an identity is `<source>/<name>@<version>`" |
| `"fix@2"` | the same: it names no source |
| `"acme/fix"` | "`acme/fix` is no identity: it names no version, …" |
| `""`, `"a b"`, `"../fix"`, `"acme/fix@0"` | the part that fails, as for any identity |

Checking the identity's shape is not resolving it (core.32). It is 57's
"a landing fix named by anything but an identity with its version". It
holds 89's "no part of the package" by construction, since no identity
is a path.

The way not taken is keeping `NamedKind` and refusing it when
`identity()` is `None`. That would give a refusal of its own words for
what `Identity` already refuses, naming the part, in the words a lock's
and a package's identity are refused with (core.30). A source of `repo`,
`launch` or a tool's name is not refused. core.42 rules those out for a
source from outside, and a landing fix names a kind from any source,
which its reader resolves.

### 4. `max_landing_fixes` is a whole number of at least 0

`max_landing_fixes` reads as `Option<u64>`. Any value but an integer of
at least 0 is refused naming `kinds.<kind>.max_landing_fixes` with "must
be an integer of at least 0", as `max_fixes` and `max_send_backs` were.
0 reads, as before.

### 5. Either field on a kind with stages is refused naming it

The judgement does not change, only its words. Where the kind's
`stages` reads as one or more, each field is refused naming it, whether
the kind lands or not, after the stages' own problems, `landing_fix`
before `max_landing_fixes`:

| field | reason |
|---|---|
| `landing_fix` | "is on a kind with stages, and only a kind that lands with no stages names its landing fix; a kind with stages names where a failed submission sends its place back by a stage's `on_fail`" |
| `max_landing_fixes` | "is on a kind with stages, and only a kind that lands with no stages names how many landing fixes it runs; a kind with stages bounds its send-backs by a stage's `max_send_backs`" |

A field of the wrong shape is named once, by its shape. Where `stages`
is refused, neither field is judged against it.

### 6. Neither field needs the other

A kind may name `landing_fix` without `max_landing_fixes`, or the bound
without the fix, as before. 57 lists no refusal of either alone, and C.4
gives neither a default (58).

### 7. A package names its landing fix by identity

`read-package` needs no new check. The landing fix is an identity and
no path, so the parts the package checks are unchanged (89; core.43).
The package reads with its `landing_fix` as written, whether or not it
holds the kind named. Its content hash covers the identity as
`kind.toml`'s bytes and no file of the kind named (90; core.44).

### 8. `weir check` lists the landing fix and the bound

Under `  lands with no stages`, `weir check` lists
`  landing fix <identity>` where the kind names its landing fix, the
identity as written, and `  landing fixes at most <n>` where it names
its bound. Neither line is printed where its field is left out. The
lines say the field's name in words, as `  lands with no stages` says
`lands`. The way not taken is `  fixes run as <identity>`, which leaves
out the merge queue that record 0005 has the name say.

### 9. The pack's cases

`ExpectKind` in `crates/weir-kinds/tests/pack.rs` spells the two fields
`landing_fix`, the identity as spelled, and `max_landing_fixes`. Each
is left out where the kind names none, and `kind_differences` holds both
against the kind read, naming the case. Every case's expectation is
read with unknown fields refused, so a case that still says `fix` fails
its read. The three cases of the fix kind keep their names, the fix in
each now its landing fix, as the specs' scenarios keep theirs. Each
spells the fields as renamed and holds `acme/fix@1`:

- `a-kind-that-lands-with-no-stages-names-its-fix` declares
  C.4's `[kinds.train]` with `branch`, `lands = true`, `stages = []`,
  `landing_fix = "acme/fix@1"` and `max_landing_fixes = 2`. It expects
  `train` at version 1 with no stages and both fields (52, 57).
- `a-kind-with-stages-that-names-its-fix` declares `bump`,
  landing, with one stage `code` delivering commits and both fields. It
  expects the refusal naming `kinds.bump.landing_fix` and then
  `kinds.bump.max_landing_fixes`, each reason holding `only a kind that
  lands with no stages names` (57).
- `a-package-of-a-kind-that-names-its-fix` holds `kind.toml`
  alone, declaring `train` as the first case does, held at
  `acme/train@1`. It expects that identity, the hash of that one file
  and `train` with both fields (52, 89, 92).

A fourth case joins: `a-landing-fix-that-is-no-identity`, a kind that
lands with no stages naming `landing_fix = "fix"`. It expects the
refusal naming `kinds.train.landing_fix`, its reason holding `is no
identity` (57). A reader that takes a bare name fails it.

### 10. The clause index stays at a11f6d5

`pack/clauses.toml` moves when a clause is added, removed or
renumbered. 8eec6dd amends 52, 57, 89, 94 and C.4, and adds, removes or
renumbers none. Moving the index there would take in 94 to 99, whose
cases are the statechart change's.

## Risks / Trade-offs

- A declaration written against this bolt with `fix` or `max_fixes` now
  reads with neither field set, each returned among the kind's sections.
  The bolt never landed on weir's main, and flume reads neither field at
  its main. flume's unit that reads them is written for `landing_fix`
  and `max_landing_fixes`.
- `Kind`'s two public fields change name and `landing_fix` its type, so
  a caller that reads them changes with its pin. weir builds no `Kind`
  outside `read_kind`, and flume builds none of weir's.

## Migration Plan

flume moves its `weir-kinds` pin in `Cargo.toml` to the revision of
weir's main that lands this bolt, in its unit
`a-base-places-fix-kind-is-read-from-weirs-format`. weir publishes no
release and tags none, so the pin is that revision. Nothing else moves.
