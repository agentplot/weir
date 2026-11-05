//! A package's content hash is C.4's, over every regular file with its
//! path, and spells and reads back as `sha256:` and 64 lowercase hex
//! digits (90, C.4; S18; core.44).

use weir_kinds::ContentHash;

const A_CONTENT_HASH_IS: &str = "a content hash is `sha256:` and 64 lowercase hex digits";

/// The package of the kind-packages spec's *A known package's hash*.
const KNOWN: [(&str, &str); 2] = [("a.md", "one\n"), ("a/b.md", "two\n")];

const KNOWN_HASH: &str = "sha256:9fd22069b62b656b446d5cc2b60f897a2640d2d005bab6cfb39d6289ca8a9cf8";

// A package's content hash is C.4's, over every regular file.

#[test]
fn a_known_packages_hash() {
    let hash = ContentHash::of(KNOWN);
    assert_eq!(hash.to_string(), KNOWN_HASH);
    assert_eq!(hash.hex(), &KNOWN_HASH["sha256:".len()..]);
}

#[test]
fn files_are_hashed_in_byte_order_of_their_paths_whatever_order_they_come_in() {
    // `a.md` comes before `a/b.md`, since `.` comes before `/`, though a
    // walk of the folder may give `a/b.md` first.
    let [first, second] = KNOWN;
    assert_eq!(ContentHash::of([second, first]).to_string(), KNOWN_HASH);
}

#[test]
fn one_byte_changed_s18() {
    let before = ContentHash::of(KNOWN);
    let changed = ContentHash::of([("a.md", "one\n"), ("a/b.md", "twO\n")]);
    assert_ne!(changed, before);
    let added = ContentHash::of([("a.md", "one\n"), ("a/b.md", "two\n"), (".DS_Store", "")]);
    assert_ne!(added, before);
}

#[test]
fn a_file_moved_or_its_bytes_moved_between_files_changes_the_hash() {
    let before = ContentHash::of(KNOWN);
    let moved = ContentHash::of([("a.md", "one\n"), ("b/b.md", "two\n")]);
    assert_ne!(moved, before);
    // The lengths keep a byte at the end of one file from passing for the
    // first byte of the next.
    let shifted = ContentHash::of([("a.md", "one\nt"), ("a/b.md", "wo\n")]);
    assert_ne!(shifted, before);
}

#[test]
fn paths_and_bytes_need_not_be_text() {
    let as_text = ContentHash::of(KNOWN);
    let as_bytes = ContentHash::of(KNOWN.map(|(path, text)| (path.as_bytes(), text.as_bytes())));
    assert_eq!(as_bytes, as_text);
    let not_utf8 = ContentHash::of([(&b"a\xff.md"[..], &b"\xfe"[..])]);
    assert_eq!(not_utf8.hex().len(), 64);
}

// A content hash reads and spells back.

#[test]
fn a_content_hash_read_and_spelled() {
    let hash: ContentHash = KNOWN_HASH.parse().unwrap();
    assert_eq!(hash.to_string(), KNOWN_HASH);
    assert_eq!(hash, ContentHash::of(KNOWN));
}

#[test]
fn text_that_is_no_content_hash() {
    let hex = &KNOWN_HASH["sha256:".len()..];
    for (text, why) in [
        (
            KNOWN_HASH.to_uppercase().replacen("SHA256", "sha256", 1),
            "holds 'F'".to_owned(),
        ),
        (hex.to_owned(), "does not begin with `sha256:`".to_owned()),
        (
            format!("SHA256:{hex}"),
            "does not begin with `sha256:`".to_owned(),
        ),
        (KNOWN_HASH[..70].to_owned(), "has 63 hex digits".to_owned()),
        (format!("{KNOWN_HASH}0"), "has 65 hex digits".to_owned()),
        ("sha256:".to_owned(), "has 0 hex digits".to_owned()),
        (String::new(), "does not begin with `sha256:`".to_owned()),
    ] {
        let refusal = text.parse::<ContentHash>().unwrap_err();
        assert_eq!(refusal.text, text);
        assert_eq!(
            refusal.reason,
            format!("is no content hash: it {why}, and {A_CONTENT_HASH_IS}"),
            "{text}"
        );
    }
}

#[test]
fn a_refusal_shows_the_text_and_why() {
    let refusal = "sha256:00".parse::<ContentHash>().unwrap_err();
    assert_eq!(
        refusal.to_string(),
        format!("`sha256:00` is no content hash: it has 2 hex digits, and {A_CONTENT_HASH_IS}")
    );
    let refusal = "".parse::<ContentHash>().unwrap_err();
    assert_eq!(
        refusal.to_string(),
        format!(
            "the text is no content hash: it does not begin with `sha256:`, and {A_CONTENT_HASH_IS}"
        )
    );
}
