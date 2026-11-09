# Design

## Context

openspec 1.14.3, the release devenv.nix pins, warns of a requirement
whose text runs over 500 characters, and `--strict` fails a warning as
an error. Over weir's specs on main at 1cc073a it refuses `adapter-checks`,
`cadence`, `capture-take`, `host-conformance`, `import-lines`,
`inbox-cases`, `kind-packages`, `kinds`, `meeting-file` and
`signals-folder-import`, 23 requirements in all. No task of weir's gate
runs it.

sluice, flume and boomdrive each hold the same check as a `check-fast`
task of their root project, `specs`. sluice split its one long
requirement by editing its spec. weir changes a spec only through an
OpenSpec change (CLAUDE.md, *Specs*), so the splits here are this
change's delta specs.

Rules this relies on: core.39, every check a task of the gate and no
verification a command run by hand; core.40, every tool a gate task
runs at the version the tree pins; core.37, a change that bears on a
clause cites it. No page of `docs/architecture/` but `core.md` exists,
and this change touches no crate's area.

## Goals / Non-Goals

**Goals:**

- The gate fails a spec `openspec validate --strict` refuses.
- Every requirement of weir's specs reads strictly, saying what it said.

**Non-Goals:**

- Validating open changes in the gate. sluice did it in a chore of its
  own, after its specs passed. Here it waits for one too.
- Rewording any requirement beyond what its split needs, or correcting
  one. A requirement whose words disagree with a rule keeps its words.
  For example, `kinds` counts six deliverables where core.31 counts
  seven spellings. Fixing that is a change of its own.

## Decisions

### 1. One task, as the family spells it

The root project's `specs` runs `openspec validate --specs --strict
--no-interactive`, with `OPENSPEC_TELEMETRY=0` and
`OPENSPEC_NO_UPDATE_CHECK=1`. Its inputs are `/openspec/specs/**/*`,
`/openspec/config.yaml`, `/devenv.nix` and `/devenv.lock`, and it is
tagged `check-fast` and run from the workspace root. This is sluice's,
flume's and boomdrive's task word for word, so the family holds one
check (CLAUDE.md, *The family*). It caches like the root's other
checks: the validator reads only its inputs, at the pinned release.

The other way was a script of weir's own around the validator. It was
not taken: it would be a second spelling of a check the family already
holds.

### 2. A split says the same

Each long requirement becomes a requirement under its own name, kept as
MODIFIED, and one or more added beside it. Together they say what it
said:

- A list of conditions is cut between its items, and each part keeps
  the requirement's subject and its SHALL. For example, *A declaration
  is refused whole when* starts each part of `kinds`' refusals.
- A citation goes with the words it cited. Where one citation covered a
  whole list, each part of the list carries it.
- A sentence that said what one part holds is written as that part's
  SHALL. For example, the declaration case's answers in `kinds` read
  "SHALL expect one of two answers".
- The requirement split keeps every scenario it had, word for word.
  `openspec archive` refuses a MODIFIED requirement that drops a
  scenario the spec holds. OpenSpec's own guidance for a change made to
  split a long requirement says the same: keep its header and every
  scenario, cut its text to one behavior, and add each behavior cut as
  a requirement with its own scenarios.
- Each added part carries a copy, word for word, of each scenario that
  shows its behavior. No scenario is written new, so each is proved by
  the test that proves it now. For example, `host-conformance`'s *A
  case read*, proved by `pack:test`'s `a_case_read`
  (`crates/weir-pack/tests/host_cases.rs`), shows a step's input and
  its expected report both. So it stands under the host case, under a
  step's invocation and input, and under a step's report.
- A requirement naming the pack's cases, whose one scenario is *The
  cases named*, keeps that scenario, and each part split from it
  carries a copy. "Each kind of case this requirement names" then holds
  of each part.

The other way was to remove each long requirement and add its parts
under new names, each scenario moved and not copied. It was not taken.
It would rename 23 requirements a reader or a test may cite, and it
would make every split into a removal.

OpenSpec appends an added requirement at the end of its spec. So a
split requirement's parts lie apart, and no requirement points at
another by place. Each reads alone, by its own name.

### 3. The splits

| Spec | Requirement | Split into |
|---|---|---|
| adapter-checks | The adapter checks hold a run's lines to the contract | the checks given and naming every breach; a line or an offer that breaks the contract; raw material, an excerpt or a completion that does |
| adapter-checks | The meeting checks hold a meeting adapter to 66 | what they are given and the completion; an offer that breaks 66 |
| cadence | The pack holds cadence's cases | the cases and their clauses; a case of the wrong shape |
| capture-take | The report is written and read as weir-report/1 | its format and its reading back; what it holds |
| host-conformance | A host case holds its steps, its starting records and the records expected | its folder; a step's invocation and input; a step's expected report |
| host-conformance | Each step is one of three invocations with the case's flags | every step's flags, an intake's and an import's; a run step's declaration |
| host-conformance | The report a host prints is compared with the step's | the report compared; a run step's listing and cursor |
| host-conformance | The records at the tip are compared as written | the tip compared; a capture's fields of the host's own; its pointer |
| host-conformance | The pack's host cases cover what a host does | S1 to S4; a malformed line, raw material with no raw place and an unnamed field; an import |
| import-lines | A move of any word is carried as it stands | any word read; what a move holds |
| inbox-cases | An inbox case holds a starting inbox, one operation and what must come of it | its folder; its operation and the refusal it expects |
| inbox-cases | The inbox cases hold the inbox's clauses | 25 to 31 and every clause cited; batches, items, briefs and sends; imports, formats and the layout |
| kind-packages | read-package reads a package at the identity it is held at | the read; a kind that lands with no stages |
| kind-packages | A package declares exactly one kind and names no other | its one kind; its landing fix |
| kinds | A declaration reads into its kinds | the kinds and their defaults; a kind that lands with no stages |
| kinds | A stage reads with its name, prompt and end | its name, prompt, end and settings; its standing instructions and what it makes |
| kinds | A declaration is checked whole and refused naming the field | the refusal's fields and order; a kind's stages; a stage; a field's shape; a kind with stages naming a landing fix |
| kinds | weir check reads a declaration | the listing and the refusal; a kind that lands with no stages and its landing fix |
| kinds | The pack holds the kind format's cases | the cases and the crate's tests; a case of the wrong shape; a declaration's case's answers; each kind it expects; 51 to 58, S10 and S11; S12, settings and a repeat; a kind that lands with no stages; a landing fix |
| kinds | A kind's table holding agents and no stages gives settings | the settings; left to the reader; a table that is a kind |
| kinds | A kind that lands with no stages may name its landing fix and its bound | the two fields; left out; their old spellings |
| meeting-file | The event time is the meeting's start where the file states it | the ways in order; front matter |
| signals-folder-import | One import line per capture folder, in the order of their names | the capture folders; the one `moves.rec` |

## Risks / Trade-offs

- [A split could say more or less than the requirement it splits] →
  every condition and citation of each long requirement is carried into
  one of its parts. Every scenario stays where it was, with copies only
  where decision 2 says. No test changes, and each scenario is held by
  the test that held it.
- [A scenario stands under more than one requirement] → each copy is
  word for word, so one test still proves it, and a change to it is a
  change to each copy.
- [An added requirement lies away from the one it was split from] → each
  is named for what it holds, and `openspec show` lists a spec's
  requirements whole.
