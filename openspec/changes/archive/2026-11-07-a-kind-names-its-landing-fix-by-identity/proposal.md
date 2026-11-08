# Proposal

## Why

Record 0005 was accepted by the user on 2026-11-07, on their card:
"Yes, write it into both formats". It has a kind that lands name its
landing fix by the fix kind's identity with its version. The landing
fix is the kind a place runs when the merge queue sends its branch
back. The statement at blueprints 8eec6dd says so for weir:

- 52 lets a kind that lands with no stages name its landing fix, by
  that kind's identity with its version. It may also name how many
  landing fixes it runs before its place waits on its caller.
- 57 refuses either on a kind with stages, naming the field. It also
  refuses a landing fix named by anything but an identity with its
  version, naming the field.
- 89 lets a package name that one kind beside its own, an identity and
  no part of the package.
- 94 lets a statechart kind that lands name both, as a simple kind with
  no stages may.
- C.4 spells the two fields `landing_fix = "flume/fix@1"` and
  `max_landing_fixes = 2` on `[kinds.train]`. It gives neither a
  default, returns `landing_fix` as written and refuses one that is no
  identity with its version.

This bolt, `weir-kinds-name-their-fix`, built the change
`a-kind-with-no-stages-names-its-fix` from the statement at 31f9f1c.
That change spells the fields `fix` and `max_fixes`, and reads `fix` as
a kind's name in any of its four forms. Record 0005's consequences say
these fields are renamed before the bolt lands, and that their reading
takes an identity alone. flume's queued unit
`a-base-places-fix-kind-is-read-from-weirs-format` reads
`landing_fix` and `max_landing_fixes`.

## What Changes

- `read-kinds` reads `landing_fix` and `max_landing_fixes` on a kind's
  table into the kind, in place of `fix` and `max_fixes`. Neither old
  field is one the format names any more, so each is returned unread
  among a kind's sections, as any field the format does not name is (56).
- `landing_fix` is an identity with its version,
  `<source>/<name>@<version>`, returned as written. A bare name, a name
  with a version and no source, or a source and a name with no version
  is refused naming the field. So is a value that is not a string, or a
  string that is no identity (57, C.4).
- `max_landing_fixes` is a whole number of at least 0, as `max_fixes`
  was. Each field is left out when left out, and either reads without
  the other (58).
- A kind with stages that names either is refused naming the field,
  whether it lands or not (57).
- A table holding `agents` and either field reads as a kind, not as
  settings (core.41).
- `read-package` reads a package whose kind names its landing fix. The
  identity is not a part, and the package need not hold that kind (89).
- `weir check` lists `  landing fix <identity>` and
  `  landing fixes at most <n>` under a kind's line saying it lands with
  no stages.
- The pack's kinds cases expect a kind's `landing_fix` and
  `max_landing_fixes`. The three cases of the fix kind spell the fields
  so and hold an identity. A fourth case holds a landing fix that is no
  identity, refused naming the field.
- core.31, with the details of core.32, core.41 and core.43, says it as
  a rule. These edits are made on this branch with the change.

## Capabilities

### New Capabilities

None.

### Modified Capabilities

- `kinds`: a kind that lands with no stages may name its landing fix by
  identity with its version, and its bound. The refusals of either, the
  settings rule, `weir check`'s listing and the pack's kinds cases
  follow.
- `kind-packages`: a package whose kind names its landing fix reads,
  the identity no part of it.

## Impact

- `crates/weir-kinds`: `Kind`'s fields `fix: Option<NamedKind>` and
  `max_fixes: Option<u64>` become `landing_fix: Option<Identity>` and
  `max_landing_fixes: Option<u64>`, read by `read_kinds`
  (`src/kinds.rs`), with the docs of the crate and of `read_kinds`.
  `NamedKind` stays, as a reader's way to read a kind's name.
- `crates/weir`: `weir check`'s listing, and its tests.
- `pack/kinds/`: the case format's expected kind spells the two fields
  as renamed, read by `crates/weir-kinds/tests/pack.rs`. The three cases
  of the fix kind spell them so and hold `acme/fix@1`, and
  `a-landing-fix-that-is-no-identity` joins them.
- `docs/architecture/core.md`: the header cites record 0005, and core.31
  and the details of core.32, core.41 and core.43 are amended from 52,
  57, 89 and C.4 at 8eec6dd.
- `pack/clauses.toml` is unchanged. 8eec6dd amends 52, 57, 89, 94 and
  C.4, and adds, removes or renumbers no clause of weir's.
- The statechart side of 94 waits on weir's statechart kinds (94, 95),
  a change of its own. weir reads no statechart kind yet.
- flume pins weir-kinds at the revision of weir's main that lands this
  bolt, in its unit `a-base-places-fix-kind-is-read-from-weirs-format`.
