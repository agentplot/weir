//! The pack's cases of the two built-in adapters, each run as a host runs
//! the adapter, its lines compared with those the case expects and held to
//! the adapter checks, held to "The built-in adapters pass the checks on
//! the pack's cases" (adapter-checks; 61, 62, 64; C.7; core.34, core.36).

use std::collections::BTreeSet;
use std::fs::{self, File};
use std::path::{Path, PathBuf};
use std::time::SystemTime;

use time::format_description::well_known::Rfc3339;
use time::OffsetDateTime;
use weir_adapters::{folder, meeting};
use weir_capture::{read_line, Line};
use weir_pack::{
    check_lines, read_adapter_case, satisfies, AdapterCase, BuiltIn, BuiltInCase, Clause, Pack,
    RawPlace, Shape,
};

/// Every case of a built-in adapter in the pack, read.
fn built_in_cases() -> Vec<BuiltInCase> {
    Pack::embedded()
        .cases(Shape::Adapter)
        .iter()
        .filter_map(|case| match read_adapter_case(case) {
            Ok(AdapterCase::BuiltIn(read)) => Some(read),
            Ok(AdapterCase::Lines(_)) => None,
            Err(why) => panic!("{why}"),
        })
        .collect()
}

/// The case's input written into `dir` under its own name, each file given
/// the modification time the case names: the folder's path, or the file's.
fn place_input(case: &BuiltInCase, dir: &Path) -> PathBuf {
    let name = case.input.rsplit('/').next().unwrap_or(&case.input);
    let input = dir.join(name);
    let base = match case.adapter {
        BuiltIn::Folder => {
            fs::create_dir_all(&input).unwrap();
            input.clone()
        }
        BuiltIn::Meeting => dir.to_path_buf(),
    };
    for (path, bytes) in &case.files {
        let file = base.join(path);
        fs::write(&file, bytes).unwrap();
        if let Some(at) = case.modified.get(path) {
            let at = OffsetDateTime::parse(at, &Rfc3339)
                .unwrap_or_else(|error| panic!("{}: `modified` {path}: {error}", case.id));
            File::options()
                .write(true)
                .open(&file)
                .unwrap()
                .set_modified(SystemTime::from(at))
                .unwrap();
        }
    }
    input
}

/// How the adapter's run of the case differs from what it expects, and
/// every breach of the adapter checks its lines make; none when it keeps
/// both.
fn differences(case: &BuiltInCase) -> Vec<String> {
    let dir = tempfile::tempdir().unwrap();
    let raw_dir = tempfile::tempdir().unwrap();
    let input = place_input(case, dir.path());
    let run = match case.adapter {
        BuiltIn::Folder => folder::run(&input, raw_dir.path()),
        BuiltIn::Meeting => meeting::run(&input, raw_dir.path()),
    };
    let lines = match run {
        Ok(lines) => lines,
        Err(why) => return vec![format!("{}: the run did not begin: {why}", case.id)],
    };
    let mut differences = Vec::new();
    let expected: Vec<Line> = case
        .expected
        .iter()
        .map(|line| read_line(line).unwrap_or_else(|why| panic!("{}: {why}: {line}", case.id)))
        .collect();
    if lines != expected {
        differences.push(format!(
            "{}: prints\n{}\nand the case expects\n{}",
            case.id,
            lines
                .iter()
                .map(Line::to_json)
                .collect::<Vec<_>>()
                .join("\n"),
            case.expected.join("\n")
        ));
    }
    let printed: Vec<String> = lines.iter().map(Line::to_json).collect();
    differences.extend(
        check_lines(&printed, RawPlace::Folder(raw_dir.path()), true)
            .iter()
            .map(|breach| format!("{}: {breach}", case.id)),
    );
    differences
}

#[test]
fn the_built_in_adapters_pass_every_case() {
    let cases = built_in_cases();
    assert!(!cases.is_empty());
    let differences: Vec<String> = cases.iter().flat_map(differences).collect();
    assert_eq!(differences, Vec::<String>::new());
}

#[test]
fn the_cases_hold_both_adapters_and_their_clauses() {
    let pack = Pack::embedded();
    let cited: BTreeSet<Clause> = pack
        .cases(Shape::Adapter)
        .iter()
        .filter(|case| matches!(read_adapter_case(case), Ok(AdapterCase::BuiltIn(_))))
        .flat_map(|case| satisfies(case).unwrap_or_else(|why| panic!("{why}")))
        .collect();
    for clause in [61, 62, 64] {
        assert!(
            cited.contains(&Clause::number(clause)),
            "no case cites {clause}"
        );
    }
    for adapter in [BuiltIn::Folder, BuiltIn::Meeting] {
        assert!(
            built_in_cases().iter().any(|case| case.adapter == adapter),
            "no case of {adapter:?}"
        );
    }
}

#[test]
fn a_file_dropped_twice() {
    let case = built_in_cases()
        .into_iter()
        .find(|case| case.id == "adapter/a-file-dropped-twice")
        .expect("the case of S14");
    let distinct: BTreeSet<&Vec<u8>> = case.files.values().collect();
    assert_eq!(
        (case.files.len(), distinct.len()),
        (2, 1),
        "one file, two names"
    );
    assert_eq!(differences(&case), Vec::<String>::new());
    let offers = case
        .expected
        .iter()
        .filter(|line| matches!(read_line(line), Ok(Line::Offer(_))))
        .count();
    assert_eq!(offers, 1);
}

#[test]
fn a_meeting_file_with_its_start() {
    let case = built_in_cases()
        .into_iter()
        .find(|case| case.id == "adapter/a-meeting-file-with-its-start")
        .expect("the case of a meeting's start");
    let text = String::from_utf8(case.files["standup.md"].clone()).unwrap();
    assert!(text.contains("start: 2026-10-20T15:00:00-04:00"), "{text}");
    let Ok(Line::Offer(offer)) = read_line(&case.expected[0]) else {
        panic!("{}", case.expected[0]);
    };
    assert!(offer.key.as_str().starts_with("meeting/file/"));
    assert_eq!(offer.event_at, "2026-10-20T15:00:00-04:00");
    assert_eq!(differences(&case), Vec::<String>::new());
}

#[test]
fn a_case_whose_lines_differ_fails_naming_it() {
    let mut case = built_in_cases()
        .into_iter()
        .find(|case| case.id == "adapter/a-meeting-file-with-no-start")
        .unwrap();
    case.modified
        .insert("call.txt".into(), "2026-10-24T09:00:00Z".into());
    let said = differences(&case);
    assert_eq!(said.len(), 1, "{said:?}");
    assert!(
        said[0].starts_with("adapter/a-meeting-file-with-no-start: prints"),
        "{}",
        said[0]
    );
}
