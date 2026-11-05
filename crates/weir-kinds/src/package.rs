//! A kind's package: one kind at one version with every part it names,
//! read at the identity it is held at, from its files given as paths and
//! bytes or from a folder, and hashed the same on every machine (89, 90,
//! 92; C.4 as amended at blueprints 361a599; core.43, core.44).

use std::collections::{BTreeMap, BTreeSet};
use std::fs;
use std::path::Path;

use toml::Table;

use crate::fields::{field, key};
use crate::hash::ContentHash;
use crate::identity::Identity;
use crate::kinds::{read_kinds, Kind};
use crate::refusal::{Problem, Problems, Refusal};

/// The file at a package's root that declares its kind (C.4).
const KIND_TOML: &str = "kind.toml";

/// A package read: the one kind it declares, at the identity it is held
/// at, with its content hash (89, 90, 92).
#[derive(Debug, Clone, PartialEq)]
pub struct Package {
    /// The identity it was held at, which its declaration's name and
    /// version are (89; core.43).
    pub identity: Identity,
    /// Its one kind, as `read-kinds` reads it, each part a stage names
    /// returned as written (53, 89).
    pub kind: Kind,
    /// Its content hash, over every regular file it holds (90, C.4).
    pub hash: ContentHash,
    /// Every top-level key of its `kind.toml` but `kinds`, unread and as
    /// written (56).
    pub sections: Table,
}

/// `read-package` over a package's files, each a path within the package
/// and its bytes, as a tool holds its built-in kinds, and the identity it
/// is held at (92). It gives the package's one kind, that identity and
/// its content hash, or refuses the package whole, naming every field
/// that fails it (57, 89).
///
/// A path is relative, with `/` between segments none of which is empty,
/// `.` or `..`, and names one file; a path that is not, or that lies
/// beneath another file's, is refused naming it, since no folder holds
/// such files. The same files give the same answer as the folder that
/// holds them read by [`read_package_dir`], and every time; nothing is
/// read but what is given (90, 93).
pub fn read_package<P: AsRef<[u8]>, B: AsRef<[u8]>>(
    files: impl IntoIterator<Item = (P, B)>,
    held: &Identity,
) -> Result<Package, Refusal> {
    let given: Vec<(P, B)> = files.into_iter().collect();
    let files = given
        .iter()
        .map(|(path, bytes)| (path.as_ref(), bytes.as_ref()))
        .collect();
    read(files, &Walked::default(), held)
}

/// `read-package` over the folder `dir` and the identity it is held at
/// (92): the folder walked into its regular files, read as
/// [`read_package`] reads them.
///
/// The walk follows no link: a symbolic link anywhere in the package,
/// whatever it leads to, is refused naming its path within the package,
/// and an entry that is neither a regular file, a folder nor a link is no
/// part of it and is not hashed (89, 90; C.4 as amended at blueprints
/// 361a599; record 0004). The path to `dir` is the reader's and may pass
/// through links. A file or folder that cannot be read is refused naming
/// its path within the package, and a `dir` that cannot be read naming
/// `dir`. Reading changes nothing (93; core.1).
pub fn read_package_dir(dir: &Path, held: &Identity) -> Result<Package, Refusal> {
    let shown = dir.display().to_string();
    match fs::metadata(dir) {
        Ok(metadata) if metadata.is_dir() => {}
        Ok(_) => return Err(refused(shown, "is not a folder, and a package is a folder")),
        Err(error) => return Err(refused(shown, format!("could not be read: {error}"))),
    }
    let mut walked = Walked::default();
    if let Err(error) = walk(dir, &[], &mut walked) {
        return Err(refused(shown, format!("could not be read: {error}")));
    }
    let files = walked
        .files
        .iter()
        .map(|(path, bytes)| (path.as_slice(), bytes.as_slice()))
        .collect();
    read(files, &walked, held)
}

/// What a walk of a package's folder found, each by its path within the
/// package: its regular files with their bytes, its folders, and every
/// entry it refuses, a link or one that could not be read, with why.
#[derive(Debug, Default)]
struct Walked {
    files: Vec<(Vec<u8>, Vec<u8>)>,
    folders: Vec<Vec<u8>>,
    refused: Vec<(Vec<u8>, String)>,
}

/// Walks the folder at `dir`, `within` the package, into `walked`, reading
/// each entry's own type and following no link. The error is that the
/// folder could not be listed.
fn walk(dir: &Path, within: &[u8], walked: &mut Walked) -> std::io::Result<()> {
    for entry in fs::read_dir(dir)? {
        let entry = entry?;
        let path = joined(within, entry.file_name().as_encoded_bytes());
        match entry.file_type() {
            Ok(kind) if kind.is_symlink() => walked.refused.push((
                path,
                "is a symbolic link, and a package holds none".to_owned(),
            )),
            Ok(kind) if kind.is_dir() => {
                walked.folders.push(path.clone());
                if let Err(error) = walk(&entry.path(), &path, walked) {
                    walked
                        .refused
                        .push((path, format!("could not be read: {error}")));
                }
            }
            Ok(kind) if kind.is_file() => match fs::read(entry.path()) {
                Ok(bytes) => walked.files.push((path, bytes)),
                Err(error) => walked
                    .refused
                    .push((path, format!("could not be read: {error}"))),
            },
            // A fifo, a socket or a device is no part, and C.4 hashes
            // regular files alone.
            Ok(_) => {}
            Err(error) => walked
                .refused
                .push((path, format!("could not be read: {error}"))),
        }
    }
    Ok(())
}

