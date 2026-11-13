---
number: 9
title: The addresses of the moved tools name boomdrivedev, and the names that are no address stay
status: deprecated
date: 2026-11-12
decision-makers:
- the brief of the fix main--weirs-addresses-name-boomdrivedev
- the code stage of the fix main--weirs-addresses-name-boomdrivedev
---

# The addresses of the moved tools name boomdrivedev, and the names that are no address stay

Status: deprecated on 2026-11-13. It holds a one-off move, which is the plan's and no record's (crew's rule briefs.15 as amended 2026-11-13).

Sources: the brief; `gh repo list boomdrivedev`, `gh repo list agentplot` and `gh api repos/agentplot/<repo>` for each repository weir's tree names, read on 2026-11-12; boomdrivedev/blueprints, which holds `006fc34` and answers the style guide's address the Vale rules link to; boomdrivedev/timber's record 0032, the same fix in timber; afterthought/kit-standard `docs/orgs/`; record 0003, whose rename is the precedent; rules core.2 and core.37; the OpenSpec change weirs-addresses-name-boomdrivedev.

## Context and problem statement

On 2026-11-12 every repository weir's tree names moved from the agentplot owner on GitHub to boomdrivedev: blueprints, boomdrive, boomdrive-cloud, sluice, flume, timber, crew, weir, millpond, headwater, eddy, sounding, boom, site-theme and strategy. GitHub answers each old address with the new one. agentplot/boomdrive-web stays, and agentplot/flywheel stays archived there. A commit cited in agentplot/blueprints is the same commit in boomdrivedev/blueprints. weir has no flake, site, policy or skill of its own that names an address: its `devenv.yaml` takes nixpkgs, treefmt-nix and git-hooks from their own owners, and its skills under `.claude/` are OpenSpec's.

The tree said agentplot in five kinds of place: addresses of the moved tools; addresses of the two that stay; names that take agentplot and are no address; fixtures that name weir's own repository; and the archived OpenSpec changes. weir-branch's, weir-signals' and weir-cadence's tests of core.2 read a ruled-out name anywhere in a word, so a module comment citing boomdrivedev/blueprints read to them as naming the tool `boomdrive`. Five questions are left to the fix:

1. Do the names that take agentplot move?
2. Which records move their citations?
3. Do the fixtures move?
4. How does a crate cite the statement under core.2?
5. What of the specs?

## Considered options

For the names, which are the family's name in prose, "the agentplot tool family", the kit standard's org in `.config/kit.toml`, the license's holder, and the design and ops streams `agentplot-design` and `agentplot-ops` with the keys of their signals:

1. Keep them.
2. Move them to boomdrivedev.

For the records:

1. Move the citations in `docs/adr/`, and keep the archived OpenSpec changes as written.
2. Move every record, the archived changes too.
3. Move none.

For the fixtures, a send's target `agentplot/weir fix` or `agentplot/weir change` and its outcome `landing agentplot/weir#14` or `landing agentplot/weir 1a2b3c4`, in weir-signals' tests and the pack's inbox cases:

1. Move them.
2. Keep them as made-up data, as timber's record 0032 keeps its own.

For a crate's citation of the statement:

1. Each test of core.2 reads a ruled-out name as a whole word, as weir-pack's, weir-adapters', weir-bounded's and the command's already do, and the crates cite boomdrivedev/blueprints.
2. Amend core.2 to name `boomdrivedev` as not ruled out, and keep the tests reading substrings.
3. Cite the statement in the crates with no owner.

## Decision outcome

The names stay, option 1. A name is no address, and the brief moves addresses. The kit standard knows an org `agentplot` and none named `boomdrivedev`, so `.config/kit.toml` keeps naming the profile it reads. The streams are Flume's names for its recipients, and a signal is cited by the key it was taken under.

The records are option 1. A record is read for its citations, which a reader follows, so the citations move at the revisions they name; record 0003's line on agentplot/boomdrive-web and its line on the archived agentplot/flywheel stay, those addresses having stayed. An archived change is the change as it was proposed and archived; record 0003's rename left the archive as written, and this fix does the same.

The fixtures move, option 1. Unlike timber's, which name a checkout at `example.com` and no tool, weir's name weir's own repository by its address, and the brief moves every such address. A pack case gives both the records it starts from and the records a host must write, so a host that passed it before passes it after.

A crate's citation is option 1. core.2 rules out naming a tool, and `boomdrivedev` names an owner. Four of the seven tests already read a name as a whole word, so `a screw and soundings` names no tool; the other three take that reading, and core.2 needs no amendment. Option 2 would change a rule for what the rule never ruled out, and option 3 would leave the statement's address with no owner.

The specs move through the OpenSpec change weirs-addresses-name-boomdrivedev: conformance-pack's index requirement and remote-limit's requirement on the limit cite boomdrivedev/blueprints and boomdrivedev/sluice. No spec's Purpose names an address.

### Consequences

- The README, CLAUDE.md, `openspec/config.yaml`, `Cargo.toml`'s and `devenv.nix`'s comments, `.gitignore`, `.vale.ini`, the Vale rules' links, the rules' narrative and core.53's sources, the crates' module comments, the records' citations and the specs name boomdrivedev/blueprints, boomdrivedev/boomdrive, boomdrivedev/sluice, boomdrivedev/flume and boomdrivedev/timber.
- The pack's index names its statement `boomdrivedev/blueprints:design/weir/requirements.md`, at the same commit, `006fc34`.
- weir-branch's, weir-signals' and weir-cadence's names tests read a ruled-out name as a whole word, each with a test that `boomdrive-hosted` is named and `boomdrivedev/blueprints` is not.
- `openspec/config.yaml` keeps the local path of one host's checkout of blueprints, whose folder name is the host's.
- The statement in boomdrivedev/blueprints, `design/weir/requirements.md`, still names agentplot/strategy, agentplot/sluice and agentplot/weir; moving them is blueprints' own change.
- No rule changes: this record moves addresses once and binds no later diff.
