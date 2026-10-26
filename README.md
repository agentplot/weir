# weir

**Designed, not built.** The design is settled in
[agentplot/blueprints](https://github.com/agentplot/blueprints); no code
exists yet.

weir is the shared code of the agentplot tool family. It defines, once,
what every open tool shares: the capture contract, the record formats,
the shared-branch write protocol, the kind format, cadence and the
built-in adapters. millpond, headwater, eddy, sounding and flume build on
its crates, and Flywheel builds on the same crates later. A conformance
pack holds each host to the contract: contract lines go in, and the
records a host must write from them come out.

A weir measures and regulates a stream's flow. The contract and its
conformance pack do that for the family's captures.

- **The design:** `design/capture/design.md` in agentplot/blueprints,
  and its plan, `plan.md` beside it.
- **The statement**, weir's invariants as numbered clauses, comes next,
  as `design/weir/requirements.md` in agentplot/blueprints.

## The crates

| crate | what |
|---|---|
| `weir-capture` | The capture contract: its types and key rules, JSON in flight, recutils at rest, a sha256 content hash and the excerpt cut. |
| `weir-signals` | The signal, move and item records, and the inbox's layout. |
| `weir-adapters` | The folder drop, the meeting file and the signals-folder import. |
| `weir-branch` | The one write protocol for a shared branch. |
| `weir-kinds` | The one kind format, with an elaboration kind's section. |
| `weir-cadence` | Five-field cron, caught up once after a missed run. |
| the conformance pack | Contract lines in, the records a host must write out. |

Each record format is named `weir-<record>/<major>`. Every crate is
public, and none keeps state or knows which tool or host uses it.

## This repository

| path | what |
|---|---|
| `Cargo.toml` | the Rust workspace, with no crates yet |
| `devenv.nix` | the shell: moon through proto, and Vale |
| `.config/kit.toml` | the kit standard's profile this repository follows |

```sh
devenv shell -- check-fast   # format, prose and the commit hooks, through moon
devenv shell -- prose        # the prose, through Vale
```

## License

MIT
