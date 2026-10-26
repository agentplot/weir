//! The pack built into the crate, read the same way as the folder it was
//! built from (C.7).

use std::fs;
use std::path::Path;

use weir_pack::{Pack, Shape};

/// The repository's pack, read from its folder.
fn from_the_folder() -> Pack {
    Pack::read(&Path::new(env!("CARGO_MANIFEST_DIR")).join("../../pack")).unwrap()
}

#[test]
fn the_built_in_pack_is_the_folder() {
    assert_eq!(Pack::embedded(), from_the_folder());
}

#[test]
fn a_case_reads_the_same_from_both() {
    let built_in = Pack::embedded().cases(Shape::Cadence);
    let read = from_the_folder().cases(Shape::Cadence);
    assert!(!built_in.is_empty());
    assert_eq!(built_in[0], read[0]);
    assert_eq!(built_in[0].id(), format!("cadence/{}", built_in[0].name));
}

#[test]
fn a_folder_read_as_a_pack() {
    let dir = tempfile::tempdir().unwrap();
    let write = |path: &str, text: &str| {
        let file = dir.path().join(path);
        fs::create_dir_all(file.parent().unwrap()).unwrap();
        fs::write(file, text).unwrap();
    };
    write("host/held/case.toml", "satisfies = [14]\n");
    write("host/held/start/inbox/x.rec", "format: weir-capture/1\n");
    write("host/held/.hidden", "passed over");
    write("host/stray.toml", "satisfies = [1]\n");
    write("kinds/two/case.toml", "satisfies = [51]\n");
    let pack = Pack::read(dir.path()).unwrap();
    assert_eq!(
        pack.files().keys().collect::<Vec<_>>(),
        [
            "host/held/case.toml",
            "host/held/start/inbox/x.rec",
            "host/stray.toml",
            "kinds/two/case.toml"
        ]
    );
    let held = pack.case(Shape::Host, "held").unwrap();
    assert_eq!(held.id(), "host/held");
    assert_eq!(held.text("case.toml"), Ok("satisfies = [14]\n"));
    assert_eq!(
        held.under("start").keys().collect::<Vec<_>>(),
        ["inbox/x.rec"]
    );
    assert_eq!(pack.cases(Shape::Host).len(), 1);
    assert!(pack.cases(Shape::Adapter).is_empty());
    assert_eq!(
        held.text("tip/x"),
        Err("host/held: holds no `tip/x`".to_string())
    );

    let out = tempfile::tempdir().unwrap();
    held.write_to(out.path()).unwrap();
    assert_eq!(
        fs::read_to_string(out.path().join("start/inbox/x.rec")).unwrap(),
        "format: weir-capture/1\n"
    );
}

#[test]
fn a_folder_that_is_not_there() {
    let refused = Pack::read(Path::new("/nonexistent/pack")).unwrap_err();
    assert!(refused.starts_with("the pack's"), "{refused}");
}

#[test]
fn the_built_in_pack_holds_the_index_and_a_case_of_each_shape() {
    let pack = Pack::embedded();
    assert!(pack.file("clauses.toml").is_some(), "no clauses.toml");
    for shape in Shape::ALL {
        assert!(!pack.cases(shape).is_empty(), "no case of {shape}");
    }
}
