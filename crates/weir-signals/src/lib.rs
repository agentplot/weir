//! The inbox of the statement in agentplot/blueprints
//! `design/weir/requirements.md`: the records an inbox keeps beside its
//! captures, where each lies, and every write it takes, fixed once so that
//! every host and tool reads and writes one inbox the same way (40 to 43).
//!
//! The crate holds the layout, where each record lies under `inbox/` (42;
//! C.2); each record's format and major (40); the signal and the reading
//! (25 to 28); the move and the batch (29 to 33); the item, its brief and
//! its sends (34 to 36); the inbox read whole from one tip (43); the check
//! every write passes (41, 46); and the import operation (37 to 39, 75a).
//!
//! An inbox is read whole from the files of one tip, which the caller hands
//! in, and every operation forms its write from that read and checks it
//! there, so the write protocol can form it again on a new tip and every
//! check is made again (46). The crate reads no repository and runs no
//! program: every function answers from its arguments alone (core.1).

pub mod format;
pub mod import;
pub mod inbox;
pub mod item;
pub mod layout;
pub mod moves;
pub mod signal;
pub mod write;

pub use format::{Format, FormatError};
pub use import::{
    CaptureRefused, ImportReport, Imported, ImportedCapture, ImportedMove, ImportedSignal,
    MoveNotTaken,
};
pub use inbox::{InboxAt, InboxError, ItemAt, Unmoved, Weight};
pub use item::{Item, MadeBy, Outcome, SendRecord};
pub use layout::Place;
pub use moves::{Batch, Decision, Move, NewMove, Proposal, Word};
pub use signal::{ReadSignal, Signal, SignalId};
pub use write::{NotTaken, Refused, Write};
