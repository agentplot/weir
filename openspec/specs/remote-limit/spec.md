# remote-limit Specification

## Purpose
`weir-bounded`: a command run to a limit in a process group of its own
and ended with everything it started, what it printed and whether its
output was cut, the family's limit and grace, and a stopped call's
words (69, C.3; record 0008; core.53).

## Requirements

### Requirement: A command runs to its limit in a process group of its own

`weir-bounded`'s `output` SHALL run a command in a process group of its own and wait for it no longer than the limit it is given. A command SHALL have ended once its leader has exited and no process of its group is left. At the limit its group SHALL be asked to end, given `GRACE`, then killed, so that nothing it started outlives the call, and the command SHALL be said stopped at its limit, `Stopped::AtLimit` (69 as amended at blueprints ad3a1ae; record 0008; core.1, core.53).

#### Scenario: A process that never ends is ended with its group
- **WHEN** a command prints `begun` on standard output and `said` on standard error, starts a `sleep 60` in the background, then sleeps sixty seconds itself, and is run to a limit of 300 ms with a halt never set
- **THEN** it is said stopped at its limit in under five seconds, its standard output is `begun` and its standard error `said`, and the background sleep is gone within a second of the call's return

#### Scenario: A process asked to end says so in its grace
- **WHEN** a command that prints `asked` and exits on being asked to end sleeps sixty seconds, and is run to a limit of 300 ms with a halt never set
- **THEN** it is said stopped at its limit, and its standard output is `asked`

#### Scenario: A process that will not end is killed
- **WHEN** a command that ignores being asked to end loops for ever, and is run to a limit of 300 ms with a halt never set
- **THEN** it is said stopped at its limit no sooner than `GRACE` and in under five seconds

#### Scenario: A child that outlives its leader is ended at the limit
- **WHEN** a command starts a `sleep 60` in the background with its output sent elsewhere, prints the sleep's process id, and exits 0 at once, and is run to a limit of 300 ms with a halt never set
- **THEN** it is said stopped at its limit, its exit is 0, and the background sleep is gone within a second of the call's return

### Requirement: A command that ends within its limit gives its exit and all it printed

A command that ends within its limit SHALL be returned with its exit, everything it printed on each of standard output and standard error, and not said stopped. Its standard input SHALL be what the caller set on the command (core.53).

#### Scenario: A process that ends gives its exit and all it printed
- **WHEN** a command prints `out` on standard output and `err` on standard error and exits 3, run to a limit of ten seconds
- **THEN** it is not said stopped, its exit is 3, and its standard output and error are `out` and `err`, each with its newline

#### Scenario: The input the caller sets is read
- **WHEN** `cat` is run to a limit of ten seconds with its standard input a file holding `asked`
- **THEN** its standard output is `asked`, and it is not said stopped

### Requirement: What a command prints is kept to the part the caller names

`output_kept` SHALL keep the first `kept` bytes of each of a command's standard output and standard error, and SHALL say of each whether what it kept is less than all the command printed there; `output` SHALL keep all of each (core.53).

#### Scenario: What passes the kept part is said cut
- **WHEN** a command prints `whole` and a newline then `part` on standard error and `out` on standard output, and is run keeping 8 bytes of each
- **THEN** its standard error is `whole`, a newline and `pa`, said cut, and its standard output is `out`, not said cut

### Requirement: The crate reads a command's output with no thread

`weir-bounded` SHALL start no thread: a command's standard output and standard error SHALL be written to unnamed temporary files and read once it has ended (core.1, core.53).

#### Scenario: The crate starts no thread
- **WHEN** every source file under `crates/weir-bounded/src` is read
- **THEN** none of them holds `thread::spawn` or `thread::Builder`

### Requirement: A group is asked to end before it is killed

`end_group` SHALL ask a process group to end, wait for it up to the grace it is given, reaping its leader through the caller's `reap` as it waits, then kill what is left, and SHALL return at once when no process of the group is left (core.53).

#### Scenario: A group already gone is left alone
- **WHEN** a command has exited and been waited on, and `end_group` is called for its group with a grace of ten seconds
- **THEN** it returns in under a second

### Requirement: The limit and the grace are the family's

