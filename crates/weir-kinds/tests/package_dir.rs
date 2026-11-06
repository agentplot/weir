//! A package read from a folder: walked without following any link into
//! the files read-package reads, so it answers as its paths and bytes do,
//! hashes the same wherever it lies and whenever it was written, and is
//! refused naming each symbolic link it holds (89, 90, 92; C.4 as amended
//! at blueprints 361a599; S17; record 0004). Every folder is a temporary
//! one of the test's own.

use std::fs::{self, File};
use std::os::unix::fs::{symlink, PermissionsExt};
use std::path::{Path, PathBuf};
use std::time::{Duration, SystemTime};

use weir_kinds::{
    read_package, read_package_dir, ContentHash, Identity, Package, Problem, Refusal,
};

const REVIEW: &str = r#"
[kinds.review]
version = 1

[[kinds.review.stages]]
name = "review"
prompt = "prompts/review.md"
deliverables = ["commits"]
"#;

/// `review`'s package: its declaration, its prompt and a part no field
/// names.
const FILES: [(&str, &str); 3] = [
    ("kind.toml", REVIEW),
    ("prompts/review.md", "Review the change.\n"),
    ("prompts/checklist.md", "- [ ] tests\n"),
];

const A_LINK: &str = "is a symbolic link, and a package holds none";

fn identity(text: &str) -> Identity {
    text.parse().unwrap_or_else(|e| panic!("{text} reads: {e}"))
}

/// Writes `files` under `root`, each at its path, making its folders.
fn lay_out(root: &Path, files: &[(&str, &str)]) {
    for (path, text) in files {
        let path = root.join(path);
        fs::create_dir_all(path.parent().unwrap()).unwrap();
        fs::write(path, text).unwrap();
    }
}

/// A temporary folder holding `review`'s package at `pkg`, and the path to
/// that package.
fn review_package() -> (tempfile::TempDir, PathBuf) {
    let top = tempfile::tempdir().unwrap();
    let dir = top.path().join("pkg");
    lay_out(&dir, &FILES);
    (top, dir)
}

