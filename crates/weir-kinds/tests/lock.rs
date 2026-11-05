//! A lock reads into every identity it records with its hash, by
//! identity, or is refused whole, naming every field that fails it, and
//! spells in identity order as C.4 writes it, reading back as it was; a
//! package is checked against it (91, 93, C.4 as amended at blueprints
//! 361a599; S18; core.30, core.45).

use weir_kinds::toml::{Table, Value};
use weir_kinds::{
    read_lock, read_package, spell_lock, Checked, ContentHash, Identity, Lock, LockEntry, Mismatch,
    Package, Problem,
};

const AN_IDENTITY_IS: &str = "an identity is `<source>/<name>@<version>`";

const A_CONTENT_HASH_IS: &str = "a content hash is `sha256:` and 64 lowercase hex digits";

/// A content hash's spelling, its 64 hex digits all `digit`.
fn hash(digit: char) -> String {
    format!("sha256:{}", digit.to_string().repeat(64))
}

fn identity(text: &str) -> Identity {
    text.parse().unwrap_or_else(|e| panic!("{text} reads: {e}"))
}

/// The text of a lock recording each `(kind, hash)` in its order.
fn lock_of(entries: &[(&str, &str)]) -> String {
    entries
        .iter()
        .map(|(kind, hash)| format!("[[kinds]]\nkind = {kind:?}\nhash = {hash:?}\n"))
        .collect::<Vec<_>>()
        .join("\n")
}

fn read(text: &str) -> Lock {
    read_lock(text).unwrap_or_else(|refusal| panic!("refused:\n{refusal}\n---\n{text}"))
}

fn refused(text: &str) -> Vec<Problem> {
    match read_lock(text) {
        Ok(lock) => panic!("read, though it should be refused: {lock:?}"),
        Err(refusal) => refusal.problems,
    }
}

/// The one problem at `field`.
fn at<'a>(problems: &'a [Problem], field: &str) -> &'a Problem {
    let found: Vec<_> = problems.iter().filter(|p| p.field == field).collect();
    match found[..] {
        [problem] => problem,
        _ => panic!("expected one problem at {field}, found {problems:?}"),
    }
}

/// Every identity a lock records, spelled, with its hash, in its order.
fn entries(lock: &Lock) -> Vec<(String, String)> {
    lock.entries
        .iter()
        .map(|(identity, entry)| (identity.to_string(), entry.hash.to_string()))
        .collect()
}

// read-lock reads every identity a lock records with its hash.

#[test]
fn two_versions_of_one_kind() {
    let text = lock_of(&[("acme/review@1", &hash('1')), ("acme/review@2", &hash('2'))]);
    let lock = read(&text);
    assert_eq!(
        entries(&lock),
        [
            ("acme/review@1".to_owned(), hash('1')),
            ("acme/review@2".to_owned(), hash('2')),
        ]
    );
    let entry = &lock.entries[&identity("acme/review@2")];
    assert_eq!(entry.hash, hash('2').parse::<ContentHash>().unwrap());
    assert!(entry.sections.is_empty());
    assert!(lock.sections.is_empty());
}

#[test]
fn entries_are_held_in_identity_order_whatever_order_they_are_recorded_in() {
    // By source, then name, then version as a number: `@10` after `@9`.
    let lock = read(&lock_of(&[
        ("zeta/a@1", &hash('1')),
        ("acme/review@10", &hash('2')),
        ("acme/review@9", &hash('3')),
        ("acme/lint@1", &hash('4')),
    ]));
    let order: Vec<_> = entries(&lock).into_iter().map(|(kind, _)| kind).collect();
    assert_eq!(
        order,
        ["acme/lint@1", "acme/review@9", "acme/review@10", "zeta/a@1"]
    );
}

#[test]
fn an_empty_lock() {
    assert_eq!(read(""), Lock::default());
    assert!(read("kinds = []\n").entries.is_empty());
    let lock = read("note = \"none yet\"\n");
    assert!(lock.entries.is_empty());
    assert_eq!(lock.sections["note"], Value::from("none yet"));
}

#[test]
fn a_lock_records_any_source_repo_and_launch_among_them() {
    let lock = read(&lock_of(&[
        ("repo/bump@1", &hash('1')),
        ("launch/fix@3", &hash('2')),
        ("flume/change@1", &hash('3')),
    ]));
    assert_eq!(lock.entries.len(), 3);
}

