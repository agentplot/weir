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

mod stores {
    //! The stores weir's tests hold to the cases: one of each spelling of
    //! C.8, and two that break it (113; S25).

    use std::cell::Cell;
    use std::convert::Infallible;

    use weir_delivery::{json, rec, Record, Refusal, Store, Time};

    /// A store over a list of records.
    #[derive(Default)]
    pub struct List {
        pub records: Vec<Record>,
        pub adds: usize,
        pub reads: Cell<usize>,
    }

    impl Store for List {
        type Error = Infallible;

        fn read(&self) -> Result<Vec<Record>, Infallible> {
            self.reads.set(self.reads.get() + 1);
            Ok(self.records.clone())
        }

        fn add(&mut self, records: &[Record]) -> Result<(), Infallible> {
            self.adds += 1;
            self.records.extend_from_slice(records);
            Ok(())
        }
    }

    /// A store over recutils text, as on a shared branch.
    #[derive(Default)]
    pub struct Recutils {
        pub text: String,
    }

    impl Store for Recutils {
        type Error = Refusal;

        fn read(&self) -> Result<Vec<Record>, Refusal> {
            rec::read(&self.text)
        }

        fn add(&mut self, records: &[Record]) -> Result<(), Refusal> {
            if records.is_empty() {
                return Ok(());
            }
            if !self.text.is_empty() {
                self.text.push('\n');
            }
            self.text.push_str(&rec::write(records));
            Ok(())
        }
    }

    /// A store over JSON lines, as on a machine.
    #[derive(Default)]
    pub struct JsonLines {
        pub text: String,
    }

    impl Store for JsonLines {
        type Error = Refusal;

        fn read(&self) -> Result<Vec<Record>, Refusal> {
            json::read_lines(&self.text)
        }

        fn add(&mut self, records: &[Record]) -> Result<(), Refusal> {
            self.text.push_str(&json::write_lines(records));
            Ok(())
        }
    }

    /// A store that leaves out of its reads every delivery expiring before
    /// 2026-11-10T00:00:00Z (S25).
    #[derive(Default)]
    pub struct DropsExpired(pub List);

    impl Store for DropsExpired {
        type Error = Infallible;

        fn read(&self) -> Result<Vec<Record>, Infallible> {
            let cut = Time::parse("2026-11-10T00:00:00Z", "cut")
                .unwrap()
                .instant();
            Ok(self
                .0
                .read()?
                .into_iter()
                .filter(|record| {
                    !matches!(record, Record::Delivery(delivery)
                        if delivery.expires_at().instant() < cut)
                })
                .collect())
        }

        fn add(&mut self, records: &[Record]) -> Result<(), Infallible> {
            self.0.add(records)
        }
    }

    /// A store that adds no `weir-known/1` record.
    #[derive(Default)]
    pub struct LosesKnownNames(pub List);

    impl Store for LosesKnownNames {
        type Error = Infallible;

        fn read(&self) -> Result<Vec<Record>, Infallible> {
            self.0.read()
        }

        fn add(&mut self, records: &[Record]) -> Result<(), Infallible> {
            let kept: Vec<Record> = records
                .iter()
                .filter(|record| !matches!(record, Record::Known(_)))
                .cloned()
                .collect();
            self.0.add(&kept)
        }
    }
}

/// Every case's verdict, the failures said in full.
fn failed(verdicts: &[(String, Result<(), Vec<String>>)]) -> Vec<String> {
    verdicts
        .iter()
        .filter_map(|(name, verdict)| {
            verdict
                .as_ref()
                .err()
                .map(|said| format!("{name}:\n  {}", said.join("\n  ")))
        })
        .collect()
}

/// The verdict of the case `name` among `verdicts`.
fn verdict_of<'a>(
    verdicts: &'a [(String, Result<(), Vec<String>>)],
    name: &str,
) -> &'a Result<(), Vec<String>> {
    &verdicts
        .iter()
        .find(|(id, _)| id == &format!("delivery/{name}"))
        .unwrap_or_else(|| panic!("no verdict for {name}"))
        .1
}

/// The JSON line of the first record of `format` in the case's file.
fn json_of(name: &str, file: &str, format: &str) -> String {
    let case = Pack::embedded()
        .case(Shape::Delivery, name)
        .expect("the case");
    let records = weir_delivery::rec::read(case.text(file).unwrap()).unwrap();
    let record = records
        .iter()
        .find(|record| record.format() == format)
        .expect("a record of the format");
    weir_delivery::json::write(record)
}

