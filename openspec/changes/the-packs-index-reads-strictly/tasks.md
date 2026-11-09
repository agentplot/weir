# Tasks

## 1. The spec

- [ ] 1.1 Split `conformance-pack`'s *The pack keeps an index of the statement's clauses* in this change's delta spec, as design 1 says, each part 500 characters or fewer and the requirement split keeping its scenario. True when, with this change archived, `devenv shell -- moon run root:specs` passes, with every one of weir's specs read strictly.

## 2. The checks

- [ ] 2.1 Run the checks of `root`, the one project this change touches. True when `devenv shell -- moon run root:format root:prose root:hooks root:toolchain root:openspec-claude root:specs` passes.