fn held() -> Identity {
    identity("acme/review@1")
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

/// Every file under `dir`, followed through no link, with its bytes and
/// its modification time.
fn state(dir: &Path) -> Vec<(PathBuf, Vec<u8>, SystemTime)> {
    let mut found = Vec::new();
    for entry in fs::read_dir(dir).unwrap() {
        let entry = entry.unwrap();
        let metadata = fs::symlink_metadata(entry.path()).unwrap();
        if metadata.is_dir() {
            found.extend(state(&entry.path()));
        } else if metadata.is_file() {
            found.push((
                entry.path(),
                fs::read(entry.path()).unwrap(),
                metadata.modified().unwrap(),
            ));
        }
    }
    found.sort();
    found
}

// read-package reads a package from a folder.

#[test]
fn a_package_that_reads() {
    let (_top, dir) = review_package();
    let package = read_package_dir(&dir, &held()).unwrap();
    assert_eq!(package.identity, held());
    let prompts: Vec<_> = package.kind.stages.iter().map(|s| &s.prompt).collect();
    assert_eq!(prompts, ["prompts/review.md"]);
    assert_eq!(package.hash, read_package(FILES, &held()).unwrap().hash);
}

#[test]
fn a_package_read_twice() {
    let (_top, dir) = review_package();
    let before = state(&dir);
    let first = read_package_dir(&dir, &held()).unwrap();
    let second = read_package_dir(&dir, &held()).unwrap();
    assert_eq!(first, second);
    assert_eq!(
        state(&dir),
        before,
        "every file as it was, its bytes and its modification time"
    );
}

#[test]
fn the_same_files_on_disk_and_in_memory() {
    let change = "[kinds.change]\nversion = 2\n\n[[kinds.change.stages]]\nname = \"code\"\nprompt = \"prompts/code.md\"\nstanding = \"standing.md\"\ndeliverables = [\"tasks\"]\nrepeat = \"per-task-group\"\n";
    let files = [
        ("kind.toml", change),
        ("prompts/code.md", "Build the group.\n"),
        ("standing.md", "Commit as you go.\n"),
        ("a.md", "one\n"),
        ("a/b.md", "two\n"),
    ];
    let top = tempfile::tempdir().unwrap();
    lay_out(top.path(), &files);
    let held = identity("flume/change@2");
    let on_disk = read_package_dir(top.path(), &held).unwrap();
    let in_memory = read_package(files, &held).unwrap();
    assert_eq!(on_disk, in_memory);
    assert_eq!(on_disk.identity, held);
}

#[test]
fn a_package_of_a_kind_that_lands_with_no_stages() {
    // Its kind names no part, so `kind.toml` is the one file it holds (51,
    // 57 as amended at blueprints 51b6316; 89, 92).
    let train = "[kinds.train]\nlands = true\nstages = []\n";
    let files = [("kind.toml", train)];
    let top = tempfile::tempdir().unwrap();
    lay_out(top.path(), &files);
    let held = identity("acme/train@1");
    let on_disk = read_package_dir(top.path(), &held).unwrap();
    let in_memory = read_package(files, &held).unwrap();
    assert_eq!(on_disk, in_memory);
    assert_eq!(on_disk.identity, held);
    assert_eq!(on_disk.kind.name, "train");
    assert!(on_disk.kind.lands);
    assert!(on_disk.kind.stages.is_empty());
    assert_eq!(on_disk.hash, ContentHash::of(files));
}

#[test]
fn a_folder_reached_through_a_link() {
    let top = tempfile::tempdir().unwrap();
    let real = top.path().join("real");
    lay_out(&real.join("pkg"), &FILES);
    symlink(&real, top.path().join("via")).unwrap();
    symlink(real.join("pkg"), top.path().join("pkg-link")).unwrap();

    let through_its_own_path = read_package_dir(&real.join("pkg"), &held()).unwrap();
    let through_a_linked_folder = read_package_dir(&top.path().join("via/pkg"), &held()).unwrap();
    let through_a_link_to_it = read_package_dir(&top.path().join("pkg-link"), &held()).unwrap();
    assert_eq!(through_a_linked_folder, through_its_own_path);
    assert_eq!(through_a_link_to_it, through_its_own_path);
}

#[test]
fn two_machines_two_folders_s17() {
    let first = tempfile::tempdir().unwrap();
    let second = tempfile::tempdir().unwrap();
    let here = first.path().join("pkg");
    let there = second.path().join("deep/er/still/pkg");
    lay_out(&here, &FILES);
    let mut reversed = FILES;
    reversed.reverse();
    lay_out(&there, &reversed);
    let long_ago = SystemTime::UNIX_EPOCH + Duration::from_secs(1_000_000_000);
    for (path, _) in FILES {
        let file = File::options().write(true).open(there.join(path)).unwrap();
        file.set_modified(long_ago).unwrap();
        fs::set_permissions(there.join(path), fs::Permissions::from_mode(0o600)).unwrap();
        fs::set_permissions(here.join(path), fs::Permissions::from_mode(0o644)).unwrap();
    }
    let here = read_package_dir(&here, &held()).unwrap();
    let there = read_package_dir(&there, &held()).unwrap();
    assert_eq!(here.hash, there.hash);
    assert_eq!(here, there);
}

#[test]
fn a_file_added_to_the_folder_moves_its_hash_s18() {
    let (_top, dir) = review_package();
    let before = read_package_dir(&dir, &held()).unwrap().hash;
    fs::write(dir.join(".DS_Store"), "").unwrap();
    let after = read_package_dir(&dir, &held()).unwrap().hash;
    assert_ne!(after, before);
}

#[test]
fn an_entry_neither_a_file_a_folder_nor_a_link_is_not_hashed() {
    let (_top, dir) = review_package();
    let before = read_package_dir(&dir, &held()).unwrap();
    let _socket = std::os::unix::net::UnixListener::bind(dir.join("s")).unwrap();
    assert!(fs::symlink_metadata(dir.join("s")).is_ok());
    let after = read_package_dir(&dir, &held()).unwrap();
    assert_eq!(after, before);
}

#[test]
fn an_empty_folder_is_not_hashed() {
    let (_top, dir) = review_package();
    let before = read_package_dir(&dir, &held()).unwrap();
    fs::create_dir(dir.join("empty")).unwrap();
    assert_eq!(read_package_dir(&dir, &held()).unwrap(), before);
}

// A package holds no symbolic link.

#[test]
fn a_prompt_that_is_a_link_inside_the_package_s17() {
    let (_top, dir) = review_package();
    let kind_toml = REVIEW.replace("prompts/review.md", "prompts/code.md");
    fs::write(dir.join("kind.toml"), kind_toml).unwrap();
    symlink("review.md", dir.join("prompts/code.md")).unwrap();
    let problems = refused(read_package_dir(&dir, &held()));
    assert_eq!(fields(&problems), ["prompts/code.md"]);
    assert_eq!(problems[0].reason, A_LINK);
}

#[test]
fn links_of_every_other_kind() {
    let (top, dir) = review_package();
    fs::write(top.path().join("outside.md"), "elsewhere\n").unwrap();
    symlink("prompts/review.md", dir.join("notes.md")).unwrap();
    symlink("prompts", dir.join("shared")).unwrap();
    symlink(top.path().join("outside.md"), dir.join("outside.md")).unwrap();
    symlink("nowhere.md", dir.join("dangling.md")).unwrap();
    symlink("../kind.toml", dir.join("prompts/up.toml")).unwrap();
    let problems = refused(read_package_dir(&dir, &held()));
    assert_eq!(
        fields(&problems),
        [
            "dangling.md",
            "notes.md",
            "outside.md",
            "prompts/up.toml",
            "shared"
        ],
        "each link's file, by its path within the package"
    );
    assert!(problems.iter().all(|p| p.reason == A_LINK), "{problems:?}");
}

#[test]
fn kind_toml_itself_a_link() {
    let (_top, dir) = review_package();
    fs::rename(dir.join("kind.toml"), dir.join("real.toml")).unwrap();
    symlink("real.toml", dir.join("kind.toml")).unwrap();
    let problems = refused(read_package_dir(&dir, &held()));
    assert_eq!(fields(&problems), ["kind.toml"], "named once, as a link");
    assert_eq!(problems[0].reason, A_LINK);
}

#[test]
fn a_part_beneath_a_linked_folder_is_named_once_by_the_link() {
    let (_top, dir) = review_package();
    let kind_toml = REVIEW.replace("prompts/review.md", "shared/review.md");
    fs::write(dir.join("kind.toml"), kind_toml).unwrap();
    symlink("prompts", dir.join("shared")).unwrap();
    let problems = refused(read_package_dir(&dir, &held()));
    assert_eq!(fields(&problems), ["shared"]);
}

#[test]
fn a_package_of_links_is_refused_with_its_other_problems() {
    let (_top, dir) = review_package();
    symlink("review.md", dir.join("prompts/link.md")).unwrap();
    let problems = refused(read_package_dir(&dir, &identity("acme/review@2")));
    assert_eq!(
        fields(&problems),
        ["prompts/link.md", "kinds.review.version"],
        "the links first, then the rest in design 4's order"
    );
}

// A folder that cannot be read.

#[test]
fn a_folder_within_the_package_that_cannot_be_read() {
    let (_top, dir) = review_package();
    fs::create_dir(dir.join("private")).unwrap();
    fs::set_permissions(dir.join("private"), fs::Permissions::from_mode(0o000)).unwrap();
    if fs::read_dir(dir.join("private")).is_ok() {
        // Run as root, which reads every folder: nothing to refuse.
        fs::set_permissions(dir.join("private"), fs::Permissions::from_mode(0o755)).unwrap();
        return;
    }
    let read = read_package_dir(&dir, &held());
    fs::set_permissions(dir.join("private"), fs::Permissions::from_mode(0o755)).unwrap();
    let problems = refused(read);
    assert_eq!(fields(&problems), ["private"]);
    assert!(
        problems[0].reason.starts_with("could not be read: "),
        "{problems:?}"
    );
}

#[test]
fn a_folder_that_is_not_there_or_not_a_folder() {
    let top = tempfile::tempdir().unwrap();
    let missing = top.path().join("missing");
    let problems = refused(read_package_dir(&missing, &held()));
    assert_eq!(fields(&problems), [missing.display().to_string()]);
    assert!(problems[0].reason.starts_with("could not be read: "));

    let file = top.path().join("kind.toml");
    fs::write(&file, REVIEW).unwrap();
    let problems = refused(read_package_dir(&file, &held()));
    assert_eq!(fields(&problems), [file.display().to_string()]);
    assert_eq!(
        problems[0].reason,
        "is not a folder, and a package is a folder"
    );
}
