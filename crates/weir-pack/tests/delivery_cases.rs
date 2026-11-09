//! The delivery cases, held to delivery-cases' "A delivery case holds a
//! store's starting records and one call" and "A delivery case names one
//! thing that must come of its call" (C.7, C.8).

use weir_pack::{read_delivery_case, Pack, Shape};

/// The case `delivery/<name>` of a pack holding `files`, each by its path
/// from the case's folder.
fn case_of(name: &str, files: &[(&str, &str)]) -> weir_pack::Case {
    let pack = Pack::from_files(
        files
            .iter()
            .map(|(path, text)| (format!("delivery/{name}/{path}"), text.as_bytes().to_vec())),
    );
    pack.case(Shape::Delivery, name).expect("the case")
}

const START: &str = "format: weir-delivery/1\n\
                     key: place/7f3a/end/2\n\
                     reader: planner\n\
                     message: construct ended.\n\
                     recorded_at: 2026-11-08T10:02:00-05:00\n\
                     expires_at: 2026-11-15T10:02:00-05:00\n";

const ADDED: &str = "format: weir-confirmation/1\n\
                     key: place/7f3a/end/2\n\
                     reader: planner\n\
                     at: 2026-11-08T10:40:12-05:00\n";

#[test]
fn a_misspelt_call() {
    let case = case_of(
        "a-misspelt-call",
        &[(
            "case.toml",
            "satisfies = [102]\n\n[call.confrim]\nkey = \"place/7f3a/end/2\"\nreader = \"planner\"\n\
             at = \"2026-11-08T10:40:12-05:00\"\n",
        )],
    );
    let why = read_delivery_case(&case).unwrap_err();
    assert!(why.starts_with("delivery/a-misspelt-call: "), "{why}");
    assert!(why.contains("confrim"), "{why}");
    let misplaced = case_of(
        "a-field-under-the-call",
        &[(
            "case.toml",
            "satisfies = [105]\n\n[call.owed]\nnow = \"2026-11-08T10:40:12-05:00\"\ndone = []\n",
        )],
    );
    assert!(read_delivery_case(&misplaced).unwrap_err().contains("done"));
}

#[test]
fn a_case_that_expects_both_records_and_a_refusal() {
    let toml =
        "satisfies = [102]\n\n[call.confirm]\nkey = \"place/7f3a/end/2\"\nreader = \"planner\"\n\
                at = \"2026-11-08T10:40:12-05:00\"\n";
    let both = case_of(
        "both",
        &[
            (
                "case.toml",
                &format!("{toml}\n[refused]\nfield = \"key\"\nreason_holds = \"no delivery\"\n"),
            ),
            ("start.rec", START),
            ("added.rec", ADDED),
        ],
    );
    let why = read_delivery_case(&both).unwrap_err();
    assert!(why.starts_with("delivery/both: "), "{why}");
    let none = case_of("none", &[("case.toml", toml), ("start.rec", START)]);
    assert!(read_delivery_case(&none)
        .unwrap_err()
        .starts_with("delivery/none: "));
    let one = case_of(
        "one",
        &[
            ("case.toml", toml),
            ("start.rec", START),
            ("added.rec", ADDED),
        ],
    );
    let read = read_delivery_case(&one).unwrap();
    assert_eq!(read.id, "delivery/one");
    assert_eq!(read.start.len(), 1);
    assert!(matches!(&read.expected, weir_pack::Outcome::Added(added) if added.len() == 1));
}

#[test]
fn a_case_out_of_its_shape() {
    let toml =
        "satisfies = [101]\nheld = true\n\n[call.confirm]\nkey = \"k\"\nreader = \"planner\"\n\
                at = \"2026-11-08T10:40:12-05:00\"\n";
    for (name, files, says) in [
        ("held-of-confirm", vec![("case.toml", toml)], "never gives"),
        (
            "a-stray-file",
            vec![("case.toml", toml), ("notes.md", "a note")],
            "notes.md",
        ),
        (
            "records-out-of-their-format",
            vec![
                ("case.toml", toml),
                ("start.rec", "format: weir-delivery/2\n"),
            ],
            "start.rec",
        ),
        (
            "two-calls",
            vec![(
                "case.toml",
                "satisfies = [105]\ndone = []\n\n[call.owed]\nnow = \"x\"\n\n[call.confirm]\n\
                 key = \"k\"\nreader = \"r\"\nat = \"x\"\n",
            )],
            "exactly one of `call.record`",
        ),
    ] {
        let why = read_delivery_case(&case_of(name, &files)).unwrap_err();
        assert!(why.starts_with(&format!("delivery/{name}: ")), "{why}");
        assert!(why.contains(says), "{why}");
    }
}

