//! The kind format, read and checked (51 to 58, 81, 82; C.4).
//!
//! A declaration is TOML. [`read_kinds`] is `read-kinds` (81): given a
//! declaration's text, it returns every kind under its `kinds` table, each
//! with its stages, its agents' settings and the sections the format does
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
//! The values of the sections a reader does not know are [`toml`]'s,
//! re-exported so that a caller names the same version.

#![warn(missing_docs)]

mod deliverable;
mod elaboration;
mod fields;
mod kinds;
mod quiet;
mod refusal;

pub use deliverable::{Deliverable, End};
pub use elaboration::{read_elaboration, Elaboration, Ending};
pub use kinds::{read_kinds, Declaration, Kind, KindAgents, KindSettings, Stage};
pub use refusal::{Problem, Refusal};
/// The TOML crate whose values a section the format does not name holds
/// (56), so that a caller reads them with the same version.
pub use toml;
