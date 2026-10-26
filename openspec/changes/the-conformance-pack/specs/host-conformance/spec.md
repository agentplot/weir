# Spec Delta

## Purpose

Holding a host to the contract: the host shape's cases, and
`weir conform <host command>`, which runs a host against every one of
them through the seam the pack fixes and says which it passed (statement
13 to 24, 37 to 39, 65, 75 to 77, 87; C.7; S15).

## ADDED Requirements

### Requirement: A host case holds its steps, its starting records and the records expected

A host case SHALL hold, in its folder: `case.toml` naming its clauses, the limit, and one or more steps in order; `start/`, the files the branch holds before the first step, which may be absent; and `tip/`, every file the branch's tip must hold after the last step. Each step SHALL name its invocation, `intake`, `run` or `import`, and its input: for an intake, a lines file (`lines.jsonl`) and optionally a raw place, a folder of the case; for a run, the case's adapter, a POSIX shell script in the case's folder that prints the case's lines, writes the case's raw material into the raw place it is handed and exits as the case says; for an import, a file of import lines. Each step SHALL say what the host must report of it: the keys taken in order, the count held and each refusal by its line, its key and a phrase its reason holds; for a run step, also the listing's count, its failures, the cursor kept and whether the run was complete (15, 23; C.7). A field the case format does not name SHALL fail the case's read, naming the case and the field.

#### Scenario: A case read
- **WHEN** a host case with two run steps and a `start/` holding one capture is read
- **THEN** both steps are read in order, each with its adapter and its expected report, and the starting and expected files by their paths from the branch's root

#### Scenario: A misspelt field
- **WHEN** a host case's step names `refusals` in place of `refused`
- **THEN** reading it fails, naming the case and `refusals`

### Requirement: weir conform runs each case against a fresh bare repository

For each host case, `weir conform <host command>` SHALL make a bare git repository with no remote in a temporary folder, its branch `inbox` seeded with the case's starting records in one commit, or with an empty tree when the case has none, and SHALL run the host once per step, in order, against that repository (C.7). Nothing SHALL be fetched or pushed to any other place, and the temporary folder SHALL be removed when the case ends (68).

#### Scenario: A case of a held key
- **WHEN** a case's `start/` holds `inbox/captures/release/acme/v1/capture.rec` and its one step offers `release/acme/v1`
- **THEN** the host is run against a branch whose tip already holds that capture, and the case expects it held

### Requirement: Each step is one of three invocations with the case's flags

