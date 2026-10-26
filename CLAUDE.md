# weir

The shared code of the agentplot tool family, one Rust workspace. It
defines, once, the capture contract, the record formats, the
shared-branch write protocol, the kind format, cadence, the built-in
adapters and the conformance pack. millpond, headwater, eddy, sounding
and flume build on it, and Flywheel later. Designed, not built.

The design is in agentplot/blueprints: the capture design,
`design/capture/design.md`, and its plan, `plan.md` beside it. weir's
statement, `design/weir/requirements.md`, is written from them next
(plan step F1). This repository cites the statement's clauses by number
and never restates them. Where this repository and the design disagree,
the design wins.

## Rules

The rules are in `docs/architecture/`: `core.md` binds wherever a change
is made, and an area page, when one exists, binds the paths its
frontmatter names. The decisions behind them are `docs/adr/`, one MADR
record each, proposed until the operator's yes. A design or a commit
that relies on a rule cites its id; one that departs from a rule says
which and why. Where `docs/architecture/` and the narrative docs
disagree, `docs/architecture/` wins. A rule is sent to `core.md` or its
area page with an id, the next free number in its area, amended in the
same commit as the decision that changes it. The kit standard binds
beside them as its `code` profile says (`.config/kit.toml`).

@docs/architecture/core.md

## Checks

Every tool runs through the shell, one command at a time:
`devenv shell -- <command>`. moon runs the checks, tagged by layer:

| layer | what | how long |
|---|---|---|
| `check-fast` | format (treefmt) and the README's prose (Vale) | seconds |

`devenv shell -- check-fast` runs them (`moon run :#check-fast`);
`devenv shell -- moon run root:format` and `prose` run one each. Each
crate brings its cargo tests and lint as a moon project of its own when
it lands.

## Commits

Stage by pathspec and commit the paths you wrote: `git add <paths>`, then
`git commit -m "<subject>" -- <paths>`. Never `-a`, never `add .`.
Conventional Commits subjects, scope the part of the tree touched; the
commit-msg hook refuses any other.
