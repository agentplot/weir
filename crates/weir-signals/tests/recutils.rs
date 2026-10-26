//! The tests' recutils helper: what it holds to `recfix` and reads with
//! `recsel` is GNU recutils' own reading (core.24).

mod common;

use common::{recfix, recsel_value, run};

#[test]
fn a_record_passes_recfix_and_reads_back_through_recsel() {
    let text = "format: weir-move/1\ntarget: two lines\n+ of reason\n";
    recfix(text);
    assert_eq!(recsel_value(text, "target"), "two lines\nof reason");
}

#[test]
#[should_panic(expected = "recfix refused")]
fn a_text_recfix_refuses_fails_the_test() {
    recfix("format: weir-move/1\nno colon here\n");
}

#[test]
#[should_panic(expected = "is not on PATH")]
fn a_program_not_on_the_path_fails_the_test() {
    run("recfix-is-not-this", &[], "");
}
