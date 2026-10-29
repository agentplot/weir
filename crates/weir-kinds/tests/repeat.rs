//! A stage reads whether it repeats per task group, and a repeat the format
//! does not allow is refused naming the field (53, 57; C.4).

mod common;

use common::read;
use weir_kinds::{Deliverable, End, Problem, Repeat};

// A stage reads whether it repeats per task group.

#[test]
fn a_stage_that_repeats_per_task_group() {
    let change = &read("repeat.toml").unwrap().kinds["change"];
    let repeats: Vec<_> = change
        .stages
        .iter()
        .map(|s| (s.name.as_str(), s.repeat))
        .collect();
    assert_eq!(
        repeats,
        [
            ("construct", None),
            ("code", Some(Repeat::PerTaskGroup)),
            ("verify", None),
        ]
    );
    let code = &change.stages[1];
    assert_eq!(code.end, End::Deliverable(Deliverable::Tasks));
    assert!(code.sections.is_empty());
}

#[test]
fn the_spelling_reads_back() {
    assert_eq!(Repeat::PerTaskGroup.to_string(), "per-task-group");
}

#[test]
fn a_stage_that_declares_no_repeat_repeats_not() {
    let code = &read("stages-only.toml").unwrap().kinds["tidy"].stages[0];
    assert_eq!(code.repeat, None);
}

// A repeat the format does not allow is refused.

#[test]
fn another_value_or_shape_of_repeat_is_refused() {
    let refusal = read("repeat-refused.toml").unwrap_err();
    let fields: Vec<_> = refusal.problems.iter().map(|p| p.field.as_str()).collect();
    assert_eq!(
        fields,
        [
            "kinds.a-flag.stages[0].repeat",
            "kinds.a-list.stages[0].repeat",
            "kinds.another-value.stages[0].repeat",
            "kinds.commits.stages[0].repeat",
            "kinds.conversation.stages[0].repeat",
        ]
    );
    assert_eq!(
        refusal.problems[2].reason,
        "`per-task` is not per-task-group, the one way a stage repeats"
    );
    assert_eq!(
        refusal.problems[0].reason,
        "must be the string \"per-task-group\""
    );
}

#[test]
fn repeat_on_a_stage_that_does_not_deliver_tasks() {
    let refusal = read("repeat-refused.toml").unwrap_err();
    let reason = |field: &str| {
        refusal
            .problems
            .iter()
            .find(|p| p.field == field)
            .map(|p| p.reason.clone())
            .unwrap()
    };
    assert_eq!(
        reason("kinds.commits.stages[0].repeat"),
        "is on a stage that delivers commits, and only a stage that delivers tasks repeats per task group"
    );
    assert_eq!(
        reason("kinds.conversation.stages[0].repeat"),
        "is on a stage that is a conversation, and only a stage that delivers tasks repeats per task group"
    );
}

#[test]
fn repeat_beside_an_end_already_refused() {
    let refusal = read("repeat-beside-a-refused-end.toml").unwrap_err();
    assert_eq!(
        refusal.problems,
        [Problem {
            field: "kinds.bump.stages[0].deliverables".to_owned(),
            reason: "`commit` is not one of commits, change, tasks, archived, \
                     report:<path> and committed:<path>"
                .to_owned(),
        }]
    );
}
