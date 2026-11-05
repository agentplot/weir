//! A kind's identity reads into its source, name and version and spells
//! back, and a kind named with its source or version left out reads into
//! its parts (88; C.4 as amended at blueprints 361a599; core.42).

use weir_kinds::{Identity, NamedKind, LAUNCH, REPO};

const A_NAME_IS: &str = "a name is one or more ASCII letters, digits, `_` and `-`";
const A_VERSION_IS: &str =
    "a version is a whole number of at least 1, in decimal digits with no leading zero";
const AN_IDENTITY_IS: &str = "an identity is `<source>/<name>@<version>`";

fn identity(text: &str) -> Identity {
    text.parse().unwrap_or_else(|e| panic!("{text} reads: {e}"))
}

fn named(text: &str) -> NamedKind {
    text.parse().unwrap_or_else(|e| panic!("{text} reads: {e}"))
}

// An identity reads into its parts and spells back.

#[test]
fn an_identity_read_and_spelled() {
    for (text, source, name, version) in [
        ("acme/review@1", "acme", "review", 1),
        ("flume/change@12", "flume", "change", 12),
    ] {
        let read = identity(text);
        assert_eq!(
            (read.source(), read.name(), read.version()),
            (source, name, version),
            "{text}"
        );
        assert_eq!(read.to_string(), text);
    }
}

#[test]
fn text_that_is_no_identity() {
    for (text, why) in [
        (
            "acme/review",
            format!("it names no version, and {AN_IDENTITY_IS}"),
        ),
        (
            "review@1",
            format!("it names no source, and {AN_IDENTITY_IS}"),
        ),
        (
            "acme/review@0",
            format!("its version `0` is 0, and {A_VERSION_IS}"),
        ),
        (
            "acme/review@02",
            format!("its version `02` has a leading zero, and {A_VERSION_IS}"),
        ),
        (
            "acme/review@+1",
            format!("its version `+1` holds '+', and {A_VERSION_IS}"),
        ),
        (
            "acme/review@1.5",
            format!("its version `1.5` holds '.', and {A_VERSION_IS}"),
        ),
        (
            "a/b/c@1",
            format!("its name `b/c` holds '/', and {A_NAME_IS}"),
        ),
        ("/review@1", format!("its source is empty, and {A_NAME_IS}")),
        ("acme/@1", format!("its name is empty, and {A_NAME_IS}")),
        (
            "acme/re view@1",
            format!("its name `re view` holds ' ', and {A_NAME_IS}"),
        ),
    ] {
        let refusal = text.parse::<Identity>().expect_err(text);
        assert_eq!(refusal.text, text);
        assert_eq!(refusal.reason, format!("is no identity: {why}"), "{text}");
    }
    assert_eq!(
        "acme/review".parse::<Identity>().unwrap_err().to_string(),
        format!("`acme/review` is no identity: it names no version, and {AN_IDENTITY_IS}")
    );
}

#[test]
fn a_version_too_large_for_a_number_is_no_identity() {
    let refusal = "acme/review@18446744073709551616"
        .parse::<Identity>()
        .unwrap_err();
    assert!(
        refusal.reason.starts_with("is no identity: its version"),
        "{refusal}"
    );
    assert_eq!(
        identity("acme/review@18446744073709551615").version(),
        u64::MAX
    );
}

#[test]
fn identities_in_order() {
    let mut identities: Vec<Identity> =
        ["acme/review@10", "acme/review@9", "acme/lint@1", "beta/a@1"]
            .into_iter()
            .map(identity)
            .collect();
    identities.sort();
    let spelled: Vec<String> = identities.iter().map(Identity::to_string).collect();
    assert_eq!(
        spelled,
        ["acme/lint@1", "acme/review@9", "acme/review@10", "beta/a@1"]
    );
}

#[test]
fn a_repositorys_and_a_launchs_sources() {
    assert_eq!((REPO, LAUNCH), ("repo", "launch"));
    let read = identity("repo/review@1");
    assert_eq!(read.source(), REPO);
    assert_eq!(identity("launch/review@1").source(), LAUNCH);
}

// A kind named with its source or version left out reads into its parts.

#[test]
fn the_four_forms() {
    for (text, source, version) in [
        ("review", None, None),
        ("review@2", None, Some(2)),
        ("acme/review", Some("acme"), None),
        ("acme/review@2", Some("acme"), Some(2)),
    ] {
        let read = named(text);
        assert_eq!(
            (read.source(), read.name(), read.version()),
            (source, "review", version),
            "{text}"
        );
        assert_eq!(read.to_string(), text);
    }
}

#[test]
fn a_named_kind_left_whole() {
    assert_eq!(
        named("acme/review@2").identity(),
        Some(identity("acme/review@2"))
    );
    assert_eq!(named("acme/review").identity(), None);
    assert_eq!(named("review@2").identity(), None);
}

#[test]
fn text_that_names_no_kind() {
    for (text, why) in [
        (
            "review@",
            format!("its version is empty, and {A_VERSION_IS}"),
        ),
        ("@2", format!("its name is empty, and {A_NAME_IS}")),
        ("acme/", format!("its name is empty, and {A_NAME_IS}")),
        ("/review", format!("its source is empty, and {A_NAME_IS}")),
        (
            "a/b/c",
            format!("its name `b/c` holds '/', and {A_NAME_IS}"),
        ),
        (
            "review@0",
            format!("its version `0` is 0, and {A_VERSION_IS}"),
        ),
        (
            "re view",
            format!("its name `re view` holds ' ', and {A_NAME_IS}"),
        ),
    ] {
        let refusal = text.parse::<NamedKind>().expect_err(text);
        assert_eq!(refusal.text, text);
        assert_eq!(refusal.reason, format!("names no kind: {why}"), "{text}");
    }
}
