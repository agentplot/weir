# weir

The shared code of the agentplot tool family, one Rust workspace. It
defines, once, the capture contract, the record formats, the
shared-branch write protocol, the kind format, cadence, the built-in
adapters and the conformance pack. millpond, headwater, eddy, sounding
and flume build on it, and Boomdrive later. Designed, not built.

The design is in agentplot/blueprints: the capture design,
`design/capture/design.md`, and its plan, `plan.md` beside it. weir's
statement, `design/weir/requirements.md`, is written from them next
(plan step F1). This repository cites the statement's clauses by number
and never restates them. Where this repository and the design disagree,
the design wins.

## The family

**The family.** This tool works with the others of the Boomdrive
family: sluice (the merge queue), Flume (places and their agents),
Boomdrive (the runner), timber, weir and boom. Each tool's statement is
in agentplot/blueprints, `design/<tool>/requirements.md`. Before
designing or fixing a behavior here, look at how the related tools do
the same thing, in their code and their statements: readiness,
configuration, credentials, retries, what a place is. Where two tools do
one thing, reconcile them into one design with one owner the others call
or cite, and say so in the design. Never fix one tool alone when the
cause is that the tools differ.

**Specs.** `openspec/specs/` changes only through an OpenSpec change: a
proposal, a delta spec, and tasks, validated with `openspec validate
--strict` and archived with `openspec archive`, so the spec reconciles.
This holds for a fix or a chore too. Where the work changes a
requirement, it carries that change in the same branch. Never edit a
spec file directly.

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
| `check-fast` | format (treefmt), the README's prose (Vale), a new place's first commit through the hooks, and OpenSpec's Claude Code files | seconds |

`devenv shell -- check-fast` runs them (`moon run :#check-fast`);
`devenv shell -- moon run root:format`, `prose`, `hooks` and
`openspec-claude` run one each. Each crate brings its cargo tests and
lint as a moon project of its own when it lands.

## Claude Code

**Claude Code.** The MCP servers are `devenv.nix`'s
(`claude.code.mcpServers`), devenv's hosted `mcp.devenv.sh` among them,
each approved by name, so a session in a trusted folder starts them
unasked. devenv writes `.mcp.json` and `.claude/settings.json` on
entering the shell, and as a place's readiness sources it, and git
ignores both (agentplot/boomdrive's record 0022; record 0007): a Claude
Code setting of the repository's goes into devenv's options, never into
those files. OpenSpec's skills and commands under `.claude/` are tracked
as the OpenSpec `devenv.nix` pins writes them; when the pin moves,
`devenv shell -- sh check-openspec-claude.sh --write` writes them again,
and `root:openspec-claude` fails until it has.

## Commits

Stage by pathspec and commit the paths you wrote: `git add <paths>`, then
`git commit -m "<subject>" -- <paths>`. Never `-a`, never `add .`.
Conventional Commits subjects, scope the part of the tree touched; the
commit-msg hook refuses any other.
