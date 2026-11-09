# Design

## Context

openspec 1.14.3, the release devenv.nix pins, warns of a requirement
whose text runs over 500 characters, and `--strict` fails a warning as
an error. `root:specs` runs it (core.39, core.40). Over weir's specs on
main at 52b9552 it refuses `conformance-pack`, whose *The pack keeps an
index of the statement's clauses* the delivery change lengthened to
read the index at blueprints 006fc34 with A.14 and B.7. No other spec
or requirement is refused.

weir changes a spec only through an OpenSpec change (CLAUDE.md,
*Specs*), so the split is this change's delta spec. Rules this relies
on: core.37, a change that bears on a clause cites it; core.39 and
core.40, the gate's check at the pinned release. No page of
`docs/architecture/` but `core.md` exists, and this change touches no
crate's area. It satisfies no clause of its own: the requirement keeps
citing what it cited, 67 among the index's clauses through core.37.

## Goals / Non-Goals

**Goals:**

- `conformance-pack` reads strictly, saying what it said.

**Non-Goals:**

- Rewording the requirement beyond what its split needs.
- Validating open changes in the gate, which the-specs-read-strictly
  left to a chore of its own.

## Decisions

### 1. The split, as the 23 before it

The requirement keeps its name, as MODIFIED, and holds where the index
lies, what it holds under each section, the commit it was read from and
the numbers it leaves out. *The index holds numbers alone and follows
the statement* is added beside it, holding that the index holds no
clause's words (core.37) and changes in the change that follows the
statement. Each sentence goes to one part whole, with its citation.

The requirement split keeps *The index read*, word for word, as
`openspec archive` requires. The added part carries a copy, word for
word, since the index read under its sections is what shows it holding
numbers and sections. So `pack:test`'s
`the_index_holds_every_clause_under_its_section`
(`crates/weir-pack/tests/clauses.rs`) proves both.

The other way was to cut the commit and the numbers left out into a
part of their own. It was not taken: that part and the one it was cut
from would each hold half of what the index holds, where this split
puts what the index holds in one requirement and how it is kept in the
other.

## Risks / Trade-offs

- [The split could say more or less than the requirement it splits] →
  each of its four sentences is carried whole into one part, citation
  and all, and no test changes.
