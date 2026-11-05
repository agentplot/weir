//! A source's name and a kind's name are one or more ASCII letters,
//! digits, `_` and `-`, and `read-kinds` refuses a kind's table or a
//! table of settings under any other key, naming it (57, 88; C.4 as
//! amended at blueprints 361a599; core.42; record 0004).

use weir_kinds::{check_name, read_kinds, Problem};

const A_NAME_IS: &str = "a name is one or more ASCII letters, digits, `_` and `-`";
const A_KINDS_NAME_IS: &str = "a kind's name is one or more ASCII letters, digits, `_` and `-`";

// A source's name and a kind's name are letters, digits, _ and -.

#[test]
fn names_that_are_names() {
    for name in ["acme", "review", "code_review", "review-2", "R2"] {
        assert_eq!(check_name(name), Ok(()), "{name}");
    }
}

#[test]
fn text_that_is_not_a_name() {
    for (text, why) in [
        ("ac me", "holds ' '"),
        ("acme.x", "holds '.'"),
        ("a/b", "holds '/'"),
        ("review@2", "holds '@'"),
        ("é", "holds 'é'"),
        ("", "is empty"),
    ] {
        let refusal = check_name(text).expect_err(text);
        assert_eq!(refusal.text, text);
        assert_eq!(refusal.reason, format!("{why}, and {A_NAME_IS}"), "{text}");
    }
    assert_eq!(
        check_name("a/b").unwrap_err().to_string(),
        format!("`a/b` holds '/', and {A_NAME_IS}")
    );
    assert_eq!(
        check_name("").unwrap_err().to_string(),
        format!("the text is empty, and {A_NAME_IS}")
    );
}

// A kind's key is a name.

#[test]
fn a_kind_under_a_key_that_is_not_a_name() {
    let refusal = read_kinds(
        r#"
[[kinds."a/b".stages]]
name = "code"
prompt = "prompts/code.md"
deliverables = ["commits"]
"#,
    )
    .unwrap_err();
    assert_eq!(
        refusal.problems,
        [Problem {
            field: "kinds.\"a/b\"".to_owned(),
            reason: format!("holds '/', and {A_KINDS_NAME_IS}"),
        }]
    );
}

#[test]
fn settings_under_a_key_that_is_not_a_name() {
    // The refusal is the read's Err: no kind comes back, `bump` among them.
    let refusal = read_kinds(
        r#"
[kinds."review@2".agents]
model = "opus"

[[kinds.bump.stages]]
name = "code"
prompt = "prompts/code.md"
deliverables = ["commits"]
"#,
    )
    .unwrap_err();
    let fields: Vec<_> = refusal.problems.iter().map(|p| p.field.as_str()).collect();
    assert_eq!(fields, ["kinds.\"review@2\""]);
    assert!(refusal.problems[0].reason.contains(A_KINDS_NAME_IS));
}

#[test]
fn nothing_under_a_key_that_is_not_a_name_is_read() {
    // A version of 0 and no stage would each be refused under a kind; the
    // key is refused once, and nothing under it.
    for (text, field) in [
        ("[kinds.\"a.b\"]\nversion = 0\n", "kinds.\"a.b\""),
        ("[kinds.\"a b\"]\nversion = 0\n", "kinds.\"a b\""),
        ("[kinds.\"\"]\nversion = 0\n", "kinds.\"\""),
        ("kinds.\"é\" = 3\n", "kinds.\"é\""),
    ] {
        let refusal = read_kinds(text).unwrap_err();
        let fields: Vec<_> = refusal.problems.iter().map(|p| p.field.as_str()).collect();
        assert_eq!(fields, [field], "{text}");
    }
}

#[test]
fn names_of_every_allowed_character() {
    let mut text = String::new();
    for name in ["code_review", "review-2", "R2"] {
        text.push_str(&format!(
            "[[kinds.{name}.stages]]\nname = \"code\"\nprompt = \"prompts/code.md\"\n\
             deliverables = [\"commits\"]\n\n"
        ));
    }
    let declaration = read_kinds(&text).unwrap();
    assert_eq!(
        declaration.kinds.keys().collect::<Vec<_>>(),
        ["R2", "code_review", "review-2"]
    );
}
