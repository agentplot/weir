# Tasks

## 1. The gate

- [x] 1.1 Give the root project `specs`, a `check-fast` task running `openspec validate --specs --strict --no-interactive`, with `OPENSPEC_TELEMETRY=0` and `OPENSPEC_NO_UPDATE_CHECK=1`, its inputs `/openspec/specs/**/*`, `/openspec/config.yaml`, `/devenv.nix` and `/devenv.lock`, as sluice, flume and boomdrive spell it (design 1; core.39, core.40). `CLAUDE.md`'s table of checks and `.moon/workspace.yml`'s comment name it. True when `devenv shell -- moon run :#check-fast` runs `root:specs` among its tasks, and when `devenv shell -- moon run root:specs` passes once 2.1's specs are archived.

## 2. The specs

- [x] 2.1 Split each of the 23 requirements `openspec validate --strict` refuses as over 500 characters, in this change's delta specs, as design 2 and 3 say. Each part is 500 characters or fewer, and each requirement split keeps every scenario it had. True when, with this change archived, `devenv shell -- moon run root:specs` passes, with every one of the 23 specs read strictly.

## 3. The checks

- [x] 3.1 Run the checks of `root`, the one project this change touches. True when `devenv shell -- moon run root:format root:prose root:hooks root:toolchain root:openspec-claude root:specs` passes.
