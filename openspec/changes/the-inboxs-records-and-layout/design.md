# Design

## Context

This unit is built on the capture contract as it stands on its branch:
`weir-capture` holds the key rule (with the layout's roots and file
names it refuses), the recutils reader and writer (`rec`), the capture
record `weir-capture/1`, the cut and its `LeftOut`, the nature
(`Nature`), and `take`, which hands a host's inbox a `Taken`: the
capture and, where its offer carried an own excerpt, the `OwnSignal`
whose record that change left to this crate. No other crate exists on
this branch; the kind format and cadence come in their own units.

The statement (`agentplot/blueprints:design/weir/requirements.md` at
main) fixes what this crate must do: signals and readings (25 to 28),
moves and batches (29 to 33), items, briefs and sends (34 to 36), the
inbox's records and layout (37 to 43), the import operation (63, 75a),
the records profile (C.2) and scenarios S6, S8 and S9. Section 9 names
the crate, `weir-signals`, for the signal, move, batch, item, brief and
send records and the inbox's layout. madswan-design's ruling of
2026-10-26 (blueprints main f4aabd9) gives this unit the import
operation and the built-in adapters unit the import line's type in
`weir-capture` and the reader of crew's folders.

The rules it relies on, all in `docs/architecture/core.md`, the only
page of `docs/architecture/` (no area page exists, so none is touched):
core.1 (no state, no process, every write one a caller asked for),
core.2 (no tool of the family named), core.4 (no move chosen, no brief
composed), core.8 (ages read the event time), core.10 (held keys),
core.13 (capture and own signal in one write), core.16 and core.17
(signals and readings), core.18 and core.19 (moves and batches),
core.20 (items, weight, briefs, sends), core.21 (nothing changes but
the brief), core.22 (formats and majors), core.23 (the layout),
core.24 (recutils at rest, `recfix`), core.25 (imports), core.26 (every
check on the tip it lands on), core.37 (clauses cited, never restated)
and core.39 (the crate a moon project, one task per check). Of the kit
standard: kit:testing.1 and kit:testing.3 (layered tasks declaring
their inputs) and kit:testing.5 (proved by the checks alone).

## Goals / Non-Goals

**Goals:**

- One crate that reads an inbox whole from the files of one tip, and
  forms every write an inbox takes, each checked against that read, so
  that the write protocol can form it again on a new tip and every
  check is made again there (46).
- Every record of C.2 written and read in recutils, and the brief, each
  naming its format and major.

**Non-Goals:**

- Reading or moving a shared branch: `weir-branch`, a later bolt (F4),
  reads the tip, hands its files here, and lands the write; this crate
  touches no repository and runs no `recfix` (47; core.1). Its tests run
  `recfix` over every file it forms.
- The import line's type and its JSON, and reading crew's folders into
  import lines: the built-in adapters unit's (ruling f4aabd9). Mapping
  crew's `new-territory` to a join is the reader of crew's folders'
  (record 0002, ruling 3; core.2, core.25); here any word not of the
  six arrives unmoved.
- Choosing a move, reading a capture into signals, writing a brief's
  text, sending or landing: the callers' (70; core.4).
- A commit message for a write (50): the write protocol's.

## Decisions

### Modules

`layout` (the paths of C.2 and the reading of a path into what lies
there), `format` (`weir-<record>/<major>` read and written), `signal`,
`moves` (moves and batches), `item` (items, briefs, sends), `inbox`
(the inbox read at one tip, its queries, the check of a write),
`write` (the write and the operations that form it) and `import`.
Records reuse `weir_capture::rec` and `weir_capture::Capture`; a
signal's nature is `weir_capture::Nature`; nothing of the capture crate
is changed.

### The inbox is a value read from one tip's files

`InboxAt::read(files)` takes every `(path, text)` of one tip and reads
those under `inbox/` (paths outside it are passed over), giving the
inbox or an `InboxError` naming the path and, where there is one, the
record and field (43; core.23). The name keeps it apart from
`weir_capture::Inbox`, the trait a host implements for `take`.

