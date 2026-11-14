//! A caller's halt: one flag its clones share, which the caller sets from
//! any thread and the wait that reads a try's limit reads (record 0008 as
//! amended 2026-11-12; core.1, core.53). The crate holds none in a static:
//! a halt is its caller's alone.

use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;

/// A handle to one flag, shared by every clone of it: never set when made
/// by [`Halt::default`], set by [`Halt::halt`] for every clone and never
/// cleared, and read by [`Halt::halted`]. Two handles are equal when they
/// share one flag.
#[derive(Clone, Debug, Default)]
pub struct Halt(Arc<AtomicBool>);

impl Halt {
    /// Sets the flag, for this handle and every clone of it. It is never
    /// cleared.
    pub fn halt(&self) {
        self.0.store(true, Ordering::SeqCst);
    }

    /// Whether the flag is set.
    pub fn halted(&self) -> bool {
        self.0.load(Ordering::SeqCst)
    }
}

impl PartialEq for Halt {
    /// Whether the two handles share one flag.
    fn eq(&self, other: &Halt) -> bool {
        Arc::ptr_eq(&self.0, &other.0)
    }
}

impl Eq for Halt {}
