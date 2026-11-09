# Verify

The work on `change/a-delivery-is-done-when-its-reader-confirms-it` at
db901e5, cut from `bolt/one-delivery-form-in-weir` at 323b9e1, holds
this change. The repository's whole checks pass. Every requirement and
scenario of the four specs is met by a test that passes in them. All 22
tasks are ticked, and each one's condition holds. The change is
archived.

## Checks

- `devenv shell -- check-fast`, run once at db901e5: exit 0, 41 tasks
  completed, none failed. `root:prose` was taken from moon's cache,
  which keeps a task's outcome only when it passed and only for the
  same inputs. The other 40 ran: `root:format`, `root:toolchain`,
  `root:openspec-claude` and `root:hooks`, and every project's `format`,
  `lint`, `typecheck` and `test`, for `adapters`, `branch`, `cadence`,
  `capture`, `delivery`, `kinds`, `pack`, `signals` and `weir`. Every
  test binary passed, 749 tests in all, `delivery:test`'s 37 and
  `pack:test`'s 71 among them (core.39).
- `openspec validate a-delivery-is-done-when-its-reader-confirms-it
  --strict`: valid. Each of the four requirements the delta modifies in
  `conformance-pack` stands under the same name in
  `openspec/specs/conformance-pack/spec.md` and keeps every scenario it
  holds there.
- `openspec archive` applied the delta, 29 requirements added and 4
  modified. The new specs' Purposes are written from the proposal's
  capabilities, and conformance-pack's names seven shapes. After it,
  `openspec validate --specs --strict` passes the three new specs.
  conformance-pack fails it on a warning: its index requirement is 631
  characters, over the validator's 500, as the delta writes it. Ten
  other specs, which this change leaves as they stand, fail it the same
  way. The repository's checks do not run it.
- The statement at blueprints 006fc34, read from a local clone that
  holds it:
  - `pack/clauses.toml` names commit `006fc34` and holds every clause
    of Parts A and B under its section but 94 and 95 (A.8), 96 to 98
    (A.13) and 99 (B.4), which its header leaves out until weir's
    statechart change. A.14 holds 100 to 107 and B.7 108 to 113, as the
    statement does.
  - `crates/weir-delivery/tests/json.rs`'s `C8` is byte for byte the
    four lines of C.8's example.
- No module, test or case of the change names a tool of the family
  (core.2). `weir-delivery`'s source reads no clock, environment or
  file (core.1, core.51).

## Requirements and scenarios

Each scenario is held by the test of its name, which passed in the
checks above.

### delivery-records

| requirement | scenario | test |
|---|---|---|
| A delivery's records are the four of C.8 | Each record names its format | `weir-delivery/tests/delivery.rs` `each_record_names_its_format` |
| A key, a reader and a way are each one line | A key holding a line break | `tests/text.rs` `a_key_holding_a_line_break` |
| | A reader with a space at its end | `tests/text.rs` `a_reader_with_a_space_at_its_end` |
| A message keeps its lines | A message of several lines | `tests/delivery.rs` `a_message_of_several_lines` |
| A time is RFC 3339, kept as written | A time with no offset | `tests/text.rs` `a_time_with_no_offset` |
| The records are spelled in recutils | Four records spelled and read back | `tests/rec.rs` `four_records_spelled_and_read_back`, running `recfix --check` over the text |
| A record out of its format is refused | A record of a format the crate does not read | `tests/rec.rs` `a_record_of_a_format_the_crate_does_not_read` |
| | A field the format does not name | `tests/rec.rs` `a_field_the_format_does_not_name` |
| The records are spelled in JSON | C.8's example read | `tests/json.rs` `c8s_example_read` |
| | A delivery with its readers left out | `tests/json.rs` `a_delivery_with_its_readers_left_out` |
| A delivery's expiry is its recorded time and its span | A week's span | `tests/delivery.rs` `a_weeks_span` |

### delivery-calls

| requirement | scenario | test |
|---|---|---|
| The calls are functions over the records their caller passes | A call made twice | `tests/owed.rs` `a_call_made_twice` |
| record returns the delivery to add | A delivery recorded for two readers | `tests/record.rs` `a_delivery_recorded_for_two_readers` |
| record holds a key once | One key recorded twice (S24) | `tests/record.rs` `one_key_recorded_twice` |
| record refuses a delivery out of its shape | A delivery with no reader | `tests/record.rs` `a_delivery_with_no_reader` |
| | A reader named twice | `tests/record.rs` `a_reader_named_twice` |
| | A span of no seconds | `tests/record.rs` `a_span_of_no_seconds` |
| owed gives every delivery a reader has not confirmed | Deliveries in the order recorded | `tests/owed.rs` `deliveries_in_the_order_recorded` |
| | A delivery one reader has confirmed | `tests/owed.rs` `a_delivery_one_reader_has_confirmed` |
| owed marks a delivery past its expiry expired | An expired delivery for a misspelt reader (S23) | `tests/owed.rs` `an_expired_delivery_for_a_misspelt_reader` |
| owed says whether a reader's name is known | A name stays known after its deliveries are let go of | `tests/owed.rs` `a_name_stays_known_after_its_deliveries_are_let_go_of` |
| owed lists the deliveries done | A delivery done when every reader confirms (S22) | `tests/owed.rs` `a_delivery_done_when_every_reader_confirms` |
| confirm returns the confirmation to add | A reader confirms by its key | `tests/confirm.rs` `a_reader_confirms_by_its_key` |
| | An expired delivery confirmed | `tests/confirm.rs` `an_expired_delivery_confirmed` |
| confirm takes a reader's confirmation once | A second confirmation (S24) | `tests/confirm.rs` `a_second_confirmation` |
| confirm refuses a key or a reader the records do not hold | A confirmation of a key no delivery is under | `tests/confirm.rs` `a_confirmation_of_a_key_no_delivery_is_under` |
| hand returns a hand-over, which confirms nothing by itself | A hand-over by a notice | `tests/hand.rs` `a_hand_over_by_a_notice` |
| A reader confirms on receipt in the call that hands it over | A reader that confirms on receipt | `tests/hand.rs` `a_reader_that_confirms_on_receipt` |
| hand hands nothing to a reader that confirmed or is not named | A hand-over to a reader that has confirmed | `tests/hand.rs` `a_hand_over_to_a_reader_that_has_confirmed` |
| | A hand-over to a reader the delivery does not name | `tests/hand.rs` `a_hand_over_to_a_reader_the_delivery_does_not_name` |
| A deliverer's store is a seam of two calls | A store implemented outside the crate | `weir-pack/tests/delivery_cases.rs` `a_store_implemented_outside_the_crate` |

