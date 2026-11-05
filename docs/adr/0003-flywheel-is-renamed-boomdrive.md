---
number: 3
title: Flywheel is renamed Boomdrive
status: proposed
date: 2026-11-04
decision-makers:
- Chuck Swanberg
---

# Flywheel is renamed Boomdrive

Status: proposed, the user's decision in their own words, until they have seen it as a record.

Deciders: the user, 2026-11-04, for every repository at once: "the ecosystem, the org, the branding, the cloud and the opensource runner tool are all boomdrive. There is also a "boom" open source tool that does the claim reconciliation."
Sources: the brief of the chore flywheel-is-renamed-boomdrive, which carries the user's words and the names below.

## Context and problem statement

Flywheel, the product, its brand and its ecosystem, is renamed, and every repository of the family takes the new names together. weir names Flywheel as the host that builds on its crates later, rules its name out of the crates' code (core.2), reads crew's folders on a partition's state branch, which crew called the flywheel branch, and cites tailrace's records as the family's precedent (core.39, core.40). Which names does weir write from now on?

## Considered options

1. Every live mention takes the new name at once: code, tests, rules, records, specs and the README; an archived change, and a record's own words about its history, keep their text.
2. New text takes the new names and existing text keeps the old ones until it is next touched.

## Decision outcome

Option 1, as the user decided it for every repository. The names:

- Flywheel, the product, brand and ecosystem, and the family of open tools on their sites, sluice, flume, timber, weir and the rest, is **Boomdrive**: one word, a capital B in prose, `boomdrive` in code, commands, packages, paths and repository names, and `Boomdrive` where the code's case needs a capital. Never "Boom Drive", "BoomDrive", `boom_drive` or `boom-drive`.
- Flywheel Cloud, the control plane, is **Boomdrive Cloud**, in agentplot/boomdrive-cloud, which was flywheel-cloud.
- The runner is **Boomdrive**, in agentplot/boomdrive, which was flywheel-runner. Its command `flywheel` is `boomdrive`, and its crates `flywheel` and `flywheel-hosted` are `boomdrive` and `boomdrive-hosted`.
- flywheel-web is agentplot/boomdrive-web.
- tailrace is **timber**, in agentplot/timber: its crate, command, site and records.
- A customer's unit of work, "a Flywheel", is **a drive**, plural drives, and `Drive` in code, the interface's word and the code's the same.
- crew's word for a partition's state branch, "the flywheel's branch", is **drive** too, following the unit's word: the drive's branch, a drive branch. A runner's mark on its launches, `flywheel:<partition>/<n>`, is `boomdrive:<partition>/<n>`.
- agentplot/flywheel, the original, is archived; a mention of it as history stays, naming it as archived.
- boom, the open tool that does the claim reconciliation, keeps its name, as does anything third-party that says flywheel.
- The same pass spells **enroll, enrolls, enrolled, enrolling, enrollment**, never enrol or enrolment.

### Consequences

- core.2 rules out `Boomdrive` in Flywheel's place, and each crate's names test reads `boomdrive`; no tool is added to the list or taken from it.
- core.39 and core.40 cite timber's record 0008, which was tailrace's; record 0001 names timber among the records it took as its model.
- The signals-folder import, its specs and its tests call a partition's state branch the drive branch, and the fixture of crew's `moves.rec` says the drive's curation moves.
- No rule's decision changes: a rule that names a renamed thing says the new name and cites this record.
- The archived OpenSpec changes under `openspec/changes/archive/` keep their text.