/// Design 13's cases: each case's name, its call, the outcome it names and
/// the clauses it cites.
const THE_CASES: [(&str, &str, &str, &[u32]); 21] = [
    (
        "a-delivery-recorded-for-two-readers",
        "record",
        "added",
        &[100, 104],
    ),
    ("a-delivery-with-no-reader", "record", "refused", &[100]),
    ("a-reader-named-twice", "record", "refused", &[100]),
    ("a-delivery-with-no-message", "record", "refused", &[100]),
    ("a-span-of-no-seconds", "record", "refused", &[104]),
    ("one-key-recorded-twice", "record", "held", &[101]),
    (
        "a-reader-confirms-by-its-key",
        "confirm",
        "added",
        &[102, 106],
    ),
    ("a-second-confirmation", "confirm", "already", &[102]),
    (
        "a-confirmation-of-a-key-no-delivery-is-under",
        "confirm",
        "refused",
        &[102],
    ),
    (
        "a-confirmation-by-a-reader-not-named",
        "confirm",
        "refused",
        &[102],
    ),
    ("a-hand-over-confirms-nothing", "hand", "added", &[103]),
    ("handed-and-still-owed", "owed", "owed", &[102, 103, 105]),
    ("a-reader-confirms-on-receipt", "hand", "added", &[103, 106]),
    (
        "a-hand-over-to-a-reader-that-confirmed",
        "hand",
        "already",
        &[103],
    ),
    (
        "a-hand-over-to-a-reader-not-named",
        "hand",
        "refused",
        &[103],
    ),
    (
        "an-expired-delivery-is-shown",
        "owed",
        "owed",
        &[104, 105, 106],
    ),
    (
        "an-expired-delivery-confirmed",
        "confirm",
        "added",
        &[102, 104],
    ),
    ("owed-in-the-order-recorded", "owed", "owed", &[105]),
    ("a-name-stays-known", "owed", "owed", &[106]),
    (
        "a-delivery-done-when-every-reader-confirms",
        "owed",
        "owed",
        &[107],
    ),
    (
        "a-delivery-not-done-while-a-reader-has-not-confirmed",
        "owed",
        "owed",
        &[107],
    ),
];

fn outcome(outcome: &weir_pack::Outcome) -> &'static str {
    match outcome {
        weir_pack::Outcome::Added(_) => "added",
        weir_pack::Outcome::Held => "held",
        weir_pack::Outcome::Already(_) => "already",
        weir_pack::Outcome::Refused { .. } => "refused",
        weir_pack::Outcome::Owed { .. } => "owed",
    }
}

#[test]
fn the_cases_named() {
    use std::collections::BTreeSet;
    use weir_pack::Clause;

    let pack = Pack::embedded();
    let names: BTreeSet<String> = pack
        .cases(Shape::Delivery)
        .into_iter()
        .map(|case| case.name)
        .collect();
    let named: BTreeSet<String> = THE_CASES
        .iter()
        .map(|(name, ..)| name.to_string())
        .collect();
    assert_eq!(names, named, "the pack's delivery cases are design 13's");
    let mut cited = BTreeSet::new();
    for (name, call, expects, cites) in THE_CASES {
        let case = pack.case(Shape::Delivery, name).expect("the case");
        let read = read_delivery_case(&case).unwrap_or_else(|why| panic!("{why}"));
        assert_eq!(read.call.name(), call, "{name}");
        assert_eq!(outcome(&read.expected), expects, "{name}");
        let satisfies: Vec<Clause> = cites.iter().copied().map(Clause::number).collect();
        assert_eq!(read.satisfies, satisfies, "{name}");
        cited.extend(satisfies);
    }
    for clause in 100..=107 {
        assert!(
            cited.contains(&Clause::number(clause)),
            "no delivery case cites {clause}"
        );
    }
}
