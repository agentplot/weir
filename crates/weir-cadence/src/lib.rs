//! Cadence: whether a scheduled thing is due (59), caught up once after a
//! machine has slept (60), read from a five-field cron schedule in the
//! machine's local time (C.5), with the clock as its one seam (83 to 85).
//!
//! The caller keeps the last run of each thing; this crate keeps nothing.
#![deny(missing_docs)]
