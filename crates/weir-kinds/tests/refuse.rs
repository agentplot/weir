//! A declaration is checked whole and refused naming every field that
//! fails it, a deliverable not of the six spellings among them (51, 52,
//! 54, 57, 81; S12).

mod common;

use common::read;
use weir_kinds::{read_kinds, Problem, Refusal};

/// The fields the fixture `name` is refused naming, in the order named.
fn fields(name: &str) -> Vec<String> {
    let refusal = read(name).expect_err("the declaration is refused");
    refusal.problems.into_iter().map(|p| p.field).collect()
}

// A deliverable is one of six spellings.

#[test]
fn two_deliverables_are_refused() {
    assert_eq!(
        read("two-deliverables.toml"),
        Err(Refusal {
            problems: vec![Problem {
                field: "kinds.bump.stages[0].deliverables".to_owned(),
                reason: "names 2 deliverables, and a stage has exactly one".to_owned(),
            }],
        })
    );
}

#[test]
fn a_spelling_not_of_the_six_is_refused() {
    for fixture in ["unknown-spelling.toml", "report-without-path.toml"] {
        assert_eq!(
            fields(fixture),
            ["kinds.bump.stages[0].deliverables"],
            "{fixture}"
        );
    }
    let reason = &read("report-without-path.toml").unwrap_err().problems[0].reason;
    assert_eq!(reason, "`report:` names no path");
}

#[test]
fn a_stage_with_neither_a_deliverable_nor_a_conversation() {
    assert_eq!(
        fields("no-deliverable.toml"),
        ["kinds.bump.stages[0].deliverables"]
    );
}

#[test]
fn a_conversation_with_a_deliverable() {
    assert_eq!(
        fields("conversation-with-deliverable.toml"),
        ["kinds.bump.stages[0].deliverables"]
    );
}

// A declaration is checked whole and refused naming the field.

#[test]
fn a_kind_with_no_stage_is_refused() {
    assert_eq!(
        fields("no-stages.toml"),
        ["kinds.bare.stages", "kinds.empty.stages"]
    );
}

#[test]
fn a_stage_with_no_prompt_is_refused() {
    assert_eq!(fields("no-prompt.toml"), ["kinds.bump.stages[1].prompt"]);
}

#[test]
fn a_field_of_the_wrong_shape_is_refused() {
    assert_eq!(
        fields("wrong-shapes.toml"),
        [
            "kinds.lands-yes.lands",
            "kinds.quiet-number.stages[0].quiet",
            "kinds.version-two.version",
            "kinds.version-zero.version",
        ]
    );
}

#[test]
fn every_other_shape_the_format_names_is_checked() {
    assert_eq!(
        fields("more-shapes.toml"),
        [
            "kinds.branch-empty.branch",
            "kinds.branch-empty.starts_unready",
            "kinds.loose",
            "kinds.stage-shapes.stages[0].name",
            "kinds.stage-shapes.stages[0].prompt",
            "kinds.stage-shapes.stages[0].deliverables",
            "kinds.stage-shapes.stages[0].agents",
            "kinds.stage-shapes.stages[0].wait",
            "kinds.stage-shapes.stages[0].quiet",
            "kinds.stage-shapes.stages[0].on_fail",
            "kinds.stage-shapes.stages[0].max_send_backs",
            "kinds.stage-shapes.stages[1].conversation",
            "kinds.stage-shapes.stages[2].deliverables",
            "kinds.stages-strings.stages",
        ]
    );
    assert_eq!(fields("kinds-not-a-table.toml"), ["kinds"]);
}

#[test]
fn a_send_back_naming_no_stage_of_its_kind_is_refused() {
    let refusal = read("send-back-to-nowhere.toml").unwrap_err();
    assert_eq!(
        refusal.problems,
        [Problem {
            field: "kinds.bump.stages[0].on_fail".to_owned(),
            reason: "`deploy` names no stage of this kind".to_owned(),
        }]
    );
}

#[test]
fn two_stages_of_one_kind_share_a_name() {
    let refusal = read("duplicate-stage.toml").unwrap_err();
    assert_eq!(
        refusal.problems,
        [Problem {
            field: "kinds.bump.stages[1].name".to_owned(),
            reason: "`code` is the name of stages[0] too".to_owned(),
        }]
    );
}

#[test]
fn every_failing_field_is_named() {
    // A refusal is the Err of the read: no kind comes back beside it.
    assert_eq!(
        fields("two-faults.toml"),
        ["kinds.bare.stages", "kinds.bump.stages[1].prompt"]
    );
}

#[test]
fn text_that_is_not_toml_is_refused() {
    let refusal = read_kinds("[[kinds.bump.stages]]\nname = \"code\"\nprompt = \n").unwrap_err();
    assert_eq!(refusal.problems.len(), 1);
    let problem = &refusal.problems[0];
    assert_eq!(problem.field, "line 3, column 10");
    assert!(!problem.reason.is_empty());
}

#[test]
fn a_key_a_path_cannot_hold_bare_is_quoted() {
    let refusal = read_kinds("[kinds.\"a.b\"]\nversion = 0\n").unwrap_err();
    let fields: Vec<_> = refusal.problems.iter().map(|p| p.field.as_str()).collect();
    assert_eq!(fields, ["kinds.\"a.b\".version", "kinds.\"a.b\".stages"]);
}

#[test]
fn a_refusal_reads_one_problem_a_line() {
    let refusal = read("two-faults.toml").unwrap_err();
    assert_eq!(
        refusal.to_string(),
        "kinds.bare.stages: is missing, and a kind has at least one stage\n\
         kinds.bump.stages[1].prompt: is missing, and every stage has a prompt"
    );
}
