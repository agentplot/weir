# weir-command Specification

## Purpose
The `weir` command line: the caller that runs the built-in adapters and the import, holding no rule of its own (section 9; core.1).

## Requirements

### Requirement: The adapters run as a host runs any adapter

`weir capture folder <dir>` and `weir capture meeting <file>` SHALL take the arguments a host appends, `--raw-dir <dir>` and optionally `--cursor <c>`, print the adapter's lines to standard output, one JSON object a line, and exit 0 after the completion (C.1; core.34). The configuration file `CAPTURE_CONFIG` names SHALL not be needed.

#### Scenario: The folder drop through the command
- **WHEN** `weir capture folder <dir> --raw-dir <raw>` runs over a folder of two files
- **THEN** standard output holds two lines `read_line` reads as offers and a last line it reads as a completion listing two, and the command exits 0

#### Scenario: A host's take of the command's lines
- **WHEN** the lines of `weir capture folder` over a folder are taken by `take` with that raw place, the folder holding a file dropped twice under two names across two runs
- **THEN** the host writes one capture, its raw material the file's bytes (S14)

### Requirement: A run that cannot begin says why and prints no completion

A run that cannot begin, a missing `--raw-dir`, a folder or file that is not there, a folder whose name the key rule refuses, SHALL print no completion, say why in one line on standard error and exit non-zero, so a host records it incomplete (9; C.1).

#### Scenario: No raw place
- **WHEN** `weir capture meeting call.txt` runs with no `--raw-dir`
- **THEN** it prints nothing to standard output, one line to standard error naming `--raw-dir`, and exits non-zero

#### Scenario: A cursor handed back
- **WHEN** `weir capture folder <dir> --raw-dir <raw> --cursor c1` runs
- **THEN** it prints the same lines as with no cursor, and no cursor line

### Requirement: The import prints one import line per capture

`weir import signals <dir>` SHALL print one import line per capture `read_signals_folder` gives, in its order, and write no file (63; C.6). `--moves <file>` SHALL hand the reader that `moves.rec`, the flywheel branch's for a blueprints corpus, in place of the root's (C.6). A root, or a moves file named, it cannot read SHALL print nothing to standard output and exit non-zero, saying why.

#### Scenario: A body imported
- **WHEN** `weir import signals <root>` runs over a root with three whole captures
- **THEN** standard output holds three lines `read_import_line` reads, and the command exits 0

#### Scenario: A corpus imported with the flywheel branch's moves
- **WHEN** `weir import signals <blueprints root> --moves <branch tree>/moves.rec` runs, the branch's file moving one of the corpus's signals
- **THEN** the corpus's line carries that move, and the command exits 0

#### Scenario: A moves file that is not there
- **WHEN** `weir import signals <root> --moves <missing>` runs
- **THEN** it prints nothing to standard output, says why on standard error, and exits non-zero

#### Scenario: Run twice
- **WHEN** `weir import signals <root>` runs twice over one root
- **THEN** both runs print the same bytes (S8)

### Requirement: What the import could not carry is named on standard error

The import SHALL name on standard error, one line each, every capture it could not read whole and every move it did not carry, with why. It SHALL exit 0 when every capture was read whole, and non-zero after printing the rest when any was not; a move not carried SHALL not change the exit status (15).

#### Scenario: One capture not read
- **WHEN** one of four captures has a signal of no nature
- **THEN** three lines are printed, standard error names the fourth's folder, file and `kind`, and the command exits non-zero

#### Scenario: A move not carried
- **WHEN** a root's `moves.rec` moves a signal of no capture in it
- **THEN** standard error names that signal, and the command exits 0

### Requirement: The command names no tool

The command's own code SHALL name no tool of the family: what `weir import signals` says of crew's folders, its help among it, SHALL come from the reader of crew's folders (core.2). An unknown command or a missing argument SHALL print the usage on standard error and exit 2.

#### Scenario: An unknown command
- **WHEN** `weir capture nothing` runs
- **THEN** it prints the usage on standard error and exits 2