`weir conform` SHALL append to the host command, for every step, `--inbox <repository path> --branch inbox --limit <bytes>`, the limit the case's, and then: for an intake, `--raw-dir <the case's raw place>` when the step names one, the step's lines on standard input; for a run, `--run <declaration>`, a declaration it writes naming the case's adapter by the case's name, its command the case's script, its configuration's schema one that takes any configuration and no secret; for an import, `--import`, the step's import lines on standard input (C.7). The host command SHALL be the words given after `conform`, run with no shell.

#### Scenario: An intake step
- **WHEN** a step is an intake with a raw place and `weir conform host-under-test --flag` runs it
- **THEN** the host is run as `host-under-test --flag --inbox <repo> --branch inbox --limit <bytes> --raw-dir <raw>` with the step's lines on standard input

#### Scenario: A run step
- **WHEN** a step is a run
- **THEN** the host is run with `--run <declaration>` and no lines on standard input, the declaration's command the case's script

#### Scenario: An import step
- **WHEN** a step is an import
- **THEN** the host is run with `--import` and the step's import lines on standard input

### Requirement: The report a host prints is compared with the step's

After each step, `weir conform` SHALL read the last line the host printed on standard output as `weir-report/1`, and SHALL fail the case when it is missing or does not read, or when its keys taken, its count held or its refusals differ from the step's: a refusal is matched by its line and key, and its reason SHALL hold the phrase the step names (15; C.7). For a run step it SHALL also compare the listing's count and failures, the cursor kept and whether the run was complete (8, 9, 23). A host that exits non-zero and prints no report SHALL fail the case, naming its exit and the last line of its standard error.

#### Scenario: A refusal with another reason
- **WHEN** a step expects line 3 refused for key `moves/x` with a reason holding `first segment`, and the host reports line 3 refused for `moves/x` because `bad key`
- **THEN** the case fails, naming the line, the key, the phrase expected and the reason reported

#### Scenario: A run whose cursor is kept from an incomplete run (S4)
- **WHEN** a run step's adapter offers five captures and exits without its completion, and the host reports the run's last cursor as kept
- **THEN** the case fails, naming the cursor expected, none, and the cursor reported

### Requirement: The records at the tip are compared as written

After the last step, `weir conform` SHALL read every file at the branch's tip and SHALL fail the case when the tip holds a file `tip/` does not, lacks one `tip/` holds, or holds one whose text differs from it (24; C.7; core.14). In a capture the host wrote during the case, three fields SHALL be checked for their form in place of their text, since they are the host's and its machine's (22): `machine` SHALL be this machine's short hostname; `taken_at` SHALL be an RFC 3339 time within the case's running; and `pointer`, present exactly where `tip/` holds one, SHALL be `<machine>:<path>`, the machine the record's, and `<path>` a place on this machine that holds, byte for byte, every raw file the case's offer named, and the whole text of an own excerpt that was cut (17, 18, 21). Every other field and every other file SHALL be compared byte for byte.

#### Scenario: A host keeps raw material in a record (S15)
- **WHEN** the host under test writes a capture's raw notes into its `capture.rec`
- **THEN** the case fails, naming the file that differs and the clauses the case cites, 17 among them

#### Scenario: A host keeps raw material beside its record
- **WHEN** the host under test writes a copy of a raw file under `inbox/captures/<key>/`
- **THEN** the case fails, naming the file the tip holds that the case does not

#### Scenario: A pointer to a bank that lacks a file
- **WHEN** a capture's pointer names a path on this machine that does not hold one of the raw files the offer named
- **THEN** the case fails, naming the capture and the raw file not banked

#### Scenario: A field of the host's own
- **WHEN** two hosts that keep the contract run the same case on one machine at different times
- **THEN** both pass, though their `taken_at` differ

### Requirement: weir conform says which cases passed and exits on the verdict

`weir conform` SHALL print one line per case, its name and `passed`, or `failed` with the clauses the case cites and each difference found, and SHALL exit 0 when every case passed, 1 when any failed, and 2 when its command line names no host command (65, 87). `--pack <dir>` SHALL read the cases from that folder in place of the pack built into the command.

#### Scenario: A host that keeps the contract
- **WHEN** `weir conform` runs the pack's example host against every host case of the pack
- **THEN** every case passes, and it exits 0

#### Scenario: A host that fails one case
- **WHEN** a host fails one case
- **THEN** that case's line says `failed`, naming its clauses and what differed, every other case still runs, and it exits 1

#### Scenario: No host command
- **WHEN** `weir conform` runs with nothing after `conform`
- **THEN** it prints its usage on standard error and exits 2

### Requirement: The pack's host cases cover what a host does

The pack SHALL hold host cases exercising at least: an offer taken with its raw material banked and an own excerpt cut at the limit (S1); the same offer met again and held (S2); a run of fourteen offers, one keyed under `moves`, that one refused and thirteen taken (S3); a run that ends without its completion and the next run from the old cursor (S4); a malformed line among good ones; an offer naming raw material at an intake with no raw place; an offer carrying a field the contract does not name, holding a secret's value, which reaches no record (71; core.6); an import whose captures, signals and moves are written as read, imported twice the second time writing nothing, with a `new-territory` move and a move of another word (37 to 39, 63, 75a; S8, S9).

#### Scenario: The cases named
- **WHEN** the host cases of the pack are read
- **THEN** each kind of case this requirement names is among them, and the example host passes every one