#[test]
fn the_cases_through_a_store_of_each_spelling() {
    let pack = Pack::embedded();
    for (store, verdicts) in [
        (
            "a list",
            weir_pack::conform_deliveries(&pack, stores::List::default),
        ),
        (
            "recutils",
            weir_pack::conform_deliveries(&pack, stores::Recutils::default),
        ),
        (
            "JSON lines",
            weir_pack::conform_deliveries(&pack, stores::JsonLines::default),
        ),
    ] {
        assert_eq!(verdicts.len(), THE_CASES.len(), "{store}");
        let failed = failed(&verdicts);
        assert!(failed.is_empty(), "through {store}:\n{}", failed.join("\n"));
    }
}

#[test]
fn a_store_that_drops_expired_deliveries() {
    let verdicts = weir_pack::conform_deliveries(&Pack::embedded(), stores::DropsExpired::default);
    let said = verdict_of(&verdicts, "an-expired-delivery-is-shown")
        .as_ref()
        .expect_err("the case fails");
    let delivery = json_of(
        "an-expired-delivery-is-shown",
        "start.rec",
        "weir-delivery/1",
    );
    assert!(
        said.contains(&format!("not read back: {delivery}")),
        "{said:#?}"
    );
}

#[test]
fn a_store_that_loses_known_names() {
    let verdicts =
        weir_pack::conform_deliveries(&Pack::embedded(), stores::LosesKnownNames::default);
    let said = verdict_of(&verdicts, "a-name-stays-known")
        .as_ref()
        .expect_err("the case fails");
    let known = json_of("a-name-stays-known", "start.rec", "weir-known/1");
    assert!(
        said.contains(&format!("not read back: {known}")),
        "{said:#?}"
    );
}

#[test]
fn a_case_read_from_its_folder() {
    let name = "an-expired-delivery-is-shown";
    let built_in = Pack::embedded().case(Shape::Delivery, name).unwrap();
    let dir = tempfile::tempdir().unwrap();
    built_in
        .write_to(&dir.path().join("delivery").join(name))
        .unwrap();
    let pack = Pack::read(dir.path()).unwrap();
    assert_eq!(pack.files().len(), built_in.files.len(), "its folder alone");
    let first = weir_pack::conform_deliveries(&pack, stores::List::default);
    let second = weir_pack::conform_deliveries(&pack, stores::Recutils::default);
    assert_eq!(first, [(format!("delivery/{name}"), Ok(()))]);
    assert_eq!(first, second, "the same verdict from its own files alone");
}

#[test]
fn a_store_implemented_outside_the_crate() {
    let case = Pack::embedded()
        .case(Shape::Delivery, "a-reader-confirms-on-receipt")
        .unwrap();
    let case = read_delivery_case(&case).unwrap();
    let mut store = stores::List::default();
    weir_pack::run_delivery_case(&case, &mut store).unwrap();
    assert_eq!(
        store.adds, 2,
        "the starting records, then what the call gave"
    );
    assert!(store.reads.get() >= 2, "read back after each add");
    let formats: Vec<&str> = store
        .records
        .iter()
        .map(weir_delivery::Record::format)
        .collect();
    assert_eq!(
        formats[case.start.len()..],
        ["weir-handover/1", "weir-confirmation/1", "weir-known/1"]
    );
}

#[test]
fn a_case_whose_answer_differs() {
    let built_in = Pack::embedded()
        .case(Shape::Delivery, "handed-and-still-owed")
        .unwrap();
    let mut files: Vec<(&str, String)> = built_in
        .files
        .iter()
        .map(|(path, bytes)| (path.as_str(), String::from_utf8(bytes.clone()).unwrap()))
        .collect();
    for (path, text) in &mut files {
        if *path == "case.toml" {
            *text = text.replace("handed = [\"notice\"]", "handed = [\"wait\"]");
        }
    }
    let files: Vec<(&str, &str)> = files
        .iter()
        .map(|(path, text)| (*path, text.as_str()))
        .collect();
    let case = read_delivery_case(&case_of("handed-and-still-owed", &files)).unwrap();
    let said = weir_pack::run_delivery_case(&case, &mut stores::List::default()).unwrap_err();
    assert_eq!(said.len(), 1, "{said:#?}");
    assert!(
        said[0].starts_with(
            "`owed` gave `planner`, not known: `place/7f3a/end/2` owed, handed by notice"
        ),
        "{said:#?}"
    );
}
