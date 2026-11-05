# Tasks

Each group is one session's work for an agent that reads this change
(proposal, design, specs) and its group, and builds on the groups before
it. The statement is read at blueprints a11f6d5, with C.4, C.7 and S17
as amended at 361a599 (branch `design/weir-a-kinds-name-and-a-package-without-links`);
record 0004 and core.30, core.42 and core.43 as amended are already on
this branch. Tests reach no file outside a temporary folder of their
own.

## 1. A name, an identity and a stage's standing

- [x] 1.1 Give `weir-kinds` one public check of a name, one or more ASCII letters, digits, `_` and `-`, saying why a text is none, and have `read_kinds` refuse a kind's table or a table of settings under any other key, naming `kinds."<key>"` and reading nothing under it (design 1; 57, 88, C.4; core.42; record 0004). True when `devenv shell -- moon run kinds:test` passes tests holding the kind-identity spec's *A source's name and a kind's name are letters, digits, _ and -* and the kinds spec's *A kind's key is a name*, and every existing test unchanged.
- [x] 1.2 Add `Identity`, ordered by source, name and version as a number, and `NamedKind`, its source and version each optional, each parsed from text and spelled back as read, the version decimal digits of at least 1 with no leading zero, with the constants `repo` and `launch` (design 2; 88, C.4; core.2, core.42). True when `devenv shell -- moon run kinds:test` passes tests holding every scenario of the kind-identity spec's *An identity reads into its parts and spells back* and *A kind named with its source or version left out reads into its parts*.
- [x] 1.3 Read a stage's `standing` into `Stage::standing`, an `Option<String>` returned as written, `standing` among `STAGE_FIELDS`, refused naming the field when not a non-empty string (design 7; 53, 57, C.4). True when `devenv shell -- moon run kinds:test` passes tests holding the kinds spec's *A stage reads its standing instructions* and *Standing instructions of the wrong shape*.
- [x] 1.4 Run the checks of the projects this group touched. True when `devenv shell -- moon run kinds:format kinds:lint kinds:typecheck kinds:test weir:test` passes.

## 2. A package and its content hash

- [x] 2.1 Have `weir-kinds` take `weir-capture` as a dependency and `tempfile` for its tests, have `crates/weir-kinds/moon.yml` depend on `capture` with `/crates/weir-capture/**/*` among every task's inputs, and have the crate's `description` and the workspace `Cargo.toml`'s comment name its identities, packages, content hash and locks (design 5; kit:testing.3; core.39). True when `devenv shell -- moon run kinds:typecheck kinds:lint` passes.
- [x] 2.2 Add `ContentHash`, parsed from and spelled as `sha256:` and 64 lowercase hex digits, and the one function that computes it over a package's files in byte order of their paths, streamed through `weir_capture::content_hash_reader` (design 5; 90, C.4; core.44). True when `devenv shell -- moon run kinds:test` passes tests holding the kind-packages spec's *A known package's hash* and *One byte changed (S18)* over paths and bytes, and a `ContentHash` refused for upper-case hex, a missing `sha256:` or a length other than 64.
- [x] 2.3 Add `read_package`, given a package's files as paths and bytes and the identity it is held at, giving `Package` (its identity, kind, content hash and the declaration's unread sections) or the package refused whole with every problem, checked in design 4's order (design 3, 4; 89, 92, C.4; core.43). True when `devenv shell -- moon run kinds:test` passes tests holding every scenario of the kind-packages spec's *A package declares exactly one kind and names no other*, *Every part a package names is a regular file inside it* and *A package held at another identity is refused naming both*, its *A package that reads*, *A section beside the kind* and *A path given that is not within a package*, and its Markdown link scenario.
- [x] 2.4 Add `read_package_dir`, given a folder and the identity it is held at, walking the folder without following any link into the files `read_package` reads, each link refused naming its path within the package, and an entry neither a file, a folder nor a link not hashed (design 3; 89, 90, 92, C.4 as amended; record 0004). True when `devenv shell -- moon run kinds:test` passes tests over temporary folders holding the kind-packages spec's *A package read twice*, *The same files on disk and in memory*, *A folder reached through a link*, *Two machines, two folders (S17)* and every scenario of *A package holds no symbolic link*.
- [x] 2.5 Name `read_package`, `read_package_dir`, `ContentHash`, `Identity` and `NamedKind` in `crates/weir-kinds/src/lib.rs`'s module doc with the clauses each serves, and run the checks of the projects this group touched. True when `devenv shell -- moon run kinds:format kinds:lint kinds:typecheck kinds:test weir:test` passes.

## 3. A lock

- [x] 3.1 Add `Lock`, its entries by `Identity`, each with its `ContentHash` and unread keys, and the lock's own unread keys, and `read_lock`, refusing whole as design 6 lists, an identity recorded again named at its second entry with its first (design 6; 91, 93, C.4; core.30, core.45). True when `devenv shell -- moon run kinds:test` passes tests holding every scenario of the kind-locks spec's *read-lock reads every identity a lock records with its hash* and *A lock is refused naming the field*.
- [x] 3.2 Add `spell_lock`, giving the lock's unread keys, then one `[[kinds]]` table per entry in identity order, `kind` then `hash`, then the entry's unread keys (design 6; C.4 as amended; core.30). True when `devenv shell -- moon run kinds:test` passes tests holding the kind-locks spec's *A lock spelled and read back* and *The same lock spells the same*.
- [ ] 3.3 Add `Lock::check`, given an identity and a hash, answering recorded or not recorded, or a mismatch whose text names the kind, the lock's hash and the hash given (design 6; 91, S18; core.45). True when `devenv shell -- moon run kinds:test` passes tests holding the kind-locks spec's *A package changed by a byte (S18)* and *Recorded and not recorded*.
- [ ] 3.4 Name `Lock`, `read_lock`, `spell_lock` and `Lock::check` in the module doc, and run the checks of the projects this group touched. True when `devenv shell -- moon run kinds:format kinds:lint kinds:typecheck kinds:test weir:test` passes.

## 4. The pack

- [ ] 4.1 Have `crates/weir-kinds/tests/pack.rs` read a kinds case of exactly one of `declaration`, `package` and `lock`, as design 8 gives them, failing naming the case on a field the format does not name or a case of more than one form; lay each package out in two temporary folders at different depths with its links, read both through `read_package_dir`, check it against its `lock` where it names one, and fail naming the case where the two differ or either differs from the case (design 8; 65, 87, C.7 as amended; core.35, core.36). True when `devenv shell -- moon run kinds:test` passes with every existing case answered as before, and the kinds spec's *A misspelt field of the case* and *A case of more than one form* held by tests.
- [ ] 4.2 Add under `pack/kinds/` the cases the kinds spec's *The cases the statement gives* lists, each citing its clauses, among them 88 to 93 and S10, S17 and S18 in its comment, and extend the test that the kinds cases cite 51 to 58 to cite 88 to 93 too (88 to 93, C.7; core.35). True when `devenv shell -- moon run kinds:test` passes with each new case answered as it says.
- [ ] 4.3 Move `pack/clauses.toml` to the statement at blueprints a11f6d5: `commit = "a11f6d5"`, A.8 holding 51 to 58 and 88 to 91, and B.4 81, 82, 92 and 93 (design 9; 67; core.35). True when `devenv shell -- moon run pack:test` passes, the pack's own check counting 88 to 91 and finding each cited.

## 5. The projects this change touches

- [ ] 5.1 Run the checks of the projects this change touches. True when `devenv shell -- moon run kinds:format kinds:lint kinds:typecheck kinds:test pack:format pack:lint pack:typecheck pack:test weir:test root:format` passes.
