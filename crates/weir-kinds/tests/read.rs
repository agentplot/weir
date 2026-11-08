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
}

#[test]
fn a_declaration_with_no_kinds() {
    let declaration = read("no-kinds.toml").unwrap();
    assert!(declaration.kinds.is_empty());
    assert!(declaration.sections.is_empty());
}

#[test]
fn a_kind_that_lands_with_no_stages() {
    // 51 and 57 as amended at blueprints 51b6316.
    let declaration = read("lands-with-no-stages.toml").unwrap();
    assert_eq!(declaration.kinds.keys().collect::<Vec<_>>(), ["train"]);
    assert!(declaration.settings.is_empty());
    let train = &declaration.kinds["train"];
    assert_eq!(train.name, "train");
    assert_eq!(train.version, 1);
    assert_eq!(train.branch, "train");
    assert!(train.lands);
    assert!(train.stages.is_empty());
    assert!(train.sections.is_empty());
}

#[test]
fn a_kind_with_no_stages_holds_what_any_kind_holds() {
    // What its version, `starts_unready` and `agents` mean for a kind that
    // runs no agent is its reader's (51, 58; core.32).
    let declaration = read("no-stages-holds-the-rest.toml").unwrap();
    assert!(declaration.settings.is_empty());
    let train = &declaration.kinds["train"];
    assert_eq!(train.version, 2);
    assert_eq!(train.branch, "train");
    assert!(train.starts_unready);
    assert!(train.lands);
    assert!(train.stages.is_empty());
    assert_eq!(train.agents.every_stage, table(r#"model = "opus""#));
    assert!(train.agents.stages.is_empty());
    assert_eq!(train.sections, table(r#"owner = "release""#));
}

// A kind that lands with no stages may name its landing fix and its bound.
// A test named for a scenario keeps its name, the fix in it the landing fix.

#[test]
fn a_kind_that_lands_with_no_stages_names_its_fix() {
    // 52, 57 and C.4 as amended at blueprints 8eec6dd; core.42; record 0005.
    let declaration = read("names-its-fix.toml").unwrap();
    assert_eq!(declaration.kinds.keys().collect::<Vec<_>>(), ["train"]);
    assert!(declaration.settings.is_empty());
    let train = &declaration.kinds["train"];
    assert_eq!(train.version, 1);
    assert!(train.lands);
    assert!(train.stages.is_empty());
    let landing_fix = train.landing_fix.as_ref().unwrap();
    assert_eq!(
        (
            landing_fix.to_string(),
            landing_fix.source(),
            landing_fix.name(),
            landing_fix.version(),
        ),
        ("acme/fix@1".to_owned(), "acme", "fix", 1)
    );
    assert_eq!(train.max_landing_fixes, Some(2));
    assert!(train.sections.is_empty());
}

#[test]
fn a_fix_and_its_bound_left_out_are_returned_left_out() {
    // The format gives neither a default (58, C.4).
    let train = &read("lands-with-no-stages.toml").unwrap().kinds["train"];
    assert!(train.stages.is_empty());
    assert_eq!(train.landing_fix, None);
    assert_eq!(train.max_landing_fixes, None);
}

#[test]
fn a_bound_without_a_fix() {
    let train = &read("a-bound-without-a-fix.toml").unwrap().kinds["train"];
    assert_eq!(train.max_landing_fixes, Some(0));
    assert_eq!(train.landing_fix, None);
    assert!(train.sections.is_empty());
}

#[test]
fn the_fields_old_spellings_are_sections() {
    // `fix` and `max_fixes` are no fields the format names, so each is a
    // section, unread and unrefused (56; C.4 as amended at blueprints
    // 8eec6dd; core.30).
    let train = &read("the-fields-old-spellings.toml").unwrap().kinds["train"];
    assert!(train.stages.is_empty());
    assert_eq!(train.landing_fix, None);
    assert_eq!(train.max_landing_fixes, None);
    assert_eq!(train.sections, table("fix = \"fix\"\nmax_fixes = 2"));
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
fn settings_left_out_are_returned_left_out() {
    let code = &read("stages-only.toml").unwrap().kinds["tidy"].stages[0];
    assert_eq!(code.quiet, None);
    assert_eq!(code.on_fail, None);
    assert_eq!(code.max_send_backs, None);
    assert_eq!(code.repeat, None);
}

// A stage names what it makes besides its deliverable.

#[test]
fn a_stage_reads_what_it_makes() {
    let spike = &read("makes.toml").unwrap().kinds["spike"];
    assert_eq!(spike.stages[0].makes, ["preview", "page"]);
    assert!(spike.stages[0].sections.is_empty());
}

#[test]
fn a_stage_that_names_nothing_it_makes_makes_nothing() {
    let spike = &read("makes.toml").unwrap().kinds["spike"];
    assert!(spike.stages[1].makes.is_empty());
    let code = &read("stages-only.toml").unwrap().kinds["tidy"].stages[0];
    assert!(code.makes.is_empty());
}

// A stage reads its standing instructions' path as written.

#[test]
fn a_stage_reads_its_standing_instructions() {
    let bump = &read("bump.toml").unwrap().kinds["bump"];
    assert_eq!(
        bump.stages[0].standing.as_deref(),
        Some("prompts/bump-code-standing.md")
    );
    assert!(bump.stages[0].sections.is_empty());
    assert_eq!(bump.stages[1].standing, None);
    let code = &read("stages-only.toml").unwrap().kinds["tidy"].stages[0];
    assert_eq!(code.standing, None);
}

// A stage's quiet time is one whole number of s, m or h.

#[test]
fn a_stages_quiet_time_reads_as_a_duration() {
    let wait = &read("quiet-times.toml").unwrap().kinds["wait"];
    let quiet: Vec<_> = wait.stages.iter().map(|s| s.quiet).collect();
    assert_eq!(
        quiet,
        [
            Some(Duration::from_secs(45 * 60)),
            Some(Duration::from_secs(90)),
            Some(Duration::from_secs(2 * 60 * 60)),
        ]
    );
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
