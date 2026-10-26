//! An elaboration kind's section is read only by whoever runs
//! elaborations: a read of the kinds returns it unread and never fails on
//! it, and a read of the elaboration checks it whole (55, 56, 57; S11).

mod common;

use common::read;
use weir_kinds::toml::{Table, Value};
use weir_kinds::{read_elaboration, Elaboration, Ending};

fn strings(items: &[&str]) -> Vec<String> {
    items.iter().map(|item| (*item).to_owned()).collect()
}

#[test]
fn both_readers_read_one_elaboration_kind() {
    let declaration = read("elaboration.toml").unwrap();
    let writeback = &declaration.kinds["writeback"];
    let stages: Vec<_> = writeback.stages.iter().map(|s| s.name.as_str()).collect();
    assert_eq!(stages, ["write"]);
    assert!(writeback.lands);

    let section: Table = "inputs = [\"books/outline.md\"]\n\
                          conclusion = [\"what changes\"]\n\
                          ending = \"self-closing\"\n\
                          transcripts = false\n\
                          lands = [\"books/\", \"design/\"]"
        .parse()
        .unwrap();
    assert_eq!(
        writeback.sections,
        [("elaboration".to_owned(), Value::Table(section))]
            .into_iter()
            .collect::<Table>()
    );

    assert_eq!(
        read_elaboration(writeback),
        Ok(Some(Elaboration {
            inputs: strings(&["books/outline.md"]),
            conclusion: strings(&["what changes"]),
            ending: Some(Ending::SelfClosing),
            transcripts: false,
            lands: strings(&["books/", "design/"]),
            sections: Table::new(),
        }))
    );
}

#[test]
fn a_malformed_elaboration_section() {
    let declaration = read("elaboration-sometimes.toml").unwrap();
    let refusal = read_elaboration(&declaration.kinds["writeback"]).unwrap_err();
    let fields: Vec<_> = refusal.problems.iter().map(|p| p.field.as_str()).collect();
    assert_eq!(fields, ["kinds.writeback.elaboration.ending"]);
    assert_eq!(
        refusal.problems[0].reason,
        "`sometimes` is not one of self-closing, standing and with-operator"
    );
}

#[test]
fn a_kind_with_no_elaboration_section() {
    let declaration = read("stages-only.toml").unwrap();
    assert_eq!(read_elaboration(&declaration.kinds["tidy"]), Ok(None));
}

#[test]
fn every_field_of_the_section_is_checked() {
    let declaration = read("elaboration-shapes.toml").unwrap();
    let fields = |kind: &str| -> Vec<String> {
        let refusal = read_elaboration(&declaration.kinds[kind]).unwrap_err();
        refusal.problems.into_iter().map(|p| p.field).collect()
    };
    assert_eq!(fields("scalar"), ["kinds.scalar.elaboration"]);
    assert_eq!(
        fields("shapes"),
        [
            "kinds.shapes.elaboration.inputs",
            "kinds.shapes.elaboration.conclusion",
            "kinds.shapes.elaboration.ending",
            "kinds.shapes.elaboration.transcripts",
            "kinds.shapes.elaboration.lands",
        ]
    );
}

#[test]
fn each_ending_reads_as_its_form() {
    let declaration = read("endings.toml").unwrap();
    let ending = |kind: &str| {
        read_elaboration(&declaration.kinds[kind])
            .unwrap()
            .unwrap()
            .ending
    };
    assert_eq!(ending("closing"), Some(Ending::SelfClosing));
    assert_eq!(ending("standing"), Some(Ending::Standing));
    assert_eq!(ending("operator"), Some(Ending::WithOperator));
    assert_eq!(ending("open"), None);

    let open = read_elaboration(&declaration.kinds["open"])
        .unwrap()
        .unwrap();
    assert!(open.inputs.is_empty() && open.conclusion.is_empty() && open.lands.is_empty());
    assert!(!open.transcripts);
    assert_eq!(open.sections, "rounds = 4".parse::<Table>().unwrap());

    let spellings: Vec<_> = [Ending::SelfClosing, Ending::Standing, Ending::WithOperator]
        .iter()
        .map(Ending::to_string)
        .collect();
    assert_eq!(spellings, ["self-closing", "standing", "with-operator"]);
}
