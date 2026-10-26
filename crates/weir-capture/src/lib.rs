//! The capture contract of the statement in agentplot/blueprints
//! `design/weir/requirements.md`: what an adapter tells a host, and what a
//! host does with it, fixed once so that every host given the same lines
//! writes the same records (24).
//!
//! The crate holds an adapter's lines, offers, cursors and completions, read
//! and written as JSON Lines (1 to 10; C.1); the key rule (3, 4); an
//! adapter's declaration, its configuration's schema and the command line a
//! run is started with (11, 12, 72 to 74); the sha256 content hash the
//! built-in adapters key files by (C.6); the cut of an own excerpt (20);
//! the capture's record, `weir-capture/1`, in recutils (16, 17, 22; C.2);
//! and `take`, the seven steps a host takes lines in, through the inbox and
//! the raw bank the host hands it (13 to 24, 75 to 77; C.1).
//!
//! Every function answers from its arguments alone: the machine's name, the
//! taken-at time, the limit and the raw place are the caller's to hand in,
//! and every write is one the caller's inbox or bank makes (core.1).

pub mod key;

pub use key::{Key, KeyError};
