//! The same declaration always reads the same, and reading it depends on
//! its text alone (58, 82, I7; S10).

mod common;

use common::read;
use weir_kinds::{Deliverable, End, Kind};

/// Each stage of a kind by name, with its end.
fn stages(kind: &Kind) -> Vec<(&str, &End)> {
    kind.stages
        .iter()
        .map(|s| (s.name.as_str(), &s.end))
        .collect()
}

#[test]
fn two_kinds_read_the_same_twice() {
    let first = read("fix-and-change.toml").unwrap();
    assert_eq!(first.kinds.keys().collect::<Vec<_>>(), ["change", "fix"]);

    let fix = &first.kinds["fix"];
    assert_eq!(fix.version, 1);
    assert!(fix.starts_unready);
    assert!(fix.lands);
    assert_eq!(
        stages(fix),
        [("code", &End::Deliverable(Deliverable::Commits))]
    );

    let change = &first.kinds["change"];
    assert_eq!(change.version, 1);
    assert_eq!(
        stages(change),
        [
            ("construct", &End::Deliverable(Deliverable::Change)),
            ("code", &End::Deliverable(Deliverable::Tasks)),
            ("verify", &End::Deliverable(Deliverable::Archived)),
        ]
    );

    assert_eq!(read("fix-and-change.toml").unwrap(), first);
}

#[test]
fn a_refusal_reads_the_same_twice() {
    let first = read("two-faults.toml").unwrap_err();
    assert_eq!(read("two-faults.toml").unwrap_err(), first);
}