The paths above are under `crates/weir-delivery/` unless they name
`weir-pack`. `Store` in `src/store.rs` is implemented for nothing in the
crate.

### delivery-cases

| requirement | scenario | test |
|---|---|---|
| A delivery case holds a store's starting records and one call | A misspelt call | `weir-pack/tests/delivery_cases.rs` `a_misspelt_call` |
| A delivery case names one thing that must come of its call | A case that expects both records and a refusal | `delivery_cases.rs` `a_case_that_expects_both_records_and_a_refusal` |
| The pack runs a delivery case through a store its caller hands it | The cases through a store of each spelling | `delivery_cases.rs` `the_cases_through_a_store_of_each_spelling`, through a list, recutils text and JSON lines |
| The pack fails a store that does not read back what it added | A store that drops expired deliveries (S25) | `delivery_cases.rs` `a_store_that_drops_expired_deliveries`, failing `an-expired-delivery-is-shown` with `not read back:` and its `weir-delivery/1` line |
| | A store that loses known names | `delivery_cases.rs` `a_store_that_loses_known_names`, failing `a-name-stays-known` with `not read back:` and its `weir-known/1` line |
| The delivery cases hold A.14 | The cases named | `delivery_cases.rs` `the_cases_named` |
| The delivery cases run from fixed inputs alone | A case read from its folder | `delivery_cases.rs` `a_case_read_from_its_folder` |

`pack/delivery/` holds the 21 cases of design 13, each reading through
`read_delivery_case`, with the call, the expectation and the clauses
the table gives. 100 to 107 are each cited.

### conformance-pack

| requirement | scenario | test |
|---|---|---|
| Every case lies in one folder under its shape | A folder of no shape | `weir-pack/tests/check.rs` `a_folder_of_no_shape` |
| | A case that cites nothing | `check.rs` `a_case_that_cites_nothing` |
| | A clause with a letter | `check.rs` `a_clause_with_a_letter` |
| | A delivery case is a case of the pack | `check.rs` `a_delivery_case_is_a_case_of_the_pack` |
| The pack keeps an index of the statement's clauses | The index read | `tests/clauses.rs` `the_index_holds_every_clause_under_its_section`, against `the_statements_sections` holding A.14 and B.7 |
| The pack's check fails on a counted clause no case cites | A clause of Part A cited by no case (S16) | `check.rs` `a_clause_of_part_a_cited_by_no_case` |
| | A clause of A.14 cited by no case | `check.rs` `a_clause_of_a14_cited_by_no_case` |
| | A clause the check does not count | `check.rs` `a_clause_the_check_does_not_count` |
| | A cadence or branch case counts as any other | `check.rs` `a_cadence_or_branch_case_counts_as_any_other` |
| weir's pack passes its own check | The repository's pack | `tests/the_pack.rs` `the_repositorys_pack`, reading a case of each shape, `delivery` among them |

`tests/clauses.rs` `the_check_counts_a1_to_a10_a14_and_71` holds the
counted sections, and `tests/the_pack.rs`
`the_delivery_cases_cite_100_to_107` the pack's own citations.

## Tasks

All 22 tasks of `tasks.md` are ticked. Each one's condition holds:

- 1.1: `crates/weir-delivery` has its dependencies, its description
  citing 100 to 113 and C.8, `[lints] workspace = true`, its place in
  `Cargo.toml`'s members and crate list, `moon.yml` with four
  `check-fast` tasks over the inputs the task names, and `delivery` in
  `.moon/workspace.yml`. `src/lib.rs` denies `missing_docs`.
  `delivery:typecheck` passed.
- 1.2 to 3.4: each named test binary passed every test the task names.
- 3.5: `delivery:lint` passed, the crate's `missing_docs` denied.
- 4.1 to 4.6: `pack:test` passed every test the tasks name. `Shape::ALL`
  holds seven. weir-pack depends on weir-delivery, and its moon project
  lists `delivery` in `dependsOn` and `/crates/weir-delivery/**/*` in
  its inputs.
- 5.1: the README's table names `weir-delivery`, and `root:prose`
  passed.