Reading checks what one tip alone can show:

- every path is one the layout places: `captures/<segments>/capture.rec`
  or `signals.rec` with segments the key rule takes,
  `moves/moves.rec`, `moves/batches/<batch>.rec`,
  `items/<name>/item.rec`, `brief.md` or `sends.rec`;
- each `capture.rec` holds one `weir-capture/1` record whose key is its
  folder's, and each `signals.rec` lies beside one and numbers its
  signals `<key>#1` up in order;
- `moves.rec` holds `weir-move/1` records only, each naming a signal
  the inbox holds, of the six words with its consequence; a signal's
  first move names no `replaces`, and each later one names, as
  `replaces`, the number of the move standing before it; a move naming
  a batch names one that stands and holds that move;
- each batch file opens with its `weir-batch/1` proposal, naming its
  file's batch, then its moves, each naming the batch, and at most one
  decision, last;
- each item folder holds one `weir-item/1` record naming its folder;
  each join names an item the inbox holds and each attach a sent one;
  each send's outcome record follows a send it names.

Every read record's format names the record its place holds; a major
above 1 is refused as later than the reader reads (40; core.22).
Rendering and reading each record is a pair of functions per record,
`to_record` and `from_record`, as `Capture` has.

Alternative: a trait over a repository that the crate reads lazily.
Rejected: a lazy read could mix two tips (43), and a file list is
what the write protocol has in hand at a tip.

### Every operation forms a write and checks it at the same tip

A `Write` is the full new text of each file it adds or replaces, keyed
by path: an append is the tip's text with records added after it (47).
Each operation is a method of `InboxAt` taking what the caller hands
(who, when, the records' content) and giving `Result<Write, Refused>`,
where `Refused` names each thing not taken and why:

| operation | forms | refuses when |
|---|---|---|
| `take(&Taken)` | `capture.rec`, and `signals.rec` with the own signal | the key is held (14) |
| `read_capture(key, signals, raw)` | `signals.rec` | the capture is not held, has an own signal or a reading, or any signal lacks a field, has another nature or an excerpt standing in no raw file (26, 28) |
| `make_move(move)` | a record appended to `moves.rec`, and `item.rec` for a join's new item | 29, 30 as the spec says |
| `propose_batch(batch, by, at, moves)` | the batch's file | the name was had or holds `/`, any move is invalid, or two name one signal (32) |
| `decide_batch(batch, stands, by, at)` | the decision appended to the batch's file and, when it stands, its moves to `moves.rec`, with any join's new item | the batch is decided or any move can no longer be taken (32) |
| `propose_item(name, conclusion, by, at)` | `item.rec` | the name was had or is not a name (35) |
| `write_brief(item, text)` | `brief.md` | the item is not held (36) |
| `send(items, target, sent_at)` | a send appended to each item's `sends.rec` | an item is not held or has no brief (36) |
| `settle_send(item, target, sent_at, outcome)` | an outcome appended to `sends.rec` | no such send, or it is settled (36, 41) |
| `import(lines, machine, imported_at)` | every record of the captures taken | see *The import operation* |

Every operation ends by handing its write to `InboxAt::check`, which
any caller may also call with a write formed elsewhere: it lays the
write over the tip's files, reads the inbox that would result (every
rule of the read above), and refuses a write that replaces a
`capture.rec`, `signals.rec` or `item.rec` the tip holds, or whose new
text of `moves.rec`, a batch's file or `sends.rec` does not begin with
the tip's text at a record's end (41; core.21, core.24). It gives the
inbox after the write, so a caller can make several in turn.

The write protocol replays a refused push by calling the same operation
on the inbox read at the new tip, so every check is made there (44, 46;
core.26). Alternative: a write that carries its own preconditions,
replayed without the operation. Rejected: a precondition list is a
second statement of the checks, and could drift from them.

### Spellings C.2 leaves open

