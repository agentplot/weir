//! The pack's check, over packs built in temporary folders, held to the
//! scenarios of conformance-pack: one folder per case under its shape, the
//! index, a number naming no clause and a counted clause no case cites
//! (S16), and the same verdict from fixed inputs alone (67, 68; core.35).

use std::fs;
use std::path::Path;

use weir_pack::{check, Clause, Fault, Pack, Shape};

/// A small index: one clause or a few in each counted section, the pack's
/// own and the never-does clauses, and a section of Part B.
const INDEX: &str = r#"statement = "agentplot/blueprints:design/weir/requirements.md"
commit = "ae9d782"

[[section]]
name = "A.1"
clauses = [1, 2]

[[section]]
name = "A.2"
clauses = [14]

[[section]]
name = "A.3"
clauses = [26]

[[section]]
name = "A.4"
clauses = [33]

[[section]]
name = "A.5"
clauses = [35]

[[section]]
name = "A.6"
clauses = [37]

[[section]]
name = "A.7"
clauses = [48]

[[section]]
name = "A.8"
clauses = [57]

[[section]]
name = "A.9"
clauses = [60]

[[section]]
name = "A.10"
clauses = [61]

[[section]]
name = "A.11"
clauses = [65, 66, 67, 68]

[[section]]
name = "A.12"
clauses = [69, 70, 71]

[[section]]
name = "B.2"
clauses = [75, "75a"]

[[section]]
name = "B.6"
clauses = [87]
"#;

/// A pack in a temporary folder of `files`, each `(path, text)`.
fn pack_of(files: &[(&str, &str)]) -> (tempfile::TempDir, Pack) {
    let dir = tempfile::tempdir().unwrap();
    write_all(dir.path(), files);
    let pack = Pack::read(dir.path()).unwrap();
    (dir, pack)
}

fn write_all(root: &Path, files: &[(&str, &str)]) {
    for (path, text) in files {
        let file = root.join(path);
        fs::create_dir_all(file.parent().unwrap()).unwrap();
        fs::write(file, text).unwrap();
    }
}

/// A pack that passes: every counted clause cited, 60 only by a cadence
/// case and 48 only by a branch case, `"75a"` cited, 69 cited by none and
/// 87 by one.
fn passing() -> Vec<(&'static str, &'static str)> {
    vec![
        ("clauses.toml", INDEX),
        ("host/held-key/case.toml", "satisfies = [14, 71, 87]\n"),
        (
            "host/held-key/start/inbox/x.rec",
            "format: weir-capture/1\n",
        ),
        ("adapter/two-offers/case.toml", "satisfies = [1, 2, 61]\n"),
        ("cadence/slept/case.toml", "satisfies = [60]\n"),
        ("branch/no-remote/case.toml", "satisfies = [48]\n"),
        (
            "inbox/import/case.toml",
            "satisfies = [26, 33, 35, 37, \"75a\"]\n",
        ),
        ("kinds/two-deliverables/case.toml", "satisfies = [57]\n"),
    ]
}

/// `files` with the file at `path` given `text`, or taken out when `text`
/// is none.
fn with(
    files: &[(&'static str, &'static str)],
    path: &'static str,
    text: Option<&'static str>,
) -> Vec<(&'static str, &'static str)> {
    let mut changed: Vec<_> = files.iter().filter(|(p, _)| *p != path).copied().collect();
    if let Some(text) = text {
        changed.push((path, text));
    }
    changed
}

fn faults(files: &[(&str, &str)]) -> Vec<String> {
    let (_dir, pack) = pack_of(files);
    match check(&pack) {
        Ok(checked) => panic!("passed: {checked}"),
        Err(faults) => faults.iter().map(Fault::to_string).collect(),
    }
}

#[test]
fn a_pack_that_passes_says_how_many_cases_of_each_shape() {
    let (_dir, pack) = pack_of(&passing());
    let checked = check(&pack).unwrap_or_else(|faults| panic!("{faults:?}"));
    for shape in Shape::ALL {
        assert_eq!(checked.read[&shape], 1, "{shape}");
    }
    assert_eq!(
        checked.to_string(),
        "the pack passed, its cases read: 1 host, 1 adapter, 1 cadence, 1 branch, 1 inbox, 1 kinds"
    );
}

#[test]
fn a_folder_of_no_shape() {
    let files = with(
        &passing(),
        "hosts/held-key/case.toml",
        Some("satisfies = [14]\n"),
    );
    assert_eq!(
        faults(&files),
        ["pack/hosts: is no shape of the pack, which are host, adapter, cadence, branch, inbox, kinds"]
    );
    let stray = with(&passing(), "README.md", Some("# The pack\n"));
    assert_eq!(faults(&stray).len(), 1);
    assert!(faults(&stray)[0].starts_with("pack/README.md: is no shape"));
}

#[test]
fn a_case_that_is_no_folder() {
    let files = with(
        &passing(),
        "cadence/never-run.toml",
        Some("satisfies = [59]\n"),
    );
    assert_eq!(
        faults(&files),
        ["pack/cadence/never-run.toml: lies in its shape's folder, and every case is a folder there"]
    );
    let no_case_file = with(&passing(), "host/held-key/case.toml", None);
    assert!(faults(&no_case_file).contains(&"pack/host/held-key: holds no case.toml".to_string()));
}