/// The path of `name` in the folder at `within`.
fn joined(within: &[u8], name: &[u8]) -> Vec<u8> {
    if within.is_empty() {
        name.to_vec()
    } else {
        [within, b"/", name].concat()
    }
}

/// The one reading of a package, whether its files were given or walked
/// (92), checking in the order design 4 gives: the entries a walk
/// refused, the paths given, `kind.toml`, its one kind and no other, the
/// identity held at, and every part a stage names.
fn read(
    mut given: Vec<(&[u8], &[u8])>,
    walked: &Walked,
    held: &Identity,
) -> Result<Package, Refusal> {
    let mut problems = Problems::default();

    let mut refused: Vec<&(Vec<u8>, String)> = walked.refused.iter().collect();
    refused.sort();
    for (path, reason) in &refused {
        problems.refuse(&shown(path), reason.as_str());
    }
    let named: BTreeSet<&[u8]> = refused.iter().map(|(path, _)| path.as_slice()).collect();

    // One file at each path, each path within the package (90, 92).
    given.sort_by_key(|(path, _)| *path);
    let mut files: BTreeMap<&[u8], &[u8]> = BTreeMap::new();
    let mut twice = BTreeSet::new();
    for (path, bytes) in given {
        if let Some(fault) = path_fault(path) {
            problems.refuse(
                &shown(path),
                format!("{fault}, and a path given is {A_PATH}"),
            );
        } else if files.insert(path, bytes).is_some() && twice.insert(path) {
            problems.refuse(
                &shown(path),
                "is given twice, and a package holds one file at each path",
            );
        }
    }
    for path in files.keys() {
        if let Some(file) = folders_of(path).find(|folder| files.contains_key(folder)) {
            problems.refuse(
                &shown(path),
                format!(
                    "lies beneath `{}`, a file of the package, and no file lies beneath a file",
                    shown(file)
                ),
            );
        }
    }
    let folders: BTreeSet<&[u8]> = files
        .keys()
        .flat_map(|path| folders_of(path))
        .chain(walked.folders.iter().map(Vec::as_slice))
        .collect();
    let holds = Holds {
        files: &files,
        folders: &folders,
        named: &named,
    };

    // `kind.toml` at the root, a regular file, read by `read-kinds`, whose
    // problems are the package's (57, 89).
    let declaration = match files.get(KIND_TOML.as_bytes()) {
        Some(bytes) => match std::str::from_utf8(bytes) {
            Ok(text) => match read_kinds(text) {
                Ok(declaration) => Some(declaration),
                Err(refusal) => {
                    problems.absorb(refusal);
                    None
                }
            },
            Err(_) => {
                problems.refuse(KIND_TOML, "is not UTF-8 text, and a declaration is TOML");
                None
            }
        },
        None => {
            let fault = match holds.find(KIND_TOML) {
                Found::Folder => Some("is a folder"),
                Found::Nothing => Some("is missing"),
                // A link, or an entry not read, is named already.
                Found::File | Found::Named => None,
            };
            if let Some(fault) = fault {
                problems.refuse(
                    KIND_TOML,
                    format!(
                        "{fault}, and a package declares its kind in `kind.toml`, a regular file at its root"
                    ),
                );
            }
            None
        }
    };

    if let Some(declaration) = &declaration {
        // Exactly one kind, and settings given to none other, which would
        // name a kind outside the package (57, 89).
        let names: Vec<_> = declaration
            .kinds
            .keys()
            .map(|name| format!("`{name}`"))
            .collect();
        match names.len() {
            1 => {}
            0 => problems.refuse(
                "kinds",
                "declares no kind, and a package declares exactly one kind",
            ),
            n => problems.refuse(
                "kinds",
                format!(
                    "declares {n} kinds, {}, and a package declares exactly one kind",
                    names.join(" and ")
                ),
            ),
        }
        for name in declaration.settings.keys() {
            problems.refuse(
                &format!("kinds.{}", key(name)),
                format!(
                    "gives settings to `{name}`, a kind the package does not declare, and a package names no kind but its own"
                ),
            );
        }

        // The identity held at, its name and version the declaration's
        // (89; core.43).
        if let [kind] = declaration.kinds.values().collect::<Vec<_>>()[..] {
            let at = format!("kinds.{}", key(&kind.name));
            let declared = format!("{}/{}@{}", held.source(), kind.name, kind.version);
            let reason = format!(
                "makes the package `{declared}`, but it is held at `{held}`, and a package is held at its declaration's name and version"
            );
            if held.name() != kind.name {
                problems.refuse(&at, reason);
            } else if held.version() != kind.version {
                problems.refuse(&field(&at, "version"), reason);
            }
        }

        // Every part a stage names, a regular file inside the package (89;
        // C.4 as amended at blueprints 361a599; S17).
        for kind in declaration.kinds.values() {
            let stages = field(&format!("kinds.{}", key(&kind.name)), "stages");
            for (n, stage) in kind.stages.iter().enumerate() {
                let parts = [
                    ("prompt", Some(&stage.prompt)),
                    ("standing", stage.standing.as_ref()),
                ];
                for (name, part) in parts {
                    let Some(part) = part else { continue };
                    if let Some(fault) = holds.part_fault(part) {
                        problems.refuse(
                            &field(&format!("{stages}[{n}]"), name),
                            format!("`{part}` {fault}"),
                        );
                    }
                }
            }
        }
    }

    match (problems.finish(()), declaration) {
        (Err(refusal), _) => Err(refusal),
        (Ok(()), Some(declaration)) => {
            let kind = declaration
                .kinds
                .into_values()
                .next()
                .expect("a package read declares one kind");
            Ok(Package {
                identity: held.clone(),
                kind,
                hash: ContentHash::of(files),
                sections: declaration.sections,
            })
        }
        (Ok(()), None) => {
            unreachable!("a package whose declaration is not read has a problem naming why")
        }
    }
}