- **`weir-signal/1`**: `format`, `signal`, `kind`, `asserted_by`,
  `assertion`, `subjects` and `argues_with` (one field per entry, none
  for an empty list, as `rec::Record::push_list`), `excerpt`,
  `position`, and `cut` only on a cut own signal, spelled `<n> lines and
  <b> bytes` after C.1's closing line.
- **A reading of no signals**: a `signals.rec` holding the one comment
  line `# weir-signal/1: read, no signals`, which `recfix` accepts and
  the reader skips; its presence is what makes the capture read. A
  capture's `unread` field is as it was written (41); a capture is
  unread when it has no `signals.rec`.
- **`weir-move/1`**: `format`, `signal`, `move`, `target`, `by`, `at`,
  `batch` where it stood in one, `replaces` on a replacement. A move has
  no identity field: `replaces: <n>` names the replaced move by its
  number among its signal's moves in `moves.rec`, counting from 1, which
  is fixed once written since the file only grows. The words are spelled
  `drop`, `answered`, `challenge`, `join`, `attach`, `route`; every
  consequence is `target`, non-empty, and for a join or an attach an
  item's name.
- **`weir-batch/1`**: the proposal, `format`, `batch`, `proposed_by`,
  `proposed_at`; the decision, appended later, `format`, `batch`,
  `stands` (`yes` or `no`, recutils' booleans as `unread` is),
  `decided_by`, `decided_at`. A batch's file holds its proposal, its
  moves as `weir-move/1` records, and its decision, so a proposed batch
  is written whole and only grows (41). When it stands, the same move
  records are appended to `moves.rec`.
- **`weir-item/1`**: `format`, `item`, `made_by` (`join` or
  `conclusion`), `conclusion` where one made it, `named_by`, `named_at`.
  It holds no list of signals: the item's signals are read from the
  standing moves (34; core.20).
- **The brief**: `brief.md`'s first line is
  `<!-- format: weir-brief/1 -->`, so a brief names its format (40)
  without showing it to the person who reads the rendered brief; the
  text handed follows on the next line, unchanged. An item with no brief
  has no `brief.md`.
- **`weir-send/1`**: a send is `format`, `item`, `target` (the caller's
  text: a repository and kind, a planner or an inquiry), `brief`, as the
  brief stood, `sent_at`, and `outcome: open`. An outcome added beside
  it is a `weir-send/1` record with `item`, `target`, `sent_at` naming
  the send, and `outcome`, and no `brief`; a send is the record with a
  `brief`. Outcomes are spelled `open`, `abandoned`, `landing <what
  landed>` and `conclusion <the conclusion>`. Two sends of one item to
  one target at one time are refused, so a target and a time name one
  send.
- Times (`at`, `proposed_at`, `decided_at`, `named_at`, `sent_at`) are
  handed by the caller and checked as RFC 3339 with an offset, and kept
  as handed (core.1, core.14).

### The layout's names agree with the key rule

The layout module holds the roots and file names C.2 places. The capture
crate's key rule holds its own copy, private to it. Rather than change
that crate, a test in this crate gives each root and file name to
`Key::parse` and fails unless each file name is refused in any segment
and each root as a first segment, so the two copies cannot drift (4,
42). A batch's file, `<batch>.rec` under `moves/`, lies where no key
reaches and needs no entry.

### Items, names and joins

An item's name is non-empty, not `.` or `..`, and holds no `/`, so it
never nests under another (35). The layout never removes an item and
nothing renames one, so the names an inbox has had are the folders
under `items/`. A join naming a held item grows it while it is
proposed, and is refused once the item is sent, since 30 gives an
attach for adding to a sent item; an attach naming an item not sent is
refused. A join naming a name not held makes the item in the same
write, `made_by: join`, named by the move's maker at the move's time.
In a batch the item is made when the batch stands, since its moves do
not stand before (32).

An item is sent once its `sends.rec` holds a send. Its weight is
computed when asked: the count of its signals, the set of their
captures' sources, and the earliest and latest of their captures' event
times, compared as instants with `time` and given as written (34;
core.8).

### The excerpt check

`read_capture` takes the capture's raw material as `RawFile`s from the
caller, who reads them from the bank its pointer names (the cut text
banked whole among them). A signal is taken when its excerpt's bytes
occur in some file's bytes (26). The crate reads no bank itself
(core.1).

### The import operation

`import` takes `ImportedCapture` values, defined here as what an
import line (C.1) carries: the capture's key, source, event time, title
and link; the pointer or none; `came_in`, which must be
`CameIn::Import`; the signals in the body's order with their numbers;
and the moves, each with a signal number, a word kept as a string, a
target, who and when. The built-in adapters unit reads the line itself
in `weir-capture` (ruling f4aabd9), and whichever of the two units
lands second gives its line type a conversion into `ImportedCapture`,
so the line is read in one place and written in one place.

For each capture in order: a held key is counted and passed over, its
signals and moves with it (38); a capture whose signals do not number
from 1, lack a field or name another nature is refused whole and
reported (28); otherwise `capture.rec` (with the machine and time the
caller hands, unread when it has no signals) and its `signals.rec` are
added, and each move is checked as `make_move` checks one, against the
inbox as this import has grown it so far; one not taken is reported
with its signal, its word and why, and its signal stays unmoved (39).
An imported join makes or grows a proposed item with no brief. Nothing
is banked and no excerpt looked for (37). The whole import is one
write, checked as every write is, with an `ImportReport`: the keys
written, the held count, each capture refused, each move not taken.

### recutils and the tests

Records are written and read with `weir_capture::rec`, one record per
blank-line-separated block. The crate's tests hold every file each
operation forms to `recfix`, through a helper like the capture crate's
test helper (that one is private to its crate's tests, so this crate
has its own), with GNU recutils from `devenv.nix`. Tests build their
inboxes in code, as `(path, text)` lists, so the change adds no fixture
file type. A test, like cadence's, fails when a file under the crate's
`src/` holds a string core.2 rules out (core.2).

### The crate and its project

`crates/weir-signals`, a library, in the workspace's `members`, with
`weir-capture` as a path dependency and `time` from
`[workspace.dependencies]`; `tempfile` for tests. It is moon project
`signals` in `.moon/workspace.yml`, its `moon.yml` as the capture
crate's: `language: 'rust'` and tasks `format`
(`cargo fmt -p weir-signals --check`), `lint`
(`cargo clippy -p weir-signals --all-targets -- -D warnings`),
`typecheck` (`cargo check -p weir-signals --all-targets`) and `test`
(`cargo test -p weir-signals`), each tagged `check-fast`, run from the
workspace root, and declaring `/crates/weir-signals/**/*`,
`/crates/weir-capture/**/*`, `/Cargo.toml`, `/Cargo.lock`,
`/devenv.nix`, `/devenv.lock` and `/.prototools` as inputs (core.39,
core.40; kit:testing.1, kit:testing.3). `Cargo.toml`'s comment on the
planned crates names the signals crate landed.

## Risks / Trade-offs

- [The import line's type lands in another unit] → The import operation
  takes this crate's `ImportedCapture`, so this unit builds and tests
  alone; the conversion from the line is one function, added by
  whichever unit lands second, and the two units touch no file in
  common.
- [S9 says an imported `new-territory` arrives unmoved, while record
  0002's ruling 3 and core.25 say it is a join] → Here every word not
  of the six arrives unmoved, which keeps both if the reader of crew's
  folders maps `new-territory` to a join before printing the line;
  that is the built-in adapters unit's, and named for madswan-design.
- [`replaces` by number reads less plainly than an identity] → The
  number is fixed once written and needs no field C.2 does not name; the
  reader checks it against the moves before it.
- [A send is named by its target and time] → Two sends of one item to
  one target at one instant are refused, which no caller needs.
- [The brief's format line is a comment] → It shows nothing in a
  rendered brief; a reader that meets a `brief.md` without it refuses
  the inbox, as for any record with no format.
- [A large inbox read whole on every write] → The write protocol reads
  the tip once per attempt; the read is linear in the files and holds
  no raw material.

## Open Questions

None.
