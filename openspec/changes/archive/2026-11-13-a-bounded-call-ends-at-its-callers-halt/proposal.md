# Proposal

## Why

`weir-bounded` ends a try of a call to a remote at its limit alone. Its
caller hands it a command and a limit, and waits, so it can neither end
the wait early nor reach the group to end it. Boomdrive's runner ends
the tries it has in flight when it is stopped, so a stop leaves no git
running and the runner stops well within launchd's twenty seconds. It
built that halt in `boomdrive-bounded`, but its main now runs every
call to a remote through `weir-bounded`, and its core.58 rules out the
group's ending in a crate of its own. A stop of the runner therefore
leaves its fetch, its listing and weir-branch's read and write of crew's
state repository running up to thirty seconds and two of grace, past
launchd's kill.

Record 0008's amendment of 2026-11-12, accepted by agentplot-design and
landed on weir's main at 41cd694, gives `weir-bounded` a halt: a try
ends at whichever comes first of its limit and a halt its caller holds,
by the one ending, with words of its own. A try begun with its halt set
runs no process, and weir-branch passes its caller's halt to each try.
core.53 says so. The change was queued in afterthought/crew-state's
`plan.rec`, at agentplot's main `a158ca88`.

## What Changes

- `weir-bounded` gains `Halt`, one flag its clones share, which a caller
  sets from any thread and the crate holds in no static. `output` and
  `output_kept` take a halt beside the limit. A try ends at whichever
  comes first, by the one ending, and one begun with its halt set runs
  no process. `Ended` says which came first, `Stopped::AtLimit` or
  `Stopped::Halted`, and gives no exit where no process ran.
  **BREAKING** for a caller of `output` or `output_kept`, which passes a
  halt, `&Halt::default()` where it holds no stop, and reads `stopped`
  and `status` as options.
- `weir-bounded` gains `halted(call, remote)`, a halted call's words:
  `<call> with <remote> was stopped as its caller asked`.
- weir-branch's `Branch` carries a halt, never set unless its caller
  gives one by `with_halt`, and `weir_branch::Halt` is `weir-bounded`'s.
  Its fetch, its listing and its push each take the branch's halt. A
  halted fetch or listing fails the read. A halted push fails the write
  at its move and is never replayed. A branch with no remote takes no
  halt. Every caller that gives none runs as before.
- The README's row for `weir-bounded` says a try ends at its limit or
  its caller's halt.

## Capabilities

### New Capabilities

None.

### Modified Capabilities

- `remote-limit`: a command ends at its limit or its caller's halt,
  whichever comes first, one begun with its halt set runs no process,
  and a halted call has words of its own.
- `shared-branch-write`: a read or a write ends its tries at its
  caller's halt, a halted push is never replayed, and a halted call's
  failure follows git's last line.

## Impact

- `crates/weir-bounded`: `Halt`, `Stopped`, `halted`, and the halt in
  `output`, `output_kept` and their wait; `Ended`'s `status` and
  `stopped` become options; its tests pass a halt.
- `crates/weir-branch`: `Branch` carries a halt and gains `with_halt`;
  `lib.rs` re-exports `Halt`; `git.rs`'s `reach` takes the halt and
  names a halted call; `read_tip` and `move_to` pass the branch's halt.
  `Branch::remote`, `Branch::local`, `read`, `write` and `Ask` keep
  their signatures, so the `weir` crate's calls are unchanged.
- No record, layout, case, moon project or command line changes.
- Boomdrive's two calls of `weir_bounded::output` change when it moves
  its pin of weir past this change, in a change of its own, which its
  record 0042's amendment of 2026-11-12 already plans. sluice takes the
  halt for Ctrl-C if it chooses, when it takes `weir-bounded`.
