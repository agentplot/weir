//! A package given as paths and bytes reads into its one kind at the
//! identity it is held at, with its content hash, or is refused whole,
//! naming every field that fails it (89, 90, 92; C.4 as amended at
//! blueprints 361a599; S17; core.43).

use weir_kinds::{read_package, ContentHash, Identity, Package, Problem, Refusal};

const REVIEW: &str = r#"
[kinds.review]
version = 1

[[kinds.review.stages]]
name = "review"
prompt = "prompts/review.md"
deliverables = ["commits"]
"#;

const PROMPT: (&str, &str) = ("prompts/review.md", "Review the change.\n");

fn identity(text: &str) -> Identity {
    text.parse().unwrap_or_else(|e| panic!("{text} reads: {e}"))
}

/// The package of `kind_toml` and `files` beside it, held at `held`.
fn read(kind_toml: &str, files: &[(&str, &str)], held: &str) -> Result<Package, Refusal> {
    let files: Vec<(&str, &str)> = [("kind.toml", kind_toml)]
        .into_iter()
        .chain(files.iter().copied())
        .collect();
    read_package(files, &identity(held))
}

/// `review`'s package with its stage's prompt, and `standing` where given,
/// written as `prompt` and `standing`.
fn review_naming(prompt: &str, standing: Option<&str>) -> String {
    let standing = standing.map_or(String::new(), |s| format!("standing = {s:?}\n"));
    format!(
        "[[kinds.review.stages]]\nname = \"review\"\nprompt = {prompt:?}\n{standing}deliverables = [\"commits\"]\n"
    )
}

fn refused(read: Result<Package, Refusal>) -> Vec<Problem> {
    match read {
        Ok(package) => panic!("read, though it should be refused: {package:?}"),
        Err(refusal) => refusal.problems,
    }
}

fn fields(problems: &[Problem]) -> Vec<&str> {
    problems.iter().map(|p| p.field.as_str()).collect()
}

/// The one problem at `field`.
fn at<'a>(problems: &'a [Problem], field: &str) -> &'a Problem {
    let found: Vec<_> = problems.iter().filter(|p| p.field == field).collect();
    match found[..] {
        [problem] => problem,
        _ => panic!("expected one problem at {field}, found {problems:?}"),
    }
}

// read-package reads a package at the identity it is held at.

#[test]
fn a_package_that_reads() {
    let package = read(REVIEW, &[PROMPT], "acme/review@1").unwrap();
    assert_eq!(package.identity, identity("acme/review@1"));
    assert_eq!(package.kind.name, "review");
    assert_eq!(package.kind.version, 1);
    let stages: Vec<_> = package.kind.stages.iter().map(|s| &s.prompt).collect();
    assert_eq!(stages, ["prompts/review.md"]);
    assert_eq!(
        package.hash,
        ContentHash::of([("kind.toml", REVIEW), PROMPT]),
        "the hash is over every file with its path"
    );
    assert!(package.sections.is_empty());
}

#[test]
fn a_package_read_twice_answers_the_same() {
    let first = read(REVIEW, &[PROMPT], "acme/review@1").unwrap();
    let second = read(REVIEW, &[PROMPT], "acme/review@1").unwrap();
    assert_eq!(first, second);
}

#[test]
fn files_given_in_any_order_answer_the_same() {
    let files = [PROMPT, ("kind.toml", REVIEW), ("notes.md", "")];
    let mut reversed = files;
    reversed.reverse();
    let held = identity("acme/review@1");
    assert_eq!(
        read_package(files, &held).unwrap(),
        read_package(reversed, &held).unwrap()
    );
}

#[test]
fn a_section_beside_the_kind() {
    let kind_toml = format!("[profiles.fast]\nmodel = \"haiku\"\n{REVIEW}");
    let package = read(&kind_toml, &[PROMPT], "acme/review@1").unwrap();
    assert_eq!(
        package.sections.keys().collect::<Vec<_>>(),
        ["profiles"],
        "returned unread (56)"
    );
}

