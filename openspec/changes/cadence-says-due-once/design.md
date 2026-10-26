# Design

## Context

The tree holds no crate yet: `Cargo.toml` lists the planned crates in a
comment with `members = []`, the root moon project runs treefmt and
Vale, and `.prototools` pins moon alone. This change adds the first
crate the statement names for cadence, `weir-cadence` (section 9), and
with it the Rust toolchain every crate after it shares. The proposal
says why; the `cadence` spec says what the crate answers.

The clauses it satisfies: 59 and 60 (A.9), 83 to 85 (B.5), 86 for
cadence binding one clock, the profile C.5, scenario S13, and ruling 7
of section 10 (the machine's local time). It bears on 69, keeping no
state, through the clock seam.

The rules it relies on, all in `docs/architecture/core.md`, the only
page there (no area page exists, so none is touched):

- **core.33**: cadence reads a schedule, the last run and now and
  nothing else, in local time, caught up once. This change is that
  rule's crate.
- **core.1**: no state, no process. The one reading of the machine is
  its clock, which core.1's details allow (83).
- **core.2**: no tool of the family is named in the crate, its doc
  comments included. The host that calls cadence is "the caller".
- **core.37**: the code cites 59, 60 and 83 to 85 by number and
  restates none of them.
- **core.39** and **core.40**: the crate is a moon project of its own
  with one task per check, every new file type is claimed by a
  formatter, and the Rust pinned in two files has a check that fails
  when they differ.

Of the kit standard: kit:testing.1 and kit:testing.3 (layered moon
tasks declaring their inputs), kit:testing.5 (proved by the checks
alone), kit:toolchain.2 (the language toolchain pinned in
`.prototools`, read by `.moon/toolchains.yml` with
`versionFromPrototools`), kit:toolchain.5 (every tool through
`devenv shell --`).

## Goals / Non-Goals

**Goals:**

- One pure function that answers due or not from a schedule, an
  optional last run and a zoned now, and a clock seam around it.
- A five-field cron reader whose accepted language is exactly
  five-field cron, so a schedule that reads here reads the same in a
  crontab.
- The Rust toolchain in the tree, done once in the shape the family's
  first Rust repository set (tailrace), so the bolt's other crates add
  only their own lines.

**Non-Goals:**

- Keeping a last run, or deciding what "a thing" is: the caller keeps
  both (69). weir gives no store and no identifier for a thing.
- A tick, a timer or anything that waits: weir starts nothing (69).
- A `weir` command line for cadence, and the conformance pack's cases
  for 59 and 60 (see Open Questions).

## Decisions

### The answer is one pure function over three values

`is_due(schedule: &Schedule, last_run: Option<jiff::Timestamp>, now:
&jiff::Zoned) -> bool`. With no last run it answers true. Otherwise it
finds the first time the schedule names strictly after `last_run`, read
in `now`'s zone, and answers whether that time is at or before `now`.
One comparison against the first named time is what makes the catch-up
single (60): the answer is a yes, never a count, and a run at wake
moves `last_run` past every time missed.

`now` is a `jiff::Zoned` because the zone is part of now: the same
instant is a different local time in two zones, and two machines agree
only when given the same three values, zone included (59). `last_run`
is a `jiff::Timestamp`, an instant, because a run happened at one
moment whatever zone reads it later; the caller keeps it, in RFC 3339
as section 9 fixes.

Alternatives: `chrono::DateTime<Tz>` carries a zone only by type
parameter, and `chrono::Local` reads the process's zone inside every
computation, which would let the machine reach the answer outside the
clock. A count of missed runs was rejected: 60 asks for one catch-up,
and a count invites a caller to run per missed time.

### "Since" is strictly after the last run, at whole seconds

A named time equal to the last run has not come since it, so a run
started on its named minute is not due again within that minute. A
last run with a fraction of a second is compared as an instant: 10:00
counts after a last run at 09:59:59.700. The crate does not lean on
how the cron library steps past its start for this: it searches from
the last run truncated to its second and drops a found time not
strictly after the last run, and the spec's fraction scenario holds it.

### The clock is a trait of two methods

```rust
pub trait Clock {
    type Thing: ?Sized;
    fn now(&self) -> jiff::Zoned;
    fn last_run(&self, thing: &Self::Thing) -> Option<jiff::Timestamp>;
}
pub fn due<C: Clock>(schedule: &Schedule, thing: &C::Thing, clock: &C) -> bool;
```

B.5 is a seam with two operations, and 85 says cadence asks it for
those two alone; `due` calls each once and hands both to `is_due`. The
thing is the caller's type, so weir names no tool's notion of an
adapter or a job (core.2). weir supplies `machine_now() -> jiff::Zoned`,
`jiff::Zoned::now()`, as C.5's clock's `now`: the system clock in the
machine's local zone. A caller's clock returns it from `now` and its
own record from `last_run`. weir supplies no `last_run`: that is state,
and weir keeps none (69, core.1).

Alternative: one `MachineClock` struct with a `last_run` read from a
file was rejected as state in a crate (core.1, core.33).

### croner finds the named times; weir gates the language

`croner` 4 (`default-features = false, features = ["jiff"]`) parses a
pattern and finds the next occurrence after a `jiff::Zoned`
(`Cron::find_next_occurrence<T: CronDateTime>(&self, &T, inclusive)`,
`jiff::Zoned` implementing `CronDateTime`). Its DST rules are cron's:
a fixed time in a skipped hour runs at the first minute after the
skip, and a fixed time in a repeated hour runs at its first pass, which
is what the spec's daylight-saving requirement asks for. It moves in
real time, not wall-clock time, so the search never runs backward
across a fall-back.

croner accepts more than five-field cron: an optional seconds field, a
year, nicknames such as `@hourly`, and `L`, `W`, `#`, `?` and `+`, with
no switch to refuse them. So `Schedule::parse` reads the text itself
first: exactly five whitespace-separated fields; each a list of `*`, a
number, `a-b`, `*/n` or `a-b/n`; numbers in the field's range (minute
0 to 59, hour 0 to 23, day 1 to 31, month 1 to 12, weekday 0 to 7);
three-letter names, any case, in the month and weekday fields alone.
Anything else is refused with an error naming the field and the token.
Only then is the text handed to croner, built with
`CronParser::builder().seconds(Seconds::Optional)` (five fields read
with seconds at 0) and day-of-month and day-of-week left at their
default. A pattern croner still refuses is reported with croner's
reason under the same error type.

