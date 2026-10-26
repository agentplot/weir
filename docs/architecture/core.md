# weir rules

The rules that bind wherever a change is made in this repository,
written so a reviewer can point at a diff and say which rule it breaks.
The narrative is the statement in agentplot/blueprints,
`design/weir/requirements.md`; a rule here names the clause it
enforces and never restates it. There are no area pages yet; a rule
that binds only one crate moves to an area page when that crate
exists. Every rule below is drawn from the statement the user
ratified, or from a record the user accepted; none waits on a yes.

## Rules

### The crates and what they know

- **[core.1]** No crate keeps state, starts a process that outlives its call, or names a host, tool, machine or inbox: every function answers from its inputs alone, every write is one a caller asked for, and a child a crate runs for a write, git in the branch keeper, ends with the call.
- **[core.2]** Code outside the signals-folder import and the pack's cases names no tool of the family and no source.
- **[core.3]** No record weir defines holds a secret, and no code path writes one into a record, a line or a log.
- **[core.4]** No code path judges what a capture says, chooses a move, cuts an excerpt except as the limit forces, lists a source, starts an agent, submits or lands.

### The contract

- **[core.5]** An adapter's output is lines, each exactly one of an offer, a cursor or a completion; a line that is none or several is refused as malformed, named with its reason, and stops no other line.
- **[core.6]** An offer is taken only with a key, a source and an event time, a key that names its own capture alone, and an own excerpt with every field present; a host reads of a line only the fields the contract names.
- **[core.7]** One event has one key on every machine and every run, and the key rule refuses a leading `/`, a `..` segment, and `captures`, `moves` or `items` as a first segment.
- **[core.8]** An event time is the time at the source, never the time of listing or taking, and every age and weight reads it.
- **[core.9]** A cursor is opaque: no code path but its adapter's parses, compares or makes one, and a host hands back only the last cursor of a complete run.

### What a host writes

- **[core.10]** An inbox holds at most one capture per key: a held key writes nothing, banks nothing and is counted held, and no later offer under a key changes its capture.
- **[core.11]** Raw material is banked whole and unchanged on the machine that took it before its capture is written; a record holds a pointer `<hostname>:<path>` and never the material; nothing an adapter wrote that no taken capture names outlives the run.
- **[core.12]** The cut is one function: whole lines from the start to the last that fits together with the host's closing line, the closing line alone when none fits, nothing added when the text fits, and the whole text banked whenever it is cut.
- **[core.13]** A capture and its own signal are written in one write; a capture with an own excerpt has exactly one signal and is never unread, and every other capture is unread when written.
- **[core.14]** Two hosts given the same lines write the same records, cut the same way: no host setting, clock, machine or random value reaches a record but the taken-at time, the machine's name and how the capture came in.
- **[core.15]** A run is recorded whole, what it took, held and refused with reasons, the completion's failures, the cursor kept and whether it was complete; a run without a completion or reporting failure advances no cursor and keeps every offer it made.

### Signals, moves and items

- **[core.16]** A signal's excerpt stands verbatim in its capture's raw material or is the own excerpt as cut, or the signal is refused; an imported signal alone is exempt.
- **[core.17]** A signal's identity is `<key>#<n>` in reading order; a capture has at most one reading, written whole or refused whole naming each signal not taken.
- **[core.18]** A signal has at most one standing move, a second is refused, only a replacement naming who replaced what changes it, and a move is one of the six with its consequence or is refused.
- **[core.19]** A batch is written whole or refused whole, and while it is proposed its moves do not stand; an unmoved signal is never deleted, however old.
- **[core.20]** An item's signals are read from standing moves and its weight is computed from them and never stored; an item's name is one its inbox has never had, nests under no other and holds no `/`; a host never writes a brief; a send keeps the brief as handed.

### Records and layout

- **[core.21]** A capture, a signal and a move never change once written; what changes is added beside them, and only `brief.md` is rewritten in place.
- **[core.22]** Every record names `format: weir-<record>/<major>`; a reader of a later major reads every earlier one, and no code path writes a record in a format it did not read.
- **[core.23]** Each capture lies at the one place its key fixes, with its signals beside it; moves, batches and items lie where no key reaches; an inbox is read whole from one tip, and a host keeps nothing of it the layout does not place.
- **[core.24]** Records are recutils at rest and JSON only in flight, and every file a write changes passes `recfix` and the records' checks before it lands.
- **[core.25]** An import copies no raw material, checks no imported excerpt, keeps the key a capture was first held under and holds imported captures by key like any other; a move of another word, or without its consequence, arrives unmoved, and crew's `new-territory` is a join into a proposed item named for its target.

### The shared branch

