# Design

## Context

Record 0009 decides what moves: the addresses of the moved tools, the
records' citations and the fixtures that name weir's own repository;
and what stays: names that are no address and the archived changes. The
statement, `design/weir/requirements.md` in boomdrivedev/blueprints, is
read at the same commits as before; a commit of agentplot/blueprints is
the same commit there. This change touches no area page, there being
none, and departs from no rule.

## Decisions

1. **The specs follow the tree.** The two requirements that name a moved
   address name boomdrivedev, word for word otherwise, and keep their
   scenarios (core.37). No spec's Purpose names an address.
2. **A ruled-out name is a whole word.** core.2 rules out naming a tool
   of the family, and four of weir's seven tests of it already read a
   name as a whole word in any case: no letter, digit or `_` on either
   side. weir-branch's, weir-signals' and weir-cadence's take the same
   reading, so a module comment citing boomdrivedev/blueprints names no
   tool, and `boomdrive-hosted` still does. Each gains
   `a_tool_is_named_as_a_whole_word`, as weir-pack's has (core.2,
   core.39).
3. **The pack's cases move whole.** A case's starting records and the
   records it holds a host to name the same target, so each case's
   files move together and a host that passed it passes it still
   (core.35, core.46).
