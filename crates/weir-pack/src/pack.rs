//! The pack's files and its cases: one folder per case under
//! `<shape>/<case>/`, the shapes the six C.7 names (87; C.7). The pack is
//! read the same way whether it is the one built into the crate or a
//! folder the caller names.

use std::collections::BTreeMap;
use std::fmt;
use std::fs;
use std::io;
use std::path::Path;

mod embedded {
    include!(concat!(env!("OUT_DIR"), "/pack.rs"));
}

/// The shapes the pack holds, each a folder of its root (87; C.7).
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Shape {
    Host,
    Adapter,
    Cadence,
    Branch,
    Inbox,
    Kinds,
}

impl Shape {
    pub const ALL: [Shape; 6] = [
        Shape::Host,
        Shape::Adapter,
        Shape::Cadence,
        Shape::Branch,
        Shape::Inbox,
        Shape::Kinds,
    ];

    /// The folder its cases lie in.
    pub fn as_str(self) -> &'static str {
        match self {
            Shape::Host => "host",
            Shape::Adapter => "adapter",
            Shape::Cadence => "cadence",
            Shape::Branch => "branch",
            Shape::Inbox => "inbox",
            Shape::Kinds => "kinds",
        }
    }

    pub fn parse(text: &str) -> Option<Shape> {
        Shape::ALL.into_iter().find(|shape| shape.as_str() == text)
    }
}

impl fmt::Display for Shape {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A pack: every file, by its path from the pack's root, `/` between its
/// folders, with its bytes.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Pack {
    files: BTreeMap<String, Vec<u8>>,
}

/// One case: its shape, its folder's name, and its files by their paths
/// from that folder.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Case {
    pub shape: Shape,
    pub name: String,
    pub files: BTreeMap<String, Vec<u8>>,
}

impl Pack {
    /// The pack built into the crate: weir's `pack/` as it stood when the
    /// crate was built.
    pub fn embedded() -> Pack {
        Pack::from_files(
            embedded::FILES
                .iter()
                .map(|(path, bytes)| (path.to_string(), bytes.to_vec())),
        )
    }

    /// The pack of `files`, each by its path from the pack's root.
    pub fn from_files(files: impl IntoIterator<Item = (String, Vec<u8>)>) -> Pack {
        Pack {
            files: files.into_iter().collect(),
        }
    }

    /// The pack the folder `dir` holds, read as the built-in pack was: every
    /// file under it, a name beginning with `.` passed over; or why a file
    /// could not be read, naming it.
    pub fn read(dir: &Path) -> Result<Pack, String> {
        let mut files = BTreeMap::new();
        walk(dir, "", &mut files)
            .map_err(|(path, error)| format!("the pack's {path} cannot be read: {error}"))?;
        Ok(Pack { files })
    }

    pub fn files(&self) -> &BTreeMap<String, Vec<u8>> {
        &self.files
    }

    pub fn file(&self, path: &str) -> Option<&[u8]> {
        self.files.get(path).map(Vec::as_slice)
    }

    /// Every case of `shape`, in the order of their names: each folder
    /// under the shape's folder, with the files under it. A file lying in
    /// the shape's folder itself is no case; the pack's check names it.
    pub fn cases(&self, shape: Shape) -> Vec<Case> {
        let prefix = format!("{shape}/");
        let mut cases: BTreeMap<&str, BTreeMap<String, Vec<u8>>> = BTreeMap::new();
        for (path, bytes) in self.files.range(prefix.clone()..) {
            let Some(rest) = path.strip_prefix(&prefix) else {
                break;
            };
            if let Some((name, file)) = rest.split_once('/') {
                cases
                    .entry(name)
                    .or_default()
                    .insert(file.to_string(), bytes.clone());
            }
        }
        cases
            .into_iter()
            .map(|(name, files)| Case {
                shape,
                name: name.to_string(),
                files,
            })
            .collect()
    }

    /// The case `<shape>/<name>`, where the pack holds it.
    pub fn case(&self, shape: Shape, name: &str) -> Option<Case> {
        self.cases(shape).into_iter().find(|case| case.name == name)
    }
}

impl Case {
    /// `<shape>/<name>`, as the pack's check and every refusal name it.
    pub fn id(&self) -> String {
        format!("{}/{}", self.shape, self.name)
    }

    pub fn file(&self, path: &str) -> Option<&[u8]> {
        self.files.get(path).map(Vec::as_slice)
    }

    /// The text of the file at `path`, or why there is none: missing, or
    /// not UTF-8.
    pub fn text(&self, path: &str) -> Result<&str, String> {
        let bytes = self
            .file(path)
            .ok_or_else(|| format!("{}: holds no `{path}`", self.id()))?;
        std::str::from_utf8(bytes).map_err(|_| format!("{}: `{path}` is not UTF-8", self.id()))
    }

    /// The files under the folder `folder` of the case, by their paths from
    /// it: `start/inbox/x` as `inbox/x`.
    pub fn under(&self, folder: &str) -> BTreeMap<String, Vec<u8>> {
        let prefix = format!("{folder}/");
        self.files
            .iter()
            .filter_map(|(path, bytes)| {
                path.strip_prefix(&prefix)
                    .map(|rest| (rest.to_string(), bytes.clone()))
            })
            .collect()
    }

    /// Writes every file of the case into `dir`, by its path, so a program
    /// the case names can be run there: a script, `*.sh`, is written so it
    /// can be run. The write is the caller's to ask for (core.1).
    pub fn write_to(&self, dir: &Path) -> io::Result<()> {
        for (path, bytes) in &self.files {
            let file = dir.join(path);
            if let Some(parent) = file.parent() {
                fs::create_dir_all(parent)?;
            }
            fs::write(&file, bytes)?;
            #[cfg(unix)]
            if path.ends_with(".sh") {
                use std::os::unix::fs::PermissionsExt;
                fs::set_permissions(&file, fs::Permissions::from_mode(0o755))?;
            }
        }
        Ok(())
    }
}

/// Every file under `dir`, each by `prefix` and its path from `dir`.
fn walk(
    dir: &Path,
    prefix: &str,
    files: &mut BTreeMap<String, Vec<u8>>,
) -> Result<(), (String, io::Error)> {
    let named = |error| (format!("`{prefix}`"), error);
    let mut entries = Vec::new();
    for entry in fs::read_dir(dir).map_err(named)? {
        entries.push(entry.map_err(named)?);
    }
    for entry in entries {
        let name = entry.file_name().to_string_lossy().into_owned();
        if name.starts_with('.') {
            continue;
        }
        let path = format!("{prefix}{name}");
        let file = entry.path();
        if file.is_dir() {
            walk(&file, &format!("{path}/"), files)?;
        } else {
            let bytes = fs::read(&file).map_err(|error| (format!("`{path}`"), error))?;
            files.insert(path, bytes);
        }
    }
    Ok(())
}