- **[core.26]** A write is formed against the tip its writer read, every check is made on the tip it lands on, and it lands only there; a refused push replays the write on the new tip, checks and all, up to the writer's bound, five unless set, then reports refused and never retries on its own.
- **[core.27]** A write is one commit through a temporary index touching no working copy, pushed never with force and with the tip read as the expected old value, and a branch with no remote moves by `update-ref` from that expected value.
- **[core.28]** A lock keeps one machine's writers in turn, names its holder and its time, and never stands in for the check at the tip.
- **[core.29]** Every write's commit names what it did and who asked for it.

### Kinds

- **[core.30]** The kind format is weir's, read through `read-kinds` alone; a reader returns a section it does not know unread and unrefused, fails on none, and changes no declaration.
- **[core.31]** A declaration is checked whole and refused naming the field: no stage, a stage with no name or prompt, a field of the wrong shape, a deliverable not of the six spellings; a version left out reads as 1.
- **[core.32]** No code path in weir resolves a kind's name among a tool's, a repository's and a launch's: that is the reader's rule.

### Cadence

- **[core.33]** Cadence reads a schedule, the last run and now and nothing else, in the machine's local time, and a run missed while the machine slept is caught up once.

### Adapters and the pack

- **[core.34]** A built-in adapter speaks the contract through the same lines as any adapter and is run the same way; the folder drop's and the meeting file's keys are by the sha256 of the file.
- **[core.35]** Every case of the pack cites the clauses it exercises; the pack's check fails when a cited number names no clause or a clause of Part A is cited by no case; the pack runs with no network, no source and fixed inputs.
- **[core.36]** Every profile, host and adapter passes the pack, one that fails it is not one, and a profile adds nothing to Parts A and B.

### Design and records

- **[core.37]** A change that bears on a clause cites the clause by number; a clause that no longer holds is changed in the statement first, never worked around here.
- **[core.38]** Nothing commercial, no pricing, tier or customer, appears in this repository.

### The gate

- **[core.39]** Every file the tree tracks is owned by a formatter the gate runs, or is excluded by name with its reason; every language with a linter is linted; every crate is a moon project of its own, listed in `.moon/workspace.yml` with its language, whose check-fast tasks format, lint and test it, one task per check.
- **[core.40]** Every tool a gate task runs is the version the tree pins, on every host that runs the gate.

## Details

