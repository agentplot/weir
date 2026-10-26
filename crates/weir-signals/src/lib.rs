//! The inbox of the statement in agentplot/blueprints
//! `design/weir/requirements.md`: the records an inbox keeps beside its
//! captures, where each lies, and every write it takes, fixed once so that
//! every host and tool reads and writes one inbox the same way (40 to 43).
//!
//! An inbox is read whole from the files of one tip, which the caller hands
//! in, and every operation forms its write from that read and checks it
//! there, so the write protocol can form it again on a new tip and every
//! check is made again (46). The crate reads no repository and runs no
//! program: every function answers from its arguments alone (core.1).
