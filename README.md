# weir

**Designed, not built.** The design is settled in
[boomdrivedev/blueprints](https://github.com/boomdrivedev/blueprints); no code
exists yet.

weir is the shared code of the agentplot tool family. It defines, once,
what every open tool shares: the capture contract, the record formats,
the shared-branch write protocol, the kind format, cadence and the
built-in adapters. millpond, headwater, eddy, sounding and flume build on
its crates, and Boomdrive builds on the same crates later. A conformance
pack holds each host to the contract: contract lines go in, and the
records a host must write from them come out.

A weir measures and regulates a stream's flow. The contract and its
conformance pack do that for the family's captures.

- **The design:** `design/capture/design.md` in boomdrivedev/blueprints,
  and its plan, `plan.md` beside it.
- **The statement**, weir's invariants as numbered clauses, comes next,
  as `design/weir/requirements.md` in boomdrivedev/blueprints.

## The crates

| crate | what |
|---|---|
| `weir-capture` | The capture contract: its types and key rules, JSON in flight, recutils at rest, a sha256 content hash and the excerpt cut. |
| `weir-signals` | The signal, move and item records, and the inbox's layout. |
| `weir-adapters` | The folder drop, the meeting file and the signals-folder import. |
| `weir-branch` | The one write protocol for a shared branch. |
| `weir-bounded` | A call to a remote run to the family's limit for a try, or to its caller's halt, ended with everything it started. |
| `weir-kinds` | The one kind format, with an elaboration kind's section, a kind's packages, their content hash and locks. |
| `weir-cadence` | Five-field cron, caught up once after a missed run. |
| `weir-delivery` | One form for a message a tool owes a reader, owed until the reader confirms it by its key: its records, their spellings and the calls over a deliverer's own store. |
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