Day fields follow cron's rule: with both restricted, either matches;
with one `*`, the other alone decides. croner's README documents the
OR default but not how `*` in one field combines with a value in the
other, so the spec's `0 9 * * MON` scenario pins it. If croner
disagrees, the gate notes which day field is `*` and passes croner a
pattern whose meaning is the same under its rule (with the other day
field prefixed by `+` for AND), and the scenario holds that too.

Alternatives: the `cron` crate reads six or seven fields with seconds
and is chrono-only. A matcher of weir's own over jiff was rejected for
the occurrence search: the daylight-saving cases are where a new
implementation goes wrong, and croner has tested them.

### Errors

`Schedule::parse(&str) -> Result<Schedule, ScheduleError>`, where the
error names the field (minute, hour, day of month, month, day of week,
or the field count) and the token refused. `is_due` and `due` return a
plain `bool`. A schedule that parsed names a time within croner's
search limit unless it names a day that never comes (`0 0 31 2 *`);
when croner finds no next time, no named time has come, and the answer
is not due.

### Tests are fixed inputs, zones from a bundled database

Every scenario of the spec is a `cargo test` in the crate, with fixed
schedules, instants and IANA zones and no reading of the machine's
clock but in the machine's-now test. `jiff` takes its zone database
bundled (`tzdb-bundle-always`) as a dev-dependency feature, so a
host's `/usr/share/zoneinfo` cannot change a result (core.40's intent
for tools, held for data). The two tests that set the process's zone
(the two-machines scenario and the machine's-now scenario) run in an
integration test binary of their own, `tests/process_zone.rs`, each
spawning the test binary again with `TZ` set and reading its answer,
so no test shares an environment variable with another thread.

The recording clock of the two-calls scenario is a test double in the
crate's tests, counting calls to `now` and `last_run`.

### The toolchain, as tailrace set it

- `.prototools` pins `rust = "1.98.1"`, the family's pin today;
  `.moon/toolchains.yml` adds `rust: {versionFromPrototools: true}`.
- `rust-toolchain.toml` names the same channel with the `clippy` and
  `rustfmt` components and the `minimal` profile, so rustup in this
  tree never falls back to a host's default toolchain.
- `check-toolchain.sh` fails when `rust-toolchain.toml` and
  `.prototools` differ or when rustc, cargo, clippy or rustfmt report
  another version, run by a root `toolchain` task tagged `check-fast`
  with `cache: false`, since it reads tools outside the tree (core.40).
- `treefmt.nix` enables `rustfmt` (edition 2021), and `shellcheck` and
  `shfmt` for the check script; `devenv.nix` hands treefmt a `rustfmt`
  that activates the pinned toolchain first. `Cargo.toml` and
  `rust-toolchain.toml` are TOML, already taplo's; `Cargo.lock` is
  already excluded by `*.lock`.
- `Cargo.toml` adds `crates/weir-cadence` to `members`, `edition =
  "2021"` and `version = "0.0.0"` to `[workspace.package]`, and `jiff`
  and `croner` to `[workspace.dependencies]`.

The bolt's other crate units need the same toolchain. Whichever lands
first in the bolt brings it; a later one finds it there and adds only
its crate's lines, and crew names the overlap in the Touches blocks.

### The crate's moon project

`crates/weir-cadence/moon.yml`, project `cadence` in
`.moon/workspace.yml`, `language: 'rust'`, four tasks tagged
`check-fast`, each run from the workspace root:

| task | command |
|---|---|
| `format` | `cargo fmt -p weir-cadence --check` |
| `lint` | `cargo clippy -p weir-cadence --all-targets -- -D warnings` |
| `typecheck` | `cargo check -p weir-cadence --all-targets` |
| `test` | `cargo test -p weir-cadence` |

Each declares its inputs: `/crates/weir-cadence/**/*`, `/Cargo.toml`,
`/Cargo.lock`, `/rust-toolchain.toml`, `/devenv.nix`, `/devenv.lock`
and `/.prototools` (kit:testing.3).

## Risks / Trade-offs

- [croner's day-field rule differs from cron's when one field is `*`]
  → the `0 9 * * MON` scenario catches it, and the gate rewrites the
  pattern as the day-field decision says.
- [croner 4's `jiff` feature lags `jiff`'s releases] → the workspace
  takes the `jiff` version croner's feature requires, and the lockfile
  holds both together.
- [The siblings' toolchain lines conflict at merge] → the lines are the
  same as tailrace's, so a second unit's merge is a no-op on them.
- [A schedule naming a day that never comes reads as never due] →
  accepted: it is valid five-field cron, and cron itself never runs it.

## Open Questions

- The conformance pack has no case shape for cadence: C.7 describes
  host cases (lines in, records out) and adapter checks, while 67 and
  core.35 ask a case to cite every Part A clause, 59 and 60 among them.
  Until the design gives cadence a case shape, the crate's own tests
  prove 59 and 60, and the pack's check will report them uncited once
  it exists. This change does not need the answer.