#[test]
fn a_file_no_field_names_is_hashed() {
    let with = read(REVIEW, &[PROMPT, (".DS_Store", "x")], "acme/review@1").unwrap();
    let without = read(REVIEW, &[PROMPT], "acme/review@1").unwrap();
    assert_ne!(with.hash, without.hash);
}

#[test]
fn bytes_and_paths_need_not_be_text_but_kind_toml() {
    let files: Vec<(&[u8], &[u8])> = vec![
        (b"kind.toml", REVIEW.as_bytes()),
        (PROMPT.0.as_bytes(), PROMPT.1.as_bytes()),
        (b"art/\xffpicture.png", b"\x89PNG\xff"),
    ];
    let package = read_package(files, &identity("acme/review@1")).unwrap();
    assert_eq!(package.kind.name, "review");
}

// A path given is a file's path within the package.

#[test]
fn a_path_given_that_is_not_within_a_package() {
    for (path, why) in [
        ("/kind.toml", "is absolute"),
        ("./kind.toml", "holds a `.` segment"),
        ("prompts//code.md", "holds an empty segment"),
        ("../kind.toml", "holds a `..` segment"),
        ("prompts/", "holds an empty segment"),
        ("", "is empty"),
    ] {
        let problems = refused(read(REVIEW, &[PROMPT, (path, "x")], "acme/review@1"));
        let field = if path.is_empty() { "\"\"" } else { path };
        assert_eq!(fields(&problems), [field], "{path}");
        assert_eq!(
            problems[0].reason,
            format!(
                "{why}, and a path given is a file's path within the package, `/` between segments none of which is empty, `.` or `..`"
            ),
            "{path}"
        );
    }
}

#[test]
fn a_path_given_twice() {
    let problems = refused(read(REVIEW, &[PROMPT, PROMPT, PROMPT], "acme/review@1"));
    assert_eq!(fields(&problems), ["prompts/review.md"]);
    assert_eq!(
        problems[0].reason,
        "is given twice, and a package holds one file at each path"
    );
}

#[test]
fn a_file_beneath_a_file() {
    let problems = refused(read(
        REVIEW,
        &[PROMPT, ("notes", "x"), ("notes/a.md", "y")],
        "acme/review@1",
    ));
    assert_eq!(fields(&problems), ["notes/a.md"]);
    assert_eq!(
        problems[0].reason,
        "lies beneath `notes`, a file of the package, and no file lies beneath a file"
    );
}

// A package declares exactly one kind and names no other.

#[test]
fn no_kind_toml() {
    let problems = refused(read_package([PROMPT], &identity("acme/review@1")));
    assert_eq!(fields(&problems), ["kind.toml"]);
    assert_eq!(
        problems[0].reason,
        "is missing, and a package declares its kind in `kind.toml`, a regular file at its root"
    );
}

#[test]
fn kind_toml_a_folder() {
    let problems = refused(read_package(
        [PROMPT, ("kind.toml/kind.toml", REVIEW)],
        &identity("acme/review@1"),
    ));
    assert_eq!(fields(&problems), ["kind.toml"]);
    assert!(problems[0].reason.starts_with("is a folder, "));
}

#[test]
fn kind_toml_not_text() {
    let files: [(&[u8], &[u8]); 1] = [(b"kind.toml", b"\xff\xfe")];
    let problems = refused(read_package(files, &identity("acme/review@1")));
    assert_eq!(fields(&problems), ["kind.toml"]);
    assert_eq!(
        problems[0].reason,
        "is not UTF-8 text, and a declaration is TOML"
    );
}

#[test]
fn no_kind_or_two() {
    let none = refused(read("[profiles]\n", &[], "acme/review@1"));
    assert_eq!(fields(&none), ["kinds"]);
    assert_eq!(
        none[0].reason,
        "declares no kind, and a package declares exactly one kind"
    );

    let lint = "[[kinds.lint.stages]]\nname = \"lint\"\nprompt = \"prompts/review.md\"\ndeliverables = [\"commits\"]\n";
    let two = refused(read(&format!("{REVIEW}{lint}"), &[PROMPT], "acme/review@1"));
    assert_eq!(fields(&two), ["kinds"]);
    assert_eq!(
        two[0].reason,
        "declares 2 kinds, `lint` and `review`, and a package declares exactly one kind"
    );
}