#[test]
fn keys_the_format_does_not_name() {
    let text = format!(
        "note = \"pinned by hand\"\n\n[[kinds]]\nkind = \"acme/review@1\"\nhash = {:?}\nsource_url = \"https://example.com/review\"\n",
        hash('1')
    );
    let lock = read(&text);
    assert_eq!(entries(&lock), [("acme/review@1".to_owned(), hash('1'))]);
    assert_eq!(lock.sections.len(), 1);
    assert_eq!(lock.sections["note"], Value::from("pinned by hand"));
    let entry = &lock.entries[&identity("acme/review@1")];
    assert_eq!(entry.sections.len(), 1);
    assert_eq!(
        entry.sections["source_url"],
        Value::from("https://example.com/review")
    );
}

#[test]
fn the_same_text_gives_the_same_answer() {
    let text = lock_of(&[("acme/review@1", &hash('1'))]);
    assert_eq!(read(&text), read(&text));
    let refused_text = lock_of(&[("acme/review", &hash('1'))]);
    assert_eq!(read_lock(&refused_text), read_lock(&refused_text));
}

// A lock is refused naming the field.

#[test]
fn an_identity_recorded_twice() {
    let text = lock_of(&[
        ("acme/review@1", &hash('1')),
        ("acme/lint@1", &hash('2')),
        ("acme/review@1", &hash('3')),
    ]);
    let problems = refused(&text);
    assert_eq!(problems.len(), 1, "{problems:?}");
    assert_eq!(
        at(&problems, "kinds[2].kind").reason,
        "`acme/review@1` is recorded at kinds[0] too, and a lock records an identity at most once"
    );
}

#[test]
fn an_identity_recorded_twice_is_named_where_the_first_entry_fails_otherwise() {
    let text = lock_of(&[
        ("acme/review@1", "sha256:00"),
        ("acme/review@1", &hash('1')),
    ]);
    let problems = refused(&text);
    let fields: Vec<_> = problems.iter().map(|p| p.field.as_str()).collect();
    assert_eq!(fields, ["kinds[0].hash", "kinds[1].kind"]);
}

#[test]
fn a_kind_that_is_no_whole_identity() {
    for (kind, why) in [
        (
            "acme/review",
            format!("it names no version, and {AN_IDENTITY_IS}"),
        ),
        ("review@1", format!("it names no source, and {AN_IDENTITY_IS}")),
        (
            "acme/review@02",
            "its version `02` has a leading zero, and a version is a whole number of at least 1, in decimal digits with no leading zero".to_owned(),
        ),
    ] {
        let problems = refused(&lock_of(&[(kind, &hash('1'))]));
        assert_eq!(problems.len(), 1, "{kind}: {problems:?}");
        assert_eq!(
            at(&problems, "kinds[0].kind").reason,
            format!("`{kind}` is no identity: {why}")
        );
    }
}

#[test]
fn a_hash_that_is_no_content_hash() {
    let hex = "4e07".repeat(16);
    for (spelling, why) in [
        (hex.clone(), "does not begin with `sha256:`".to_owned()),
        (
            format!("sha256:{}", hex.to_uppercase()),
            "holds 'E'".to_owned(),
        ),
        (
            format!("sha256:{}", &hex[..63]),
            "has 63 hex digits".to_owned(),
        ),
        (format!("sha256:{hex}0"), "has 65 hex digits".to_owned()),
    ] {
        let problems = refused(&lock_of(&[("acme/review@1", &spelling)]));
        assert_eq!(problems.len(), 1, "{spelling}: {problems:?}");
        assert_eq!(
            at(&problems, "kinds[0].hash").reason,
            format!("`{spelling}` is no content hash: it {why}, and {A_CONTENT_HASH_IS}")
        );
    }
}

#[test]
fn an_entry_with_no_kind_or_no_hash() {
    let problems = refused(&format!(
        "[[kinds]]\nhash = {:?}\n\n[[kinds]]\nkind = \"acme/review@1\"\n",
        hash('1')
    ));
    assert_eq!(problems.len(), 2, "{problems:?}");
    assert_eq!(
        at(&problems, "kinds[0].kind").reason,
        "is missing, and every entry of a lock has a kind, an identity"
    );
    assert_eq!(
        at(&problems, "kinds[1].hash").reason,
        "is missing, and every entry of a lock has a hash, a content hash"
    );
}

