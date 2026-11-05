# Spec Delta

## ADDED Requirements

### Requirement: A kind's key is a name
`read-kinds` SHALL refuse a kind's table, or a table of settings, under
a key that is not one or more ASCII letters, digits, `_` and `-`,
naming the field by its key as TOML quotes it, and SHALL return no kind,
so every kind it reads has an identity (57, 88; C.4 as amended at
blueprints 361a599; record 0004).

#### Scenario: A kind under a key that is not a name
- **WHEN** a declaration holds `[[kinds."a/b".stages]]` with a name, a prompt and `deliverables = ["commits"]`
- **THEN** it is refused naming `kinds."a/b"`, the reason saying a kind's name is one or more ASCII letters, digits, `_` and `-`

#### Scenario: Settings under a key that is not a name
- **WHEN** a declaration holds `[kinds."review@2".agents]` with `model = "opus"`, beside a kind `bump` it declares
- **THEN** it is refused naming `kinds."review@2"`, and returns no kind

#### Scenario: Names of every allowed character
- **WHEN** a declaration declares `code_review`, `review-2` and `R2`
- **THEN** it reads with all three kinds

### Requirement: The pack holds package and lock cases
A package's case SHALL hold the package's files, and its links where it
has any, written in the case, the identity it is held at, and a lock to
check it against where it names one; and either its kind with its
identity and content hash, or the refusal. A lock's case SHALL hold the
lock and either its identities with their hashes, or the refusal (C.7
as amended at blueprints 361a599). The pack SHALL hold a case citing
each of 88 to 93.

#### Scenario: A package laid out in two folders (S17)
- **WHEN** the kinds crate's tests read a package's case
- **THEN** they lay its files and links out in two folders at different paths, read each at the case's identity, and fail naming the case where the two answers differ or either differs from the case's

#### Scenario: The cases the statement gives
- **WHEN** the pack's kinds cases are listed
- **THEN** they hold flume's change@1 and change@2 each read as a package of its own (S10); a prompt named `../prompts/code.md` refused, the same package with the prompt inside it read, and its prompt a link to another file inside it refused naming the link (S17); a package changed by a byte refused against a lock naming the kind and both hashes (S18); a package held at another version refused naming both identities; a lock read with two versions of a kind; a lock recording one identity twice refused; and a declaration naming a kind by a key that is not a name refused naming it

#### Scenario: The cases cite 88 to 93
- **WHEN** the clauses the kinds cases cite are gathered
- **THEN** each of 88, 89, 90, 91, 92 and 93 is among them

## MODIFIED Requirements

### Requirement: A stage reads with its name, prompt and end
A stage SHALL read with its name, its prompt as written, and either its
one deliverable or that it is a conversation (53). It SHALL read its
standing instructions' path as written (none when left out, and refused
naming the field when it is not a non-empty string), its agents with
their settings, the names of what it makes besides its deliverable
(none when left out), whether it waits (false when left out), whether it
repeats per task group (none when left out), and its `quiet`, `on_fail`
and `max_send_backs`, each of the last three returned left out when
left out (52, 53, 57, 58, C.4).

#### Scenario: A stage reads with its settings
- **WHEN** a stage declares `name = "review"`, a prompt, `conversation = true`, `wait = true`, `agents = [{ beside = "operator" }]`, `on_fail = "code"` and `max_send_backs = 3`
- **THEN** it reads as a conversation named `review` that waits on its caller's word, with one agent holding `beside = "operator"`, sending a failed place back to `code` at most three times

#### Scenario: Settings left out are returned left out
- **WHEN** a stage declares no `quiet`, `on_fail`, `max_send_backs` or `repeat`
- **THEN** it reads with each of the four left out, and no value stands in for any of them

#### Scenario: A stage reads what it makes
- **WHEN** a stage declares `makes = ["preview", "page"]`, and another stage declares no `makes`
- **THEN** the first reads making `preview` then `page`, with no section unread, and the second making nothing

#### Scenario: A stage reads its standing instructions
- **WHEN** a stage declares `standing = "prompts/bump-code-standing.md"`, and another stage declares no `standing`
- **THEN** the first reads with that path as written, with no section unread, and the second with none

#### Scenario: Standing instructions of the wrong shape
- **WHEN** a stage declares `standing = ""` or `standing = 3`
- **THEN** the declaration is refused naming that stage's `standing` field

### Requirement: The pack holds the kind format's cases

The conformance pack SHALL hold cases of the kinds shape at `pack/kinds/<case>/case.toml` (C.4; C.7), each of a declaration, a package or a lock. A declaration's case holds the declaration, written in the case, and either the kinds it reads into, each with its version and its stages in order, each stage's name, its deliverable or that it is a conversation, and its `repeat` where it repeats, the sections returned unread, and the names of the kinds it gives settings to without declaring them; or the refusal, each field named with a phrase its reason holds; citing the clauses it exercises (67). The kinds crate's tests SHALL read every case and fail naming each one whose kinds, settings, identity, hash or refusal differ (65, 87; core.36). The pack SHALL hold a case citing each of 51 to 58, among them flume's own declaration read twice the same and flume's change@2 read twice the same with its code stage repeating (S10), an elaboration kind read with its section returned unread (S11), a stage with two deliverables and a declaration with no stage, each refused naming the field (S12), a kind's `agents` table beside settings given to a kind declared elsewhere (52, 57), and a `repeat` on a stage of commits refused naming the field (57). A field the case format does not name SHALL fail the case's read, naming the case and the field.

#### Scenario: Every kinds case is answered as it says
- **WHEN** each kinds case of the pack is read by the kinds crate
- **THEN** every declaration reads into the kinds and settings its case expects, every package into its kind, identity and hash, and every lock into its entries, or each is refused naming the fields its case expects

#### Scenario: A stage with two deliverables (S12)
- **WHEN** a case's declaration gives a stage `deliverables = ["commits", "tasks"]`
- **THEN** the case expects the refusal naming that stage's `deliverables`, and the crate gives it

#### Scenario: Settings for a kind declared elsewhere
- **WHEN** a case's declaration gives `change` an `agents` table and no stages, beside `bump` with an `agents` table and stages
- **THEN** the case expects `bump` with no section unread and settings for `change`, and the crate gives them

#### Scenario: flume's change@2 (S10)
- **WHEN** a case's declaration gives `change` at version 2 with `construct`, `code` and `verify`, its `code` delivering `tasks` with `repeat = "per-task-group"`
- **THEN** the case expects `change@2` with `code` repeating per task group and the other two stages not, and the crate gives it, the same twice

#### Scenario: A misspelt field of the case
- **WHEN** a kinds case names `kind` in place of `expect`
- **THEN** reading it fails, naming the case and the field

#### Scenario: A case of more than one form
- **WHEN** a kinds case holds both a `declaration` and a `lock`
- **THEN** reading it fails, naming the case and saying it holds one of a declaration, a package and a lock