#[test]
fn a_case_that_cites_nothing() {
    for text in ["schedule = \"0 * * * *\"\n", "satisfies = []\n"] {
        let files = with(&passing(), "kinds/two-deliverables/case.toml", Some(text));
        let said = faults(&files);
        assert!(
            said.contains(&"pack/kinds/two-deliverables: cites no clause, and every case names in `satisfies` the clauses it exercises".to_string()),
            "{said:?}"
        );
    }
    let not_toml = with(
        &passing(),
        "kinds/two-deliverables/case.toml",
        Some("satisfies = [57"),
    );
    assert!(faults(&not_toml)[0].starts_with("pack/kinds/two-deliverables/case.toml: "));
    let not_a_list = with(
        &passing(),
        "kinds/two-deliverables/case.toml",
        Some("satisfies = 57\n"),
    );
    assert!(faults(&not_a_list).contains(
        &"pack/kinds/two-deliverables: `satisfies`: is not a list of clauses".to_string()
    ));
}

#[test]
fn a_clause_with_a_letter() {
    let (_dir, pack) = pack_of(&passing());
    assert!(check(&pack).is_ok());
    let files = with(
        &passing(),
        "inbox/import/case.toml",
        Some("satisfies = [26, 33, 35, 37, \"75b\"]\n"),
    );
    assert_eq!(
        faults(&files),
        ["pack/inbox/import: cites 75b, which names no clause of the statement"]
    );
    let not_a_number = with(
        &passing(),
        "inbox/import/case.toml",
        Some("satisfies = [26, 33, 35, 37, \"seventy-five\"]\n"),
    );
    assert_eq!(faults(&not_a_number).len(), 1);
}

#[test]
fn a_case_cites_a_clause_that_names_nothing() {
    let files = with(
        &passing(),
        "host/held-key/case.toml",
        Some("satisfies = [14, 140, 71]\n"),
    );
    assert_eq!(
        faults(&files),
        ["pack/host/held-key: cites 140, which names no clause of the statement"]
    );
}

#[test]
fn a_clause_of_part_a_cited_by_no_case() {
    let files = with(
        &passing(),
        "inbox/import/case.toml",
        Some("satisfies = [26, 35, 37]\n"),
    );
    assert_eq!(faults(&files), ["33: is cited by no case"]);
    let no_71 = with(
        &passing(),
        "host/held-key/case.toml",
        Some("satisfies = [14]\n"),
    );
    assert_eq!(faults(&no_71), ["71: is cited by no case"]);
}

#[test]
fn a_clause_the_check_does_not_count() {
    // No case cites 65, 66, 67, 68, 69 or 70, and the held-key case cites
    // 87; the passing pack passes all the same.
    let (_dir, pack) = pack_of(&passing());
    assert!(check(&pack).is_ok());
    let files = with(
        &passing(),
        "host/held-key/case.toml",
        Some("satisfies = [14, 71, 65, 69, 87]\n"),
    );
    let (_dir, pack) = pack_of(&files);
    assert!(check(&pack).is_ok());
}

#[test]
fn a_cadence_or_branch_case_counts_as_any_other() {
    let without_cadence = with(&passing(), "cadence/slept/case.toml", None);
    assert_eq!(faults(&without_cadence), ["60: is cited by no case"]);
    let without_branch = with(&passing(), "branch/no-remote/case.toml", None);
    assert_eq!(faults(&without_branch), ["48: is cited by no case"]);
}

#[test]
fn every_fault_is_named_at_once() {
    let mut files = with(
        &passing(),
        "host/held-key/case.toml",
        Some("satisfies = [140]\n"),
    );
    files.push(("hosts/x/case.toml", "satisfies = [14]\n"));
    let said = faults(&files);
    assert_eq!(
        said,
        [
            "pack/hosts: is no shape of the pack, which are host, adapter, cadence, branch, inbox, kinds",
            "pack/host/held-key: cites 140, which names no clause of the statement",
            "14: is cited by no case",
            "71: is cited by no case",
        ]
    );
}

#[test]
fn a_pack_with_no_index() {
    let files = with(&passing(), "clauses.toml", None);
    assert_eq!(faults(&files), ["pack/clauses.toml: is missing"]);
    let lacking = with(
        &passing(),
        "clauses.toml",
        Some("statement = \"s\"\ncommit = \"c\"\n[[section]]\nname = \"A.1\"\nclauses = [1, 2]\n"),
    );
    assert!(faults(&lacking).contains(&"pack/clauses.toml: holds no section A.2".to_string()));
}

#[test]
fn the_same_pack_on_two_machines() {
    let files = with(
        &passing(),
        "host/held-key/case.toml",
        Some("satisfies = [14, 140]\n"),
    );
    let (_one, first) = pack_of(&files);
    let (_two, second) = pack_of(&files);
    assert_eq!(check(&first), check(&second));
    let (_one, first) = pack_of(&passing());
    let (_two, second) = pack_of(&passing());
    assert_eq!(check(&first), check(&second));
    assert_eq!(Clause::number(14).to_string(), "14");
}
