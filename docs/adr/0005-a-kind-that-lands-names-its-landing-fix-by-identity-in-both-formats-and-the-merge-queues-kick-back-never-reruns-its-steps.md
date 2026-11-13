---
number: 5
title: A kind that lands names its landing fix by identity, in both formats, and the merge queue's kick-back never reruns its steps
status: accepted
date: 2026-11-07
decision-makers:
- Chuck Swanberg
- agentplot-design
---

# A kind that lands names its landing fix by identity, in both formats, and the merge queue's kick-back never reruns its steps

Status: accepted by the user on 2026-11-07, on their card: "Yes, write it into both formats".

Deciders: the user, 2026-11-07, in agentplot-design's pane: "It's not going to make sense on the new one to have the same kind that builds it do the fix. That kind is going to be a state machine with all the steps but when we do the fix, we don't need all the steps"; "Wouldn't it include the fully qualified change name, including the version?"; and "Should 'fix' be the right term, or should it be something more related to the merge queue … since this is specifically for being kicked back from that? A state machine, by contrast, might run a verify step internally and then have its own concept of what step should be run after that verification". Then, on agentplot-design's card of the same day, which showed the fields and this outcome: "Yes, write it into both formats". The field names and the wording are agentplot-design's.

Sources: boomdrivedev/blueprints `design/weir/requirements.md` 52, 57, 89, 94 and C.4, and `design/flume/requirements.md` 157 and 175, as amended with this record; boomdrivedev/flume record 0056 and its amendment of 2026-11-07, which first gave a kind with no stages a fix kind and a bound, there spelled `fix` and `max_fixes`.

## Context and problem statement

boomdrivedev/flume record 0056 has Flume launch a fix on a base place's branch when the merge queue forecasts a conflict with its base or a submission fails. The base place's kind names the kind the fix runs as and how many fixes it runs. Its amendment gave weir's format two fields for it on a simple kind with no stages, `fix` and `max_fixes`, the fix kind named by a name its reader resolves.

The user read that example as the same kind doing its own fix, and asked how it fits the statechart format, where a kind that builds is a statechart of all its steps: a kick-back from the merge queue should not run them all again. They asked for the fix kind's full identity with its version, and for a name that says the field is for the merge queue's kick-back, since a statechart has its own verify and its own transitions after it.

## Considered options

1. **`landing_fix` and `max_landing_fixes`, an identity with its version, in both formats.** A simple kind that lands with no stages, and any statechart kind that lands, may name the kind run when the merge queue sends its branch back, and how many such fixes run before its place waits on its caller.
2. **`fix` and `max_fixes` as built, on simple kinds with no stages alone.** The statechart format decides its own when Flume runs it.
3. **The statechart's own send-back** for a kick-back too, as clause 94 has it for a stage's findings.

## Decision outcome

Option 1, the user's decision.

- **The name says the merge queue.** `landing_fix` is the kind a place runs when the merge queue sends its branch back: it conflicts with its base, or a check of the base fails on it. Nothing else starts it. `max_landing_fixes` is how many before the place waits on its caller.
- **An identity with its version.** `landing_fix` names a kind as `<source>/<name>@<version>`, such as `flume/fix@1`. A bare name, or an identity with no version, is refused naming the field, so one version of a kind always means one fix, and a changed fix is a new version of the kind that names it (weir 57, 89).
- **Both formats.** On a simple kind, only one that lands with no stages may name it; a simple kind with stages keeps `on_fail` and `max_send_backs`, and naming a landing fix on it is refused. Any statechart kind that lands may name it. Its states keep their own transitions, a verify and what follows it among them, and the landing fix touches none of them: a kick-back runs a place of the named kind, never the statechart again (94).
- **Flume reads it** for a base place now (flume 203), and for a statechart kind's place when Flume runs statechart kinds, which designs then how that fix lands beside the statechart's own place (flume 175, 177).

### Consequences

- Good: a kick-back from the merge queue costs one small fix, never a statechart's whole run, and the kind that names it says which fix, at which version.
- Good: the field's name says when it runs, apart from a statechart's own verify and send-back.