- **core.1** Rules out: a static or a file a crate writes for itself; a thread, timer or server a crate starts; a daemon, a watcher or a child left running; a crate reading a host's settings, an environment variable or the machine's name except where a clause hands it one (the taken-at machine, 22; the clock, 83). Not ruled out: the `weir` command line, which is a caller of the crates (section 9); `git` run and waited on by the branch keeper for one write (C.3), which ends with the call. Source: clauses 69, 70, I1; record 0001.
- **core.2** Rules out: the strings `millpond`, `flume`, `sounding`, `crew`, `headwater`, `eddy`, `Flywheel`, `swancloud` in any module but the signals-folder import's reading of crew's folders (C.6) and the pack's case files. Not ruled out: `git`, `recfix`, `cron`, which section 9 fixes. Source: purpose; clauses 12, 64, 69; section 9.
- **core.3** Rules out: a secret's value in a record, an offer line, a declaration, a commit message or a log; a secret passed on a command line. Not ruled out: a secret's name in a declaration (11). Source: clauses 11, 71, I3.
- **core.4** Rules out: a reader of captures in a crate; a default move; a cut by anything but the limit; a listing of a source outside the three built-in adapters; `flume launch`, `sluice submit` or any landing called from a crate. Source: clauses 12, 20, 70.
- **core.5** Rules out: a line with two of `capture`, `cursor`, `done`; a line with none; a line that aborts the run or the lines after it. Source: clauses 1, 10, 13, 15; C.1.
- **core.6** Rules out: an offer taken without `event_at`; an excerpt taken with `text` missing or empty; a host-defined field read from a line; an unknown field treated as an error. Source: clauses 2, 7, 13; C.1.
- **core.7** Rules out: a key that embeds the machine, the run or the time; `captures/x`, `moves/x`, `items/x`, `/x`, `a/../b` as keys. Source: clauses 3, 4; C.1; the user's ruling of 2026-10-26 (record 0002).
- **core.8** Rules out: `event_at` set from `now()`; an age computed from `taken_at`. Source: clauses 5, 33.
- **core.9** Rules out: a cursor parsed as a time or compared for order by a host; a cursor advanced after an incomplete run. Source: clauses 8, 9.
- **core.10** Rules out: an overwrite of `capture.rec`; a second capture folder for one key; a held key reported as refused. Source: clauses 14, 16, I2.
- **core.11** Rules out: raw bytes in a `.rec`; a capture written before its raw files are banked; a pointer by IP, by the fleet's name or by the long hostname; raw files kept from an offer that was refused or held. Source: clauses 6, 17, 18, I3; record 0002.
- **core.12** Rules out: a cut mid-line; a closing line added to an uncut text; a cut text whose whole is not banked; two implementations of the cut. Source: clauses 20, 21.
- **core.13** Rules out: `capture.rec` and `signals.rec` in two commits; a capture with an own excerpt marked unread; an own signal numbered other than 1. Source: clauses 19, 24; C.2.
- **core.14** Rules out: a limit read from a host's configuration into a record's shape; a random id in a record; a locale-dependent rendering. Not ruled out: `taken_at`, `machine` and `came_in` (22). Source: clauses 22, 24, I8.
- **core.15** Rules out: a run recorded as complete without a `done` line; a cursor kept from a failed run; a refusal dropped from the run record. Source: clauses 9, 23; C.1 step 7.
- **core.16** Rules out: a signal whose excerpt is paraphrased; a signal taken whose excerpt is not found in the banked material. Not ruled out: an imported signal (37). Source: clauses 26, 37.
- **core.17** Rules out: a second `signals.rec` reading; a signal numbered by hash or time; a reading partly written. Source: clauses 27, 28.
- **core.18** Rules out: a move that overwrites another; a seventh word; a drop without a reason, a join without an item. Source: clauses 29, 30, 31, I6.
- **core.19** Rules out: a batch half applied; a move counted standing while its batch is proposed; a purge of old unmoved signals. Source: clauses 32, 33.
- **core.20** Rules out: a `weight` field; an `item.rec` listing signals a move does not name; an item renamed to a name once used; a brief a host generates; a send that re-reads the brief later. Source: clauses 34, 35, 36.
- **core.21** Rules out: an edit to `capture.rec`, `signals.rec` or a line of `moves.rec`; a send's outcome written over the send. Source: clause 41.
- **core.22** Rules out: a `.rec` without `format:`; a reader that refuses `weir-capture/1` once `/2` exists; a writer emitting a major it cannot read back. Source: clause 40, I5.
- **core.23** Rules out: a capture indexed in a second file; an item under `captures/`; a cache of an inbox apart from the branch; a read that mixes two tips. Source: clauses 42, 43; C.2.
- **core.24** Rules out: a JSON file under `inbox/`; a commit whose changed `.rec` files were not run through `recfix`; a write landed with a held key or a used item name unchecked. Source: clauses 46; C.1; C.2; section 9 (D4).
- **core.25** Rules out: `cp` of raw material at import; an imported excerpt refused for not matching; an imported capture re-keyed on a second import; a `new-territory` move dropped. Source: clauses 37, 38, 39, 63; record 0002.
- **core.26** Rules out: a push with `--force`; a replay that reuses checks from the old tip; a retry loop without a bound; a bound the keeper sets. Source: clauses 44, 46, I4; C.3; record 0002.
- **core.27** Rules out: a checkout of the shared branch; `git commit` in a working tree; `update-ref` without the expected old value. Source: clauses 45, 47, 48; C.3.
- **core.28** Rules out: a push made because the lock is held; a lock file without holder and time. Source: clause 49.
- **core.29** Rules out: a commit message that says only "update". Source: clause 50.
- **core.30** Rules out: a second kind parser in the family; `[kinds.x.elaboration]` refused by a reader of places; a declaration rewritten on read. Source: clauses 51 to 58, 81, 82, I7; C.4.
- **core.31** Rules out: a stage without `prompt` accepted; `deliverables = ["commit"]` accepted; a refusal that names no field. Source: clause 57; C.4.
- **core.32** Rules out: a precedence of tool, repository and launch coded in `weir-kinds`. Source: clause 58.
- **core.33** Rules out: a schedule read in UTC; a catch-up run per missed slot; cadence reading a file or a network. Source: clauses 59, 60, 83 to 85; C.5; record 0002.
- **core.34** Rules out: a built-in adapter writing records; a key by path or by modification time. Source: clauses 61, 62, 64; C.6; record 0002.
- **core.35** Rules out: a case with no `satisfies:`; a Part A clause no case names; a case that fetches. Source: clauses 65 to 68; C.7.
- **core.36** Rules out: a profile merged with no pack run; a profile that relaxes a clause. Source: clauses 86, 87.
- **core.37** Rules out: a comment or commit that reinterprets a clause; a behaviour the statement forbids kept behind a flag. Source: record 0001; `design/requirements-style.md`.
- **core.38** Rules out: a pricing table, a tier name, a customer name in code, docs or the README. Source: record 0001; section 9.
- **core.39** Rules out: a file type the formatter neither formats nor excludes; an exclude with no comment saying why; a crate with no project in `.moon/workspace.yml`; one task bundling several checks; a change whose only verification is a command run by hand. Source: the kit standard's code profile; tailrace record 0008 as the family precedent; the user's rule on projects per part.
- **core.40** Rules out: a tool a task finds outside its pin; `rust` or `moon` pinned in two files with no check that fails when they differ. Source: `.prototools`; tailrace record 0008.
