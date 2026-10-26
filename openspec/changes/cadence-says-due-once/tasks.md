# Tasks

## 1. Rust in the tree

Where a unit of this bolt that landed first already brought a line of
this group, the line holds as it is; add only what is missing.

- [x] 1.1 `.prototools` pins `rust = "1.98.1"`, `.moon/toolchains.yml` reads it with `rust: {versionFromPrototools: true}`, and `rust-toolchain.toml` names channel `1.98.1` with components `clippy` and `rustfmt` and profile `minimal`; proved by `devenv shell -- moon run root:toolchain` passing (task 1.2).
- [x] 1.2 `check-toolchain.sh` fails when `rust-toolchain.toml`'s channel differs from `.prototools`' rust pin or when rustc, cargo, clippy or rustfmt is another build, and the root `moon.yml` runs it as task `toolchain`, tagged `check-fast`, `cache: false`, from the workspace root (core.40); proved by `devenv shell -- moon run root:toolchain`.
- [x] 1.3 `treefmt.nix` enables `rustfmt` (edition 2021) and `shellcheck` with `shfmt` (indent 2) in one shell pipeline, and `devenv.nix` gives treefmt a `rustfmt` that activates the pinned toolchain first; every tracked file is claimed or excluded (core.39); proved by `devenv shell -- moon run root:format`.
- [x] 1.4 `Cargo.toml` sets `edition = "2021"` and `version = "0.0.0"` in `[workspace.package]`, lists `jiff` (0.2) and `croner` (4, `default-features = false`, `features = ["jiff"]`) in `[workspace.dependencies]`, and adds `crates/weir-cadence` to `members` with the comment's list kept; `Cargo.lock` is committed; proved by `devenv shell -- moon run cadence:typecheck` (task 2.1).

## 2. The crate and its project

- [x] 2.1 `crates/weir-cadence` exists as a library crate (`Cargo.toml` taking the workspace's version, edition, license and `[lints]`, a description citing 59, 60 and 83 to 85) and is project `cadence` in `.moon/workspace.yml`, its `moon.yml` `language: 'rust'` with tasks `format`, `lint`, `typecheck` and `test` as the design's table gives them, each tagged `check-fast`, run from the workspace root and declaring `/crates/weir-cadence/**/*`, `/Cargo.toml`, `/Cargo.lock`, `/rust-toolchain.toml`, `/devenv.nix`, `/devenv.lock` and `/.prototools` as inputs (core.39, kit:testing.3); proved by `devenv shell -- moon run cadence:format cadence:lint cadence:typecheck cadence:test`.

## 3. Reading a schedule

- [ ] 3.1 `Schedule::parse` accepts five-field cron, `*`, numbers, ranges, lists, `*/n` and `a-b/n` steps, and month and weekday names in any case, and the parsed schedule names the times the spec's "Lists, ranges, steps and names", "Both day fields restricted" and "Sunday as 0 and as 7" scenarios give; proved by `devenv shell -- moon run cadence:test`.
- [ ] 3.2 `Schedule::parse` refuses six fields, four fields, a nickname, `L`, `W`, `#`, `?` and `+`, an unknown name, a name outside its field and a value out of range, each with a `ScheduleError` naming the field and the token, as the spec's refusal scenarios give; proved by `devenv shell -- moon run cadence:test`.
- [ ] 3.3 With one day field `*`, the other alone decides: the spec's "A weekday with day of month unrestricted" scenario passes, croner's pattern rewritten by the gate if croner reads it otherwise; proved by `devenv shell -- moon run cadence:test`.

## 4. Saying due

- [ ] 4.1 `is_due(schedule, last_run, now)` answers due for no last run, and due exactly when a named time lies strictly after the last run and at or before now, read in now's zone; the spec's never-run, named-time, fraction, later-than-now and two-zones scenarios pass; proved by `devenv shell -- moon run cadence:test`.
- [ ] 4.2 A run missed while the machine slept is caught up once: the spec's "Three hourly times slept through" scenario (S13) passes; proved by `devenv shell -- moon run cadence:test`.
- [ ] 4.3 Daylight-saving days name each fixed time once: the spec's skipped-time and repeated-time scenarios in America/New_York pass, zones taken from jiff's bundled database (`tzdb-bundle-always`, a dev-dependency feature); proved by `devenv shell -- moon run cadence:test`.
- [ ] 4.4 The `Clock` trait has `now` and `last_run` alone, `due(schedule, thing, clock)` asks each once and answers as `is_due`, and `machine_now()` gives the system clock in the machine's local zone: the recording-clock scenario passes, and `tests/process_zone.rs` proves the two-machines scenario (Asia/Tokyo and America/Los_Angeles) and the machine's-now scenario (Australia/Adelaide) by running the test binary again with `TZ` set; proved by `devenv shell -- moon run cadence:test`.
- [ ] 4.5 Every public item has a doc comment citing the clause it serves by number (59, 60, 83 to 85) without restating it (core.37), enforced by `#![deny(missing_docs)]`; and `tests/names.rs` fails when a file under the crate's `src/` holds any string core.2 rules out (core.2); proved by `devenv shell -- moon run cadence:lint cadence:test`.

## 5. The checks

- [ ] 5.1 The checks of the projects this change touches pass: `devenv shell -- moon run root:format root:prose root:toolchain cadence:format cadence:lint cadence:typecheck cadence:test`.