`TRY_FOR` SHALL be thirty seconds and `GRACE` two seconds, boomdrivedev/sluice record 0109's, and no function of the crate SHALL read a setting, an environment variable or a file for either (C.3 as amended at blueprints ad3a1ae; record 0008; core.53).

#### Scenario: The limit and grace are the family's
- **WHEN** `TRY_FOR` and `GRACE` are read
- **THEN** they are thirty seconds and two seconds

### Requirement: A stopped call is named with its call, its remote and its limit

`stopped` SHALL give `<call> with <remote> was still going after <limit>, the most a try may take, and was stopped`, the limit as `span` says it: whole seconds as `30s`, and anything else in milliseconds, as `300ms` (C.3 as amended at blueprints ad3a1ae; core.53).

#### Scenario: A stopped call's words name its call, remote and limit
- **WHEN** `stopped` is given `git fetch`, `origin` and `TRY_FOR`
- **THEN** it gives `git fetch with origin was still going after 30s, the most a try may take, and was stopped`

#### Scenario: A limit is said in seconds or milliseconds
- **WHEN** `span` is given thirty seconds, and then 300 ms
- **THEN** it gives `30s`, and then `300ms`

### Requirement: The crate names no tool

No source file of `weir-bounded` SHALL name a tool of the family (core.2).

#### Scenario: The crate names no tool
- **WHEN** every source file under `crates/weir-bounded/src` is read
- **THEN** none of them names any of the tools core.2 rules out, as a whole word in any case

### Requirement: A command ends at its caller's halt too

`output` and `output_kept` SHALL take, beside the limit, a halt the caller holds, and SHALL end a command at whichever of its limit and its halt comes first, by the one ending: its group asked to end, given `GRACE`, then killed, and the command said halted, `Stopped::Halted`, when the halt came first. The wait that reads the limit SHALL read the halt, with no thread (record 0008 as amended 2026-11-12; core.1, core.53).

#### Scenario: A halt ends a command with its group
- **WHEN** a command prints `begun` on standard output, starts a `sleep 60` in the background and prints its process id, then sleeps sixty seconds itself, and is run to a limit of ten seconds, its halt set from another thread 300 ms after the call begins
- **THEN** it is said halted in under five seconds, its standard output begins with `begun`, and the background sleep is gone within a second of the call's return

### Requirement: A command begun with its halt set runs no process

A command begun with its halt set SHALL run no process, and SHALL be said halted with no exit and nothing printed (record 0008 as amended 2026-11-12; core.53).

#### Scenario: A command begun with its halt set runs no process
- **WHEN** a command that writes a file is run to a limit of ten seconds with its halt already set
- **THEN** it is said halted in under a second, with no exit and nothing printed on either output, and the file is not there

### Requirement: The first of the limit and the halt ends a command

A command SHALL be said stopped by whichever of its limit and its halt came first, and a halt set once its group is being ended at its limit SHALL change nothing (record 0008 as amended 2026-11-12; core.53).

#### Scenario: A halt after the limit changes nothing
- **WHEN** a command that ignores being asked to end loops for ever, and is run to a limit of 300 ms, its halt set from another thread one second after the call begins, within its grace
- **THEN** it is said stopped at its limit, not halted

### Requirement: A halt is one flag its caller holds

`Halt` SHALL be a handle to one flag that every clone of it shares: never set when made by `Halt::default()`, set by `halt` for every clone and never cleared, and read by `halted`. Two handles SHALL be equal when they share one flag. No source of the crate SHALL declare a static, so that a halt is its caller's alone (core.1, core.53).

#### Scenario: A halt is one flag its clones share
- **WHEN** a halt is made and cloned, the clone is set twice, and a second halt is made
- **THEN** the first halt reads set, the second does not, the first equals its clone and not the second halt

#### Scenario: The crate holds no static
- **WHEN** every source file under `crates/weir-bounded/src` is read
- **THEN** no line of them declares a `static` item or holds `thread_local!`

### Requirement: A halted call is named with its call and its remote

`halted` SHALL give `<call> with <remote> was stopped as its caller asked`, naming no limit and no tool (record 0008 as amended 2026-11-12; core.2, core.53).

#### Scenario: A halted call's words name its call and remote
- **WHEN** `halted` is given `git fetch` and `origin`
- **THEN** it gives `git fetch with origin was stopped as its caller asked`
