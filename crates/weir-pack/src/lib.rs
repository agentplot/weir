//! The conformance pack of the statement in agentplot/blueprints
//! `design/weir/requirements.md`: the cases every host, adapter and profile
//! of the family is held to, the same on every machine, with no source and
//! no network (65 to 68, 87; C.7).
//!
//! The crate holds the pack's files, built in, read the same way as a
//! folder its caller names; the index of the statement's clause numbers
//! the pack keeps; the pack's check, which fails when a case cites a
//! number that names no clause or a counted clause is cited by no case
//! (67); the adapter checks an adapter's lines must pass and the meeting
//! checks a meeting adapter passes (65, 66), with the adapter cases that
//! fix their verdicts; and the host cases, read, and compared with the
//! report a host prints and the records at its branch's tip (65, 87).
//!
//! It runs no program and reads no repository: `weir conform` makes the
//! repository a host is run against, runs the host and reads the tip, and
//! hands what it found here to compare. Every function answers from its
//! arguments alone (core.1).

pub mod adapter;
pub mod adapter_case;
pub mod check;
pub mod clauses;
pub mod pack;

pub use adapter::{check_lines, check_meeting_lines, Breach, Meetings, RawPlace};
pub use adapter_case::{
    read_adapter_case, AdapterCase, BuiltIn, BuiltInCase, ExpectedBreach, LinesCase,
};
pub use check::{check, Checked, Fault};
pub use clauses::{read_index, Clause, Index, Section};
pub use pack::{Case, Pack, Shape};
