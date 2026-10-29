//! A kind's agents table, for every stage and for stages it names, and a
//! kind's table that holds agents and declares no kind, read apart from the
//! kinds as settings given to a kind declared elsewhere (52, 57, 58; C.4).

mod common;

use std::collections::BTreeMap;

use common::read;
use weir_kinds::toml::Table;
use weir_kinds::KindAgents;

fn table(text: &str) -> Table {
    text.parse().unwrap()
}

// A kind's agents table reads with the kind.

#[test]
fn a_kinds_agents_table_reads_for_every_stage_and_for_one() {
    let declaration = read("kind-agents.toml").unwrap();
    assert!(declaration.settings.is_empty());
    let bump = &declaration.kinds["bump"];
    assert_eq!(
        bump.agents.every_stage,
        table(
            "program = \"claude\"\nmodel = \"sonnet\"\n\
             instructions = \"Keep each commit to one change.\"\n\
             env = { ACCOUNT = \"work\" }"
        )
    );
    assert_eq!(
        bump.agents.stages,
        BTreeMap::from([("review".to_owned(), table(r#"model = "opus""#))])
    );
    assert!(bump.sections.is_empty());
}

#[test]
fn a_stages_own_agents_read_beside_the_kinds() {
    let bump = &read("kind-agents.toml").unwrap().kinds["bump"];
    assert_eq!(bump.stages[0].agents, [table(r#"model = "haiku""#)]);
    assert!(bump.stages[1].agents.is_empty());
}

#[test]
fn an_inline_agents_table_reads_as_the_table_does() {
    let tidy = &read("kind-agents.toml").unwrap().kinds["tidy"];
    assert_eq!(tidy.agents.every_stage, table(r#"model = "haiku""#));
    assert!(tidy.agents.stages.is_empty());
}

#[test]
fn a_kind_with_no_agents_table_reads_with_none() {
    let tidy = &read("stages-only.toml").unwrap().kinds["tidy"];
    assert_eq!(tidy.agents, KindAgents::default());
    assert!(tidy.agents.is_empty());
}

// A kind's table holding agents and no stages gives settings.

#[test]
fn a_table_holding_agents_and_no_stages_declares_no_kind() {
    let declaration = read("settings-only.toml").unwrap();
    assert_eq!(declaration.kinds.keys().collect::<Vec<_>>(), ["tidy"]);
    assert_eq!(
        declaration.settings.keys().collect::<Vec<_>>(),
        ["change", "fix", "review"]
    );
}

#[test]
fn settings_read_for_every_stage_and_for_stages_named() {
    let settings = read("settings-only.toml").unwrap().settings;
    let change = &settings["change"];
    assert_eq!(change.name, "change");
    assert_eq!(
        change.agents.every_stage,
        table("model = \"sonnet\"\ninstructions = \"Work in British English.\"")
    );
    assert_eq!(
        change.agents.stages,
        BTreeMap::from([("verify".to_owned(), table(r#"model = "opus""#))])
    );
    assert!(change.sections.is_empty());

    let review = &settings["review"];
    assert!(review.agents.every_stage.is_empty());
    assert_eq!(
        review.agents.stages,
        BTreeMap::from([("read".to_owned(), table(r#"model = "haiku""#))])
    );
}

#[test]
fn settings_carry_every_field_as_written() {
    // The environment is returned as written: whether a declaration may
    // give one is its reader's rule (58).
    let fix = &read("settings-only.toml").unwrap().settings["fix"];
    assert_eq!(
        fix.agents.every_stage,
        table("program = \"claude\"\nenv = { ACCOUNT = \"work\" }")
    );
    assert_eq!(fix.sections, table(r#"owner = "docs""#));
}

#[test]
fn a_table_with_neither_agents_nor_stages_is_still_a_kind_with_no_stage() {
    let refusal = weir_kinds::read_kinds("[kinds.tidy]\nowner = \"docs\"\n").unwrap_err();
    let fields: Vec<_> = refusal.problems.iter().map(|p| p.field.as_str()).collect();
    assert_eq!(fields, ["kinds.tidy.stages"]);
}

// Each table of the wrong shape is refused naming the field.

#[test]
fn agents_tables_of_the_wrong_shape_are_refused() {
    let refusal = read("agents-shapes.toml").unwrap_err();
    let fields: Vec<_> = refusal.problems.iter().map(|p| p.field.as_str()).collect();
    assert_eq!(
        fields,
        [
            "kinds.agents-array.agents",
            "kinds.elaboration-agents.stages",
            "kinds.makes-shapes.stages[0].makes",
            "kinds.makes-shapes.stages[1].makes",
            "kinds.makes-shapes.stages[2].makes",
            "kinds.settings-number.agents",
            "kinds.stage-string.agents.stages.code",
            "kinds.stages-number.agents.stages",
        ]
    );
}
