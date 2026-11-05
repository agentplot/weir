//! The kind format, read and checked (51 to 58, 81, 82; C.4), and a
//! kind's identity and its package (88 to 90, 92; C.4).
//!
//! A declaration is TOML. [`read_kinds`] is `read-kinds` (81): given a
//! declaration's text, it returns every kind under its `kinds` table, each
//! with its stages, whether each repeats, its agents' settings and the sections the format does
//! not name, and apart from them the settings given to kinds the
//! declaration does not declare, or refuses the declaration whole, naming
//! every field that fails it (52, 57, C.4). An
//! elaboration kind's section is one of those sections: whoever runs
//! places never looks inside it, and whoever runs elaborations reads it
//! with [`read_elaboration`] (55, 56).
//!
//! A read depends on the text alone and changes nothing (58, 82). Which
//! declaration a kind's name resolves to, among a tool's own, a
//! repository's and a launch's, and how settings given to a kind declared
//! elsewhere join that kind's own, are the caller's rule, never this
//! crate's (58, core.32).
//!
//! A kind's identity is its source, its name and its version: an
//! [`Identity`] reads from and spells `<source>/<name>@<version>`, and a
//! [`NamedKind`] is a kind's name with its source or its version left out,
//! as a reader is given it (88, C.4; core.42). [`check_name`] is the one
//! check of a source's or a kind's name, and [`REPO`] and [`LAUNCH`] are
//! the sources of a repository's and a launch's kinds.
//!
//! A package holds one kind at one version with every part it names.
//! [`read_package`] is `read-package` (92) over a package's files given as
//! paths and bytes, and [`read_package_dir`] over a folder, walked without
//! following a link: given the identity it is held at, each gives the
//! package's one kind with that identity and its content hash, or refuses
//! the package whole, naming every field (89, 92; C.4; core.43). A
//! [`ContentHash`] is C.4's, computed by [`ContentHash::of`] alone over
//! every regular file and its path within the package, through
//! weir-capture's sha256, so where a package lies and when it was written
//! never reach it (90; core.44).
//!
//! The values of the sections a reader does not know are [`toml`]'s,
//! re-exported so that a caller names the same version.

#![warn(missing_docs)]

mod deliverable;
mod elaboration;
mod fields;
mod hash;
mod identity;
mod kinds;
mod name;
mod package;
mod quiet;
mod refusal;
mod repeat;

pub use deliverable::{Deliverable, End};
pub use elaboration::{read_elaboration, Elaboration, Ending};
pub use hash::{ContentHash, HashRefusal};
pub use identity::{Identity, NamedKind, LAUNCH, REPO};
pub use kinds::{read_kinds, Declaration, Kind, KindAgents, KindSettings, Stage};
pub use name::{check_name, NameRefusal};
pub use package::{read_package, read_package_dir, Package};
pub use refusal::{Problem, Refusal};
pub use repeat::Repeat;
/// The TOML crate whose values a section the format does not name holds
/// (56), so that a caller reads them with the same version.
pub use toml;
