//! The index of the statement's clauses the pack keeps, held to "The pack
//! keeps an index of the statement's clauses" (conformance-pack; 67;
//! core.37).

use weir_pack::{read_index, Clause, Pack};

/// The sections of Parts A and B and the clauses each holds, as the
/// statement numbers them.
fn the_statements_sections() -> Vec<(&'static str, Vec<Clause>)> {
    let range = |from: u32, to: u32| (from..=to).map(Clause::number).collect::<Vec<_>>();
    let mut b2 = vec![Clause::number(75), Clause::parse("75a").unwrap()];
    b2.extend(range(76, 77));
    let mut a8 = range(51, 58);
    a8.extend(range(88, 91));
    let mut b4 = range(81, 82);
    b4.extend(range(92, 93));
    // 94 to 99, statechart kinds and the engine, stay out of the index
    // until weir's statechart change takes them in with their cases.
    vec![
        ("A.1", range(1, 12)),
        ("A.2", range(13, 24)),
        ("A.3", range(25, 28)),
        ("A.4", range(29, 33)),
        ("A.5", range(34, 36)),
        ("A.6", range(37, 43)),
        ("A.7", range(44, 50)),
        ("A.8", a8),
        ("A.9", range(59, 60)),
        ("A.10", range(61, 64)),
        ("A.11", range(65, 68)),
        ("A.12", range(69, 71)),
        ("A.14", range(100, 107)),
        ("B.1", range(72, 74)),
        ("B.2", b2),
        ("B.3", range(78, 80)),
        ("B.4", b4),
        ("B.5", range(83, 85)),
        ("B.6", range(86, 87)),
        ("B.7", range(108, 113)),
    ]
}

fn the_packs_index() -> weir_pack::Index {
    let pack = Pack::embedded();
    let text = std::str::from_utf8(pack.file("clauses.toml").expect("the pack's index")).unwrap();
    read_index(text).unwrap_or_else(|why| panic!("{why}"))
}

#[test]
fn the_index_holds_every_clause_under_its_section() {
    let index = the_packs_index();
    assert_eq!(
        index.statement,
        "boomdrivedev/blueprints:design/weir/requirements.md"
    );
    assert!(!index.commit.is_empty());
    let read: Vec<(&str, Vec<Clause>)> = index
        .sections
        .iter()
        .map(|section| (section.name.as_str(), section.clauses.clone()))
        .collect();
    assert_eq!(read, the_statements_sections());
    let all: Vec<Clause> = index
        .sections
        .iter()
        .flat_map(|section| section.clauses.clone())
        .collect();
    assert_eq!(index.commit, "006fc34");
    assert_eq!(all.len(), 108, "1 to 93, 75a and 100 to 113, each once");
}

#[test]
fn the_check_counts_a1_to_a10_a14_and_71() {
    let counted = the_packs_index().counted().unwrap();
    let mut expected: Vec<Clause> = (1..=64).map(Clause::number).collect();
    expected.push(Clause::number(71));
    expected.extend((88..=91).map(Clause::number));
    expected.extend((100..=107).map(Clause::number));
    assert_eq!(counted.into_iter().collect::<Vec<_>>(), expected);
}

#[test]
fn a_clause_is_a_number_or_a_number_with_its_letter() {
    assert_eq!(Clause::parse("14"), Some(Clause::number(14)));
    assert_eq!(Clause::parse("75a").unwrap().to_string(), "75a");
    for text in ["", "a", "0", "75ab", "7 5", "-1", "75A"] {
        assert_eq!(Clause::parse(text), None, "{text:?}");
    }
    assert!(Clause::number(75) < Clause::parse("75a").unwrap());
    assert!(Clause::parse("75a").unwrap() < Clause::number(76));
    assert!(Clause::cited(&toml::Value::Integer(-3)).is_err());
    assert!(Clause::cited(&toml::Value::Boolean(true)).is_err());
}

#[test]
fn an_index_out_of_its_shape_is_refused() {
    let head = "statement = \"s\"\ncommit = \"c\"\n";
    for (text, says) in [
        ("commit = \"c\"\n", "statement"),
        (
            &*format!("{head}[[section]]\nname = \"A.1\"\nclauses = []\n"),
            "holds no clause",
        ),
        (
            &*format!("{head}[[section]]\nname = \"A.1\"\nclauses = [1, 1]\n"),
            "stands twice",
        ),
        (
            &*format!(
                "{head}[[section]]\nname = \"A.1\"\nclauses = [1]\n[[section]]\nname = \"A.1\"\nclauses = [2]\n"
            ),
            "named twice",
        ),
        (
            &*format!("{head}[[section]]\nname = \"A.1\"\nclauses = [\"x\"]\n"),
            "no clause's number",
        ),
        (
            &*format!("{head}words = \"a clause's words\"\n"),
            "words",
        ),
    ] {
        let refused = read_index(text).unwrap_err();
        assert!(refused.starts_with("clauses.toml: "), "{refused}");
        assert!(refused.contains(says), "{refused}");
    }
}

#[test]
fn an_index_lacking_a_counted_section() {
    let index = read_index(
        "statement = \"s\"\ncommit = \"c\"\n[[section]]\nname = \"A.1\"\nclauses = [1]\n",
    )
    .unwrap();
    assert_eq!(
        index.counted().unwrap_err(),
        "clauses.toml: holds no section A.2"
    );
}