#[test]
fn settings_for_another_kind() {
    let kind_toml = format!("{REVIEW}\n[kinds.fix.agents]\nmodel = \"opus\"\n");
    let problems = refused(read(&kind_toml, &[PROMPT], "acme/review@1"));
    assert_eq!(fields(&problems), ["kinds.fix"]);
    assert_eq!(
        problems[0].reason,
        "gives settings to `fix`, a kind the package does not declare, and a package names no kind but its own"
    );
}

#[test]
fn a_declaration_read_kinds_refuses() {
    let kind_toml = REVIEW.replace(r#"["commits"]"#, r#"["commits", "change"]"#);
    let problems = refused(read(&kind_toml, &[PROMPT], "acme/review@1"));
    assert_eq!(
        fields(&problems),
        ["kinds.review.stages[0].deliverables"],
        "as read-kinds names it"
    );
}

#[test]
fn a_kind_toml_that_is_no_toml() {
    let problems = refused(read("[kinds.review\n", &[PROMPT], "acme/review@1"));
    assert_eq!(fields(&problems), ["line 1, column 14"]);
}

// Every part a package names is a regular file inside it.

#[test]
fn a_prompt_outside_the_package_s17() {
    let kind_toml = review_naming("../prompts/code.md", None);
    let problems = refused(read(&kind_toml, &[PROMPT], "acme/review@1"));
    assert_eq!(fields(&problems), ["kinds.review.stages[0].prompt"]);
    assert_eq!(
        problems[0].reason,
        "`../prompts/code.md` holds a `..` segment, and nothing a package names lies outside it"
    );
}

#[test]
fn the_same_package_with_its_prompt_moved_inside_it_reads_s17() {
    let kind_toml = review_naming("prompts/code.md", None);
    let package = read(
        &kind_toml,
        &[("prompts/code.md", "Code it.\n")],
        "acme/review@1",
    )
    .unwrap();
    assert_eq!(package.kind.stages[0].prompt, "prompts/code.md");
}

#[test]
fn absolute_paths_and_standing_instructions() {
    let kind_toml = review_naming("/etc/code.md", None);
    let problems = refused(read(&kind_toml, &[PROMPT], "acme/review@1"));
    assert_eq!(fields(&problems), ["kinds.review.stages[0].prompt"]);
    assert_eq!(
        problems[0].reason,
        "`/etc/code.md` is absolute, and a part's path is relative to `kind.toml`"
    );

    let kind_toml = review_naming("prompts/review.md", Some("../standing.md"));
    let problems = refused(read(&kind_toml, &[PROMPT], "acme/review@1"));
    assert_eq!(fields(&problems), ["kinds.review.stages[0].standing"]);
}

#[test]
fn a_part_the_package_does_not_hold() {
    let kind_toml = review_naming("prompts/code.md", None);
    let missing = refused(read(&kind_toml, &[PROMPT], "acme/review@1"));
    assert_eq!(fields(&missing), ["kinds.review.stages[0].prompt"]);
    assert_eq!(
        missing[0].reason,
        "`prompts/code.md` names no file the package holds, and a package holds every part it names"
    );

    let folder = refused(read(
        &kind_toml,
        &[("prompts/code.md/part.md", "x")],
        "acme/review@1",
    ));
    assert_eq!(fields(&folder), ["kinds.review.stages[0].prompt"]);
    assert_eq!(
        folder[0].reason,
        "`prompts/code.md` names a folder, and a part is a regular file of the package"
    );

    let standing = review_naming("prompts/review.md", Some("standing.md"));
    let problems = refused(read(&standing, &[PROMPT], "acme/review@1"));
    assert_eq!(fields(&problems), ["kinds.review.stages[0].standing"]);
}

#[test]
fn a_part_naming_the_packages_own_folder() {
    for prompt in [".", "./", "./."] {
        let problems = refused(read(
            &review_naming(prompt, None),
            &[PROMPT],
            "acme/review@1",
        ));
        assert_eq!(
            fields(&problems),
            ["kinds.review.stages[0].prompt"],
            "{prompt}"
        );
    }
}

#[test]
fn a_path_with_a_dot_segment() {
    for prompt in [
        "./prompts/review.md",
        "prompts/./review.md",
        "prompts//review.md",
    ] {
        let kind_toml = review_naming(prompt, Some("./prompts/review.md"));
        let package = read(&kind_toml, &[PROMPT], "acme/review@1")
            .unwrap_or_else(|e| panic!("{prompt}: {e}"));
        assert_eq!(package.kind.stages[0].prompt, prompt, "returned as written");
        assert_eq!(
            package.kind.stages[0].standing.as_deref(),
            Some("./prompts/review.md")
        );
    }
}

#[test]
fn every_stage_of_the_kind_is_checked() {
    let kind_toml = format!(
        "{REVIEW}\n[[kinds.review.stages]]\nname = \"again\"\nprompt = \"prompts/again.md\"\nstanding = \"/standing.md\"\ndeliverables = [\"commits\"]\n"
    );
    let problems = refused(read(&kind_toml, &[PROMPT], "acme/review@1"));
    assert_eq!(
        fields(&problems),
        [
            "kinds.review.stages[1].prompt",
            "kinds.review.stages[1].standing"
        ]
    );
}

// A package holds no symbolic link: a part may name another by a relative
// path, as a Markdown link does.

#[test]
fn a_markdown_link_between_parts() {
    let prompt = (
        "prompts/review.md",
        "Hold it to [the checklist](../checklist.md).\n",
    );
    let package = read(REVIEW, &[prompt], "acme/review@1").unwrap();
    assert_eq!(package.kind.stages[0].prompt, "prompts/review.md");
}

// A package held at another identity is refused naming both.

#[test]
fn held_at_another_version() {
    let problems = refused(read(REVIEW, &[PROMPT], "acme/review@2"));
    let problem = at(&problems, "kinds.review.version");
    assert_eq!(fields(&problems), ["kinds.review.version"]);
    assert!(problem.reason.contains("`acme/review@2`"), "{problem}");
    assert!(problem.reason.contains("`acme/review@1`"), "{problem}");
    assert_eq!(
        problem.reason,
        "makes the package `acme/review@1`, but it is held at `acme/review@2`, and a package is held at its declaration's name and version"
    );
}

#[test]
fn held_under_another_name() {
    let problems = refused(read(REVIEW, &[PROMPT], "acme/lint@1"));
    assert_eq!(fields(&problems), ["kinds.review"]);
    let problem = at(&problems, "kinds.review");
    assert!(problem.reason.contains("`acme/lint@1`"), "{problem}");
    assert!(problem.reason.contains("`acme/review@1`"), "{problem}");
}

#[test]
fn held_under_another_name_and_version_is_named_once_at_the_kind() {
    let problems = refused(read(REVIEW, &[PROMPT], "acme/lint@3"));
    assert_eq!(fields(&problems), ["kinds.review"]);
}

#[test]
fn a_version_left_out_is_1() {
    let kind_toml = REVIEW.replace("version = 1\n", "");
    assert!(read(&kind_toml, &[PROMPT], "acme/review@1").is_ok());
    let problems = refused(read(&kind_toml, &[PROMPT], "acme/review@2"));
    assert_eq!(fields(&problems), ["kinds.review.version"]);
}

// A package is refused whole, naming every field that fails it, in design
// 4's order.

#[test]
fn every_problem_in_order() {
    let kind_toml = format!(
        "{}\n[kinds.fix.agents]\nmodel = \"opus\"\n",
        review_naming("../code.md", Some("/standing.md")).replace("review", "lint")
    );
    let problems = refused(read(&kind_toml, &[PROMPT, ("./x.md", "")], "acme/review@1"));
    assert_eq!(
        fields(&problems),
        [
            "./x.md",
            "kinds.fix",
            "kinds.lint",
            "kinds.lint.stages[0].prompt",
            "kinds.lint.stages[0].standing",
        ]
    );
}
