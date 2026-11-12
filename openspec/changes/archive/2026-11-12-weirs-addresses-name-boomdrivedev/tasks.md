# Tasks

## 1. The addresses

- [x] 1.1 Have conformance-pack's and remote-limit's requirements in this change's delta specs name boomdrivedev/blueprints and boomdrivedev/sluice, each keeping its scenario (design 1; record 0009). True when, with this change archived, `devenv shell -- moon run root:specs` passes.
- [x] 1.2 Have `pack/clauses.toml` name its statement `boomdrivedev/blueprints:design/weir/requirements.md` at `006fc34`, and weir-pack's tests read it so (design 1; 67; core.37). True when `devenv shell -- moon run pack:test` passes `the_index_holds_every_clause_under_its_section`.
- [x] 1.3 Have weir-signals' tests and the pack's inbox cases name a send's target and outcome at boomdrivedev/weir, each case's starting records and tip together (design 3; core.35, core.46). True when `devenv shell -- moon run signals:test` and `devenv shell -- moon run pack:test` pass.
- [x] 1.4 Have weir-branch's, weir-signals' and weir-cadence's names tests read a ruled-out name as a whole word, each with `a_tool_is_named_as_a_whole_word`, and the crates' module comments cite boomdrivedev/blueprints (design 2; core.2). True when `devenv shell -- moon run branch:test`, `signals:test`, `cadence:test`, `adapters:test`, `bounded:test`, `capture:test` and `pack:test` pass.
- [x] 1.5 Have the README, CLAUDE.md, the OpenSpec config, the comments of `Cargo.toml`, `devenv.nix`, `.gitignore` and `.vale.ini`, the Vale rules' links, the rules' narrative and core.53's sources, and the records' citations name the moved tools at boomdrivedev, as record 0009 says. True when `devenv shell -- moon run root:format root:prose` passes.

## 2. The checks

- [x] 2.1 Run the checks of the projects this change touches. True when `devenv shell -- moon run root:format root:prose root:specs pack:test signals:test branch:test cadence:test adapters:test bounded:test capture:test` passes.
