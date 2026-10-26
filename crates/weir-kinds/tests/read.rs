//! A declaration reads into its kinds and their stages, each deliverable
//! reads as its form, and a section the reader does not know never fails
//! the read (51 to 54, 56; C.4).

mod common;

use std::time::Duration;

use common::read;
use weir_kinds::toml::Table;
use weir_kinds::{Deliverable, End};

fn table(text: &str) -> Table {
    text.parse().unwrap()
}

// A declaration reads into its kinds.

#[test]
fn a_kind_reads_with_every_field() {
    let declaration = read("bump.toml").unwrap();
    assert_eq!(declaration.kinds.keys().collect::<Vec<_>>(), ["bump"]);
    let bump = &declaration.kinds["bump"];
    assert_eq!(bump.name, "bump");
    assert_eq!(bump.version, 2);
    assert_eq!(bump.branch, "bump");
    assert!(!bump.starts_unready);
    assert!(bump.lands);
    let stages: Vec<_> = bump.stages.iter().map(|s| s.name.as_str()).collect();
    assert_eq!(stages, ["code", "review"]);
    assert!(bump.sections.is_empty());
    assert!(declaration.sections.is_empty());
}

#[test]
fn fields_left_out_read_as_the_format_says() {
    let declaration = read("stages-only.toml").unwrap();
    let tidy = &declaration.kinds["tidy"];
    assert_eq!(tidy.version, 1);
    assert_eq!(tidy.branch, "tidy");
    assert!(!tidy.starts_unready);
    assert!(!tidy.lands);
    let code = &tidy.stages[0];
    assert!(code.agents.is_empty());
    assert!(!code.wait);
    assert_eq!(code.quiet, None);
    assert_eq!(code.on_fail, None);
    assert_eq!(code.max_send_backs, None);
}

#[test]
fn a_declaration_with_no_kinds() {
    let declaration = read("no-kinds.toml").unwrap();
    assert!(declaration.kinds.is_empty());
    assert!(declaration.sections.is_empty());
}

// A stage reads with its name, prompt and end.

#[test]
fn a_stage_reads_with_its_settings() {
    let bump = &read("bump.toml").unwrap().kinds["bump"];
    let review = &bump.stages[1];
    assert_eq!(review.name, "review");
    assert_eq!(review.prompt, "prompts/bump-review.md");
    assert_eq!(review.end, End::Conversation);
    assert!(review.wait);
    assert_eq!(review.agents, [table(r#"beside = "operator""#)]);
    assert_eq!(review.on_fail.as_deref(), Some("code"));
    assert_eq!(review.max_send_backs, Some(3));

    let code = &bump.stages[0];
    assert_eq!(code.prompt, "prompts/bump-code.md");
    assert_eq!(code.end, End::Deliverable(Deliverable::Commits));
    assert_eq!(
        code.agents,
        [table("program = \"claude\"\nmodel = \"opus\"")]
    );
    assert!(code.sections.is_empty());
}

#[test]
fn a_stages_quiet_time_reads_as_a_duration() {
    let bump = &read("bump.toml").unwrap().kinds["bump"];
    assert_eq!(bump.stages[0].quiet, Some(Duration::from_secs(45 * 60)));
    assert_eq!(bump.stages[1].quiet, Some(Duration::from_secs(90 * 60)));
}

// A deliverable is one of six spellings.

#[test]
fn each_spelling_reads_as_its_form() {
    let every = &read("six-deliverables.toml").unwrap().kinds["every"];
    let ends: Vec<_> = every.stages.iter().map(|s| s.end.clone()).collect();
    assert_eq!(
        ends,
        [
            End::Deliverable(Deliverable::Commits),
            End::Deliverable(Deliverable::Change),
            End::Deliverable(Deliverable::Tasks),
            End::Deliverable(Deliverable::Archived),
            End::Deliverable(Deliverable::Report("out/conclusion.md".to_owned())),
            End::Deliverable(Deliverable::Committed("docs/verify.md".to_owned())),
        ]
    );
    let spellings: Vec<_> = ends
        .iter()
        .map(|end| match end {
            End::Deliverable(deliverable) => deliverable.to_string(),
            End::Conversation => panic!("no stage here is a conversation"),
        })
        .collect();
    assert_eq!(
        spellings,
        [
            "commits",
            "change",
            "tasks",
            "archived",
            "report:out/conclusion.md",
            "committed:docs/verify.md",
        ]
    );
}

// A section the reader does not know never fails the read.

#[test]
fn a_tools_own_table_beside_the_kinds() {
    let with = read("profiles.toml").unwrap();
    let without = read("stages-only.toml").unwrap();
    assert_eq!(with.kinds, without.kinds);
    assert_eq!(
        with.sections,
        table("[profiles.fast]\nquiet = 45\nstages = \"none\"")
    );
}

#[test]
fn an_unknown_field_on_a_stage() {
    let tidy = &read("unknown-field.toml").unwrap().kinds["tidy"];
    let code = &tidy.stages[0];
    assert_eq!(code.name, "code");
    assert_eq!(code.end, End::Deliverable(Deliverable::Commits));
    assert_eq!(code.sections, table("colour = 3"));
    assert_eq!(tidy.sections, table(r#"owner = "docs""#));
}