/// What a path given is, as a reason that refuses one says it.
const A_PATH: &str =
    "a file's path within the package, `/` between segments none of which is empty, `.` or `..`";

/// Why `path`, given as a file's path within a package, is none, or `None`
/// when it is one.
fn path_fault(path: &[u8]) -> Option<&'static str> {
    if path.is_empty() {
        return Some("is empty");
    }
    if path.starts_with(b"/") {
        return Some("is absolute");
    }
    path.split(|byte| *byte == b'/')
        .find_map(|segment| match segment {
            b"" => Some("holds an empty segment"),
            b"." => Some("holds a `.` segment"),
            b".." => Some("holds a `..` segment"),
            _ => None,
        })
}

/// Every folder `path` lies in within the package, its root left out:
/// `a` and `a/b` for `a/b/c.md`.
fn folders_of(path: &[u8]) -> impl Iterator<Item = &[u8]> {
    path.iter()
        .enumerate()
        .filter(|(_, byte)| **byte == b'/')
        .map(move |(at, _)| &path[..at])
}

/// What a package holds, to say what a path within it names.
struct Holds<'a> {
    files: &'a BTreeMap<&'a [u8], &'a [u8]>,
    folders: &'a BTreeSet<&'a [u8]>,
    /// The entries refused already, a link or one not read, each named by
    /// its own path.
    named: &'a BTreeSet<&'a [u8]>,
}

/// What a path within a package names.
#[derive(Debug, PartialEq, Eq)]
enum Found {
    /// A regular file of the package.
    File,
    /// An entry refused already, or one beneath it, which is named once,
    /// by its own path.
    Named,
    /// A folder of the package.
    Folder,
    /// Nothing the package holds.
    Nothing,
}

impl Holds<'_> {
    fn find(&self, path: &str) -> Found {
        let path = path.as_bytes();
        if self.files.contains_key(path) {
            Found::File
        } else if self.named.contains(path) || folders_of(path).any(|f| self.named.contains(f)) {
            Found::Named
        } else if self.folders.contains(path) {
            Found::Folder
        } else {
            Found::Nothing
        }
    }

    /// Why a part a stage names, `prompt` or `standing`, is no regular
    /// file inside the package, or `None` when it is one, or names an
    /// entry refused already: relative to `kind.toml`, no segment `..`,
    /// and a `.` or empty segment the folder it stands in (89; C.4 as
    /// amended at blueprints 361a599).
    fn part_fault(&self, part: &str) -> Option<&'static str> {
        if part.starts_with('/') {
            return Some("is absolute, and a part's path is relative to `kind.toml`");
        }
        if part.split('/').any(|segment| segment == "..") {
            return Some("holds a `..` segment, and nothing a package names lies outside it");
        }
        let within = part
            .split('/')
            .filter(|segment| !segment.is_empty() && *segment != ".")
            .collect::<Vec<_>>()
            .join("/");
        if within.is_empty() {
            return Some(
                "names the package's own folder, and a part is a regular file of the package",
            );
        }
        match self.find(&within) {
            Found::File | Found::Named => None,
            Found::Folder => Some("names a folder, and a part is a regular file of the package"),
            Found::Nothing => {
                Some("names no file the package holds, and a package holds every part it names")
            }
        }
    }
}

/// A path within a package as a problem names it, its bytes read as UTF-8
/// where they are not.
fn shown(path: &[u8]) -> String {
    if path.is_empty() {
        "\"\"".to_owned()
    } else {
        String::from_utf8_lossy(path).into_owned()
    }
}

/// A package refused for one reason, at `field`.
fn refused(field: String, reason: impl Into<String>) -> Refusal {
    Refusal {
        problems: vec![Problem {
            field,
            reason: reason.into(),
        }],
    }
}