#[test]
fn a_kind_or_a_hash_that_is_not_a_string() {
    let problems = refused("[[kinds]]\nkind = 1\nhash = [\"sha256:\"]\n");
    assert_eq!(problems.len(), 2, "{problems:?}");
    assert_eq!(
        at(&problems, "kinds[0].kind").reason,
        "must be a string, an identity"
    );
    assert_eq!(
        at(&problems, "kinds[0].hash").reason,
        "must be a string, a content hash"
    );
}

#[test]
fn every_field_that_fails_is_named_in_the_order_read() {
    let text = lock_of(&[
        ("acme/review", &hash('1')),
        ("acme/lint@1", "sha256:00"),
        ("acme/lint@1", &hash('2')),
    ]);
    let problems = refused(&text);
    let fields: Vec<_> = problems.iter().map(|p| p.field.as_str()).collect();
    assert_eq!(fields, ["kinds[0].kind", "kinds[1].hash", "kinds[2].kind"]);
}

#[test]
fn a_lock_that_is_not_toml() {
    let problems = refused("[[kinds]]\nkind = \"acme/review@1\nhash = 1\n");
    assert_eq!(problems.len(), 1, "{problems:?}");
    assert!(
        problems[0].field.starts_with("line 2, column "),
        "{problems:?}"
    );
}

#[test]
fn a_lock_with_no_array_of_tables() {
    for text in [
        "kinds = \"acme/review@1\"\n",
        "[kinds]\nkind = \"acme/review@1\"\n",
        "kinds = [\"acme/review@1\"]\n",
    ] {
        let problems = refused(text);
        assert_eq!(problems.len(), 1, "{text}: {problems:?}");
        assert_eq!(
            at(&problems, "kinds").reason,
            "must be an array of tables, one per identity recorded"
        );
    }
}

// weir-kinds spells a lock that read-lock reads back.

/// A lock recording `acme/review@2`, then `acme/lint@1`, as a reader adds
/// them, with a top-level key the format does not name.
fn review_then_lint() -> Lock {
    let mut lock = Lock::default();
    lock.sections
        .insert("note".to_owned(), Value::from("pinned by hand"));
    for (kind, digit) in [("acme/review@2", '2'), ("acme/lint@1", '1')] {
        lock.entries.insert(
            identity(kind),
            LockEntry {
                hash: hash(digit).parse().unwrap(),
                sections: Table::new(),
            },
        );
    }
    lock
}

#[test]
fn a_lock_spelled_and_read_back() {
    let lock = review_then_lint();
    let text = spell_lock(&lock);
    assert_eq!(
        text,
        format!(
            "note = \"pinned by hand\"\n\n[[kinds]]\nkind = \"acme/lint@1\"\nhash = \"{}\"\n\n[[kinds]]\nkind = \"acme/review@2\"\nhash = \"{}\"\n",
            hash('1'),
            hash('2')
        )
    );
    let lint = text.find("acme/lint@1").unwrap();
    let review = text.find("acme/review@2").unwrap();
    assert!(lint < review, "{text}");
    assert_eq!(read(&text), lock);
}

#[test]
fn a_lock_read_in_any_order_spells_in_identity_order() {
    let text = lock_of(&[
        ("acme/review@10", &hash('1')),
        ("acme/review@9", &hash('2')),
    ]);
    let spelled = spell_lock(&read(&text));
    assert_eq!(
        spelled,
        lock_of(&[
            ("acme/review@9", &hash('2')),
            ("acme/review@10", &hash('1')),
        ])
    );
}

#[test]
fn the_same_lock_spells_the_same() {
    let lock = review_then_lint();
    let once = spell_lock(&lock);
    let twice = spell_lock(&lock);
    let again = spell_lock(&read(&once));
    assert_eq!(once.as_bytes(), twice.as_bytes());
    assert_eq!(once.as_bytes(), again.as_bytes());
}

#[test]
fn an_empty_lock_spells_as_no_text() {
    assert_eq!(spell_lock(&Lock::default()), "");
    assert_eq!(read(&spell_lock(&Lock::default())), Lock::default());
}

