# Tasks

## 1. The Rust toolchain

- [x] 1.1 Pin Rust once: add `rust = "1.98.1"` to `.prototools`, a `rust:` block with `versionFromPrototools: true` to `.moon/toolchains.yml`, and `rust-toolchain.toml` naming channel `1.98.1` with `clippy` and `rustfmt` on the minimal profile, each with a comment saying what it pins and why. True when `devenv shell -- moon run root:format` passes with the new TOML files formatted by taplo.
- [x] 1.2 Add `check-toolchain.sh`, which fails naming the difference when `rust-toolchain.toml`'s channel is not `.prototools`' pin or when `rustc`, `cargo`, `clippy` or `rustfmt` on the path is another build, and a root `toolchain` task in `moon.yml` that runs it, tagged `check-fast`, with `cache: false` (core.40). True when `devenv shell -- moon run root:toolchain` passes.
- [x] 1.3 Have treefmt claim the new file types: in `treefmt.nix`, `rustfmt` (edition 2021) for `*.rs`, and `shellcheck` then `shfmt` (indent 2) for `*.sh`; in `devenv.nix`, a `rustfmt` wrapper that activates the pinned toolchain, set as treefmt's `programs.rustfmt.package` (core.39). True when `devenv shell -- moon run root:format` passes with `check-toolchain.sh` and a `.rs` file in the tree.

## 2. The kinds crate

- [x] 2.1 Add `crates/weir-kinds` as a workspace member, with `edition = "2021"` and the `toml` 0.9 dependency in `[workspace.package]` and `[workspace.dependencies]`, and as moon project `weir-kinds` in `.moon/workspace.yml` with `language: 'rust'` and tasks `format`, `lint`, `typecheck` and `test` as the design names them, each tagged `check-fast` with its inputs, `/devenv.nix`, `/devenv.lock`, `/.prototools` and `/rust-toolchain.toml` among them (core.39, kit:testing.3). True when `devenv shell -- moon run weir-kinds:typecheck` passes on the empty crate.
- [x] 2.2 Read a declaration into its kinds and stages: `read_kinds`, `Declaration`, `Kind`, `Stage`, `End`, `Deliverable` and the six spellings, the format's defaults, `quiet` read as a `Duration`, and every unnamed key kept in `sections` (51 to 54, 56, C.4). True when `devenv shell -- moon run weir-kinds:test` passes the tests for "A declaration reads into its kinds", "A stage reads with its name, prompt and end", "Each spelling reads as its form" and "A section the reader does not know never fails the read", reading from fixtures under `tests/fixtures/` that name no tool of the family (core.2).
- [x] 2.3 Check a declaration whole: `Refusal` with every `Problem { field, reason }`, field paths with stages indexed from 0, every check the design lists, and a parse failure placed by line and column (57, 81). True when `devenv shell -- moon run weir-kinds:test` passes the tests for every scenario of "A deliverable is one of six spellings" that refuses and of "A declaration is checked whole and refused naming the field", S12 among them.
- [x] 2.4 Read an elaboration kind's section: `read_elaboration`, `Elaboration` and `Ending`, checked whole and refused naming the field, while `read_kinds` never looks inside it (55, 56). True when `devenv shell -- moon run weir-kinds:test` passes the tests for "An elaboration kind's section is read only by whoever runs elaborations", S11 among them.
- [x] 2.5 Prove the same declaration reads the same: `fix-and-change.toml` holds `fix@1` and `change@1` as S10 states, each kind reads with its stages and deliverables, and a second read equals the first (58, 82). True when `devenv shell -- moon run weir-kinds:test` passes "Two kinds read the same twice".
- [x] 2.6 Document the crate's public items, each with the clause it reads, and hold it to clippy. True when `devenv shell -- moon run weir-kinds:lint` and `devenv shell -- moon run weir-kinds:format` pass.

## 3. weir check

- [ ] 3.1 Add `crates/weir`, package `weir` with a `[[bin]]` named `weir`, depending on `weir-kinds` and on `clap` 4 with `derive` through `[workspace.dependencies]`, as a workspace member and as moon project `weir` with the same four tasks, its inputs taking `crates/weir-kinds/**/*` too. True when `devenv shell -- moon run weir:typecheck` passes.
- [ ] 3.2 Implement `weir check <path>` as the design states: the kinds and every elaboration section read, the kinds listed and exit 0 on success, each problem as `<field>: <reason>` on standard error and exit 1 on refusal, an unreadable file named on standard error with exit 2, and the file opened read-only. True when `devenv shell -- moon run weir:test` passes the tests for "weir check reads a declaration", each running the built binary against the crate's fixtures and comparing a fixture's bytes before and after.
- [ ] 3.3 Hold the command line to clippy and rustfmt. True when `devenv shell -- moon run weir:lint` and `devenv shell -- moon run weir:format` pass.

## 4. The projects this change touches

- [ ] 4.1 Run the checks of the projects this change touches. True when `devenv shell -- moon run root:format root:prose root:toolchain weir-kinds:format weir-kinds:lint weir-kinds:typecheck weir-kinds:test weir:format weir:lint weir:typecheck weir:test` passes.
