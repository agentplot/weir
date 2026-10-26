//! The adapter shape's cases of lines in and the verdict out, every one
//! run against the checks, held to "An adapter case fixes lines in and the
//! verdict out" and "The adapter checks run from fixed inputs alone"
//! (adapter-checks; 65, 68; C.7).

use std::collections::{BTreeMap, BTreeSet};
use std::fs;

use weir_pack::{read_adapter_case, AdapterCase, Case, LinesCase, Pack, RawPlace, Shape};

/// Every case of lines in the pack, read.
fn lines_cases(pack: &Pack) -> Vec<(Case, LinesCase)> {
    pack.cases(Shape::Adapter)
        .into_iter()
        .filter_map(|case| match read_adapter_case(&case) {
            Ok(AdapterCase::Lines(read)) => Some((case, read)),
            Ok(AdapterCase::BuiltIn(_)) => None,
            Err(why) => panic!("{why}"),
        })
        .collect()
}

/// The clauses a case's `satisfies` cites, as numbers.
fn cited(case: &Case) -> BTreeSet<i64> {
    let table: toml::Table = case.text("case.toml").unwrap().parse().unwrap();
    table["satisfies"]
        .as_array()
        .unwrap()
        .iter()
        .filter_map(toml::Value::as_integer)
        .collect()
}

#[test]
fn every_case_of_lines_gives_its_verdict() {
    let cases = lines_cases(&Pack::embedded());
    assert!(!cases.is_empty());
    let differences: Vec<String> = cases
        .iter()
        .flat_map(|(_, case)| case.differences())
        .collect();
    assert_eq!(differences, Vec::<String>::new());
}

#[test]
fn the_cases_cite_each_of_1_to_12_and_66() {
    let cases = lines_cases(&Pack::embedded());
    let all: BTreeSet<i64> = cases.iter().flat_map(|(case, _)| cited(case)).collect();
    for clause in (1..=12).chain([66]) {
        assert!(all.contains(&clause), "no case of lines cites {clause}");
    }
}

#[test]
fn each_check_is_passed_and_failed_by_a_case() {
    let cases = lines_cases(&Pack::embedded());
    for clause in [1, 2, 3, 4, 5, 6, 7, 9, 12, 66] {
        assert!(
            cases
                .iter()
                .any(|(case, read)| read.breaches.is_empty()
                    && cited(case).contains(&i64::from(clause))),
            "no case citing {clause} passes"
        );
        assert!(
            cases
                .iter()
                .any(|(_, read)| read.breaches.iter().any(|b| b.clause == clause)),
            "no case fails {clause}'s check"
        );
    }
}

/// A case of `toml` and `lines`, read from a pack of its own.
fn a_case(toml: &str, lines: &str) -> Result<LinesCase, String> {
    let pack = Pack::from_files([
        (
            "adapter/made/case.toml".to_string(),
            toml.as_bytes().to_vec(),
        ),
        (
            "adapter/made/lines.jsonl".to_string(),
            lines.as_bytes().to_vec(),
        ),
    ]);
    match read_adapter_case(&pack.case(Shape::Adapter, "made").unwrap())? {
        AdapterCase::Lines(case) => Ok(case),
        AdapterCase::BuiltIn(case) => panic!("a built-in case: {case:?}"),
    }
}

const TWO_LINES: &str = concat!(
    r#"{"capture": {"key": "release/acme/1.0", "source": "release", "event_at": "2026-10-10T14:02:11Z"}}"#,
    "\n",
    r#"{"capture": {"key": "captures/x", "source": "release", "event_at": "2026-10-10T14:02:11Z"}}"#,
    "\n",
);

#[test]
fn a_case_whose_lines_must_fail() {
    let case = a_case(
        "satisfies = [4]\nexited_ok = false\npasses = false\n\n[[breach]]\nline = 2\nclause = 4\nreason_holds = \"captures\"\n",
        TWO_LINES,
    )
    .unwrap();
    assert_eq!(case.differences(), Vec::<String>::new());
}

#[test]
fn a_case_the_checks_answer_otherwise() {
    let case = a_case(
        "satisfies = [4]\nexited_ok = false\npasses = true\n",
        TWO_LINES,
    )
    .unwrap();
    let differences = case.differences();
    assert_eq!(differences.len(), 1, "{differences:?}");
    assert!(
        differences[0].starts_with("adapter/made: the checks name line 2 breaks 4: "),
        "{}",
        differences[0]
    );
    let wrong_phrase = a_case(
        "satisfies = [4]\nexited_ok = false\npasses = false\n\n[[breach]]\nline = 2\nclause = 4\nreason_holds = \"items\"\n",
        TWO_LINES,
    )
    .unwrap();
    let differences = wrong_phrase.differences();
    assert_eq!(differences.len(), 2, "{differences:?}");
    assert!(differences[0].contains("expects line 2 to break 4 with a reason holding `items`"));
}

#[test]
fn a_case_out_of_its_format() {
    let misspelt = a_case(
        "satisfies = [4]\nexited_ok = false\npasses = true\nbreaches = []\n",
        TWO_LINES,
    )
    .unwrap_err();
    assert!(misspelt.starts_with("adapter/made: "), "{misspelt}");
    assert!(misspelt.contains("`breaches`"), "{misspelt}");
    let no_verdict = a_case("satisfies = [4]\nexited_ok = false\n", TWO_LINES).unwrap_err();
    assert!(no_verdict.contains("`passes`"), "{no_verdict}");
    let at_odds = a_case(
        "satisfies = [4]\nexited_ok = false\npasses = false\n",
        TWO_LINES,
    )
    .unwrap_err();
    assert!(at_odds.contains("names 0 breaches"), "{at_odds}");
}

#[test]
fn the_checks_over_one_cases_lines_on_two_machines() {
    let built_in = Pack::embedded();
    let folder = tempfile::tempdir().unwrap();
    for (path, bytes) in built_in.files() {
        let file = folder.path().join(path);
        fs::create_dir_all(file.parent().unwrap()).unwrap();
        fs::write(file, bytes).unwrap();
    }
    let copied = Pack::read(folder.path()).unwrap();
    let breaches = |pack: &Pack| -> BTreeMap<String, Vec<String>> {
        lines_cases(pack)
            .into_iter()
            .map(|(_, case)| {
                let named = case.breaches().iter().map(ToString::to_string).collect();
                (case.id, named)
            })
            .collect()
    };
    assert_eq!(breaches(&built_in), breaches(&copied));

    // A raw place read from a folder names what one read from its files
    // does.
    let (case, read) = lines_cases(&built_in)
        .into_iter()
        .find(|(case, _)| case.name == "raw-material-outside-the-raw-place")
        .unwrap();
    let out = tempfile::tempdir().unwrap();
    case.write_to(out.path()).unwrap();
    let from_folder = weir_pack::check_lines(
        &read.lines,
        RawPlace::Folder(&out.path().join("raw")),
        read.exited_ok,
    );
    assert_eq!(from_folder, read.breaches());
}
