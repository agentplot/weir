//! Builds the pack into the crate: every file under the workspace's `pack/`,
//! by its path from the pack's root, written to `$OUT_DIR/pack.rs` as
//! bytes, so the crate holds the pack it was built with wherever it runs
//! (C.7). A name beginning with `.` is passed over, as `Pack::read` passes
//! it over. Cargo builds it again whenever a file under `pack/` changes.

use std::fmt::Write as _;
use std::fs;
use std::path::{Path, PathBuf};

fn main() {
    let root = Path::new(&std::env::var("CARGO_MANIFEST_DIR").expect("cargo names the crate"))
        .join("../../pack");
    let root = root
        .canonicalize()
        .unwrap_or_else(|error| panic!("the pack {}: {error}", root.display()));
    println!("cargo:rerun-if-changed=build.rs");
    println!("cargo:rerun-if-changed={}", root.display());
    let mut files = Vec::new();
    walk(&root, &root, &mut files);
    files.sort();
    let mut out = String::from("pub(crate) static FILES: &[(&str, &[u8])] = &[\n");
    for (path, file) in &files {
        writeln!(
            out,
            "    ({path:?}, include_bytes!({:?})),",
            file.display().to_string()
        )
        .expect("a string takes every write");
    }
    out.push_str("];\n");
    let out_dir = PathBuf::from(std::env::var("OUT_DIR").expect("cargo names the output"));
    fs::write(out_dir.join("pack.rs"), out).expect("the pack written to the build's output");
}

/// Every file under `dir`, by its path from `root` with `/` between its
/// folders, and where it lies.
fn walk(root: &Path, dir: &Path, files: &mut Vec<(String, PathBuf)>) {
    let entries = fs::read_dir(dir).unwrap_or_else(|error| panic!("{}: {error}", dir.display()));
    for entry in entries {
        let entry = entry.unwrap_or_else(|error| panic!("{}: {error}", dir.display()));
        if entry.file_name().to_string_lossy().starts_with('.') {
            continue;
        }
        let path = entry.path();
        if path.is_dir() {
            walk(root, &path, files);
        } else {
            let name = path
                .strip_prefix(root)
                .expect("a file under the pack")
                .components()
                .map(|part| part.as_os_str().to_str().expect("a pack path is UTF-8"))
                .collect::<Vec<_>>()
                .join("/");
            files.push((name, path));
        }
    }
}
