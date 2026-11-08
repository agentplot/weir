//! Deliveries (A.14, B.7; C.8): one message a deliverer owes one or more
//! readers, recorded once under its key and owed to each reader until that
//! reader confirms it by its key (100 to 107).
//!
//! The crate holds the four records of C.8, `weir-delivery/1`,
//! `weir-handover/1`, `weir-confirmation/1` and `weir-known/1`; their two
//! spellings, recutils on a shared branch and JSON on a machine; the calls
//! `record`, `hand`, `confirm` and `owed` (108 to 111); and the store seam a
//! deliverer implements over its own store (112).
//!
//! Every call answers from the records, the arguments and the times its
//! caller passes, and returns the records to add without writing them. The
//! crate reads no clock, holds no store, hands no delivery over and reads
//! no message's words (69, 70, 113; core.1, core.4, core.51).
#![deny(missing_docs)]

mod calls;
mod record;
mod text;

pub use calls::Refusal;
pub use record::{
    Confirmation, Delivery, Handover, Known, Record, CONFIRMATION, DELIVERY, HANDOVER, KNOWN,
};
pub use text::{Key, Message, Reader, Span, Time, Way};