#[test]
fn keys_the_format_does_not_name_spell_where_they_stood_and_read_back() {
    let text = format!(
        concat!(
            "note = \"pinned by hand\"\n",
            "pinned = 1979-05-27T07:32:00Z\n",
            "\"a key.with dots\" = [1, 2.5, \"three\"]\n",
            "\"\\\"quoted\\\\\\n\" = true\n",
            "\n",
            "[meta]\n",
            "by = \"hand\"\n",
            "deeper = {{ many = [{{ lines = \"one\\ntwo\" }}] }}\n",
            "\n",
            "[[kinds]]\n",
            "source_url = \"https://example.com/review\"\n",
            "hash = {:?}\n",
            "kind = \"acme/review@1\"\n",
            "\n",
            "[kinds.origin]\n",
            "at = \"acme\"\n",
        ),
        hash('1')
    );
    let lock = read(&text);
    assert_eq!(lock.sections.len(), 5, "{lock:?}");
    let entry = &lock.entries[&identity("acme/review@1")];
    assert_eq!(entry.sections.len(), 2, "{entry:?}");

    let spelled = spell_lock(&lock);
    assert_eq!(read(&spelled), lock, "{spelled}");
    assert_eq!(spell_lock(&read(&spelled)), spelled);
    // The lock's own keys come before its first entry, and an entry's
    // after its `kind` and `hash`.
    let first_entry = spelled.find("[[kinds]]").unwrap();
    for key in ["note", "pinned", "meta"] {
        assert!(spelled.find(key).unwrap() < first_entry, "{key}: {spelled}");
    }
    let tail = &spelled[first_entry..];
    assert!(
        tail.starts_with(&format!(
            "[[kinds]]\nkind = \"acme/review@1\"\nhash = \"{}\"\n",
            hash('1')
        )),
        "{spelled}"
    );
    assert!(tail.contains("source_url = "), "{spelled}");
    assert!(tail.contains("origin = "), "{spelled}");
}

#[test]
fn a_key_the_format_names_among_the_unread_keys_is_not_spelled_there() {
    let mut lock = review_then_lint();
    lock.sections.insert("kinds".to_owned(), Value::from(1));
    let entry = lock.entries.get_mut(&identity("acme/lint@1")).unwrap();
    entry
        .sections
        .insert("hash".to_owned(), Value::from("sha256:00"));
    entry.sections.insert("kind".to_owned(), Value::from(2));
    assert_eq!(spell_lock(&lock), spell_lock(&review_then_lint()));
}

// A package is checked against a lock.

const REVIEW: &str = r#"
[kinds.review]
version = 1

[[kinds.review.stages]]
name = "review"
prompt = "prompts/review.md"
deliverables = ["commits"]
"#;

/// `acme/review@1`'s package, its prompt `prompt`.
fn review(prompt: &str) -> Package {
    let files = [("kind.toml", REVIEW), ("prompts/review.md", prompt)];
    read_package(files, &identity("acme/review@1")).unwrap()
}

/// A lock recording `package` at its hash, and nothing else.
fn locking(package: &Package) -> Lock {
    read(&lock_of(&[(
        &package.identity.to_string(),
        &package.hash.to_string(),
    )]))
}

#[test]
fn a_package_changed_by_a_byte_s18() {
    let package = review("Review the change.\n");
    let lock = locking(&package);
    let changed = review("Review the changes\n");
    assert_ne!(changed.hash, package.hash);

    let mismatch = lock.check(&changed.identity, &changed.hash).unwrap_err();
    assert_eq!(
        mismatch,
        Mismatch {
            identity: identity("acme/review@1"),
            locked: package.hash.clone(),
            given: changed.hash.clone(),
        }
    );
    assert_eq!(
        mismatch.to_string(),
        format!(
            "the lock records `acme/review@1` at `{}`, but its package's content hash is `{}`",
            package.hash, changed.hash
        )
    );
}

#[test]
fn recorded_and_not_recorded() {
    let package = review("Review the change.\n");
    assert_eq!(
        locking(&package).check(&package.identity, &package.hash),
        Ok(Checked::Recorded)
    );
    let other = read(&lock_of(&[("acme/review@2", &package.hash.to_string())]));
    assert_eq!(
        other.check(&package.identity, &package.hash),
        Ok(Checked::NotRecorded)
    );
    assert_eq!(
        Lock::default().check(&package.identity, &package.hash),
        Ok(Checked::NotRecorded)
    );
}
