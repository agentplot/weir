//! The store seam (112): a deliverer's store of records, held to the pack
//! through its two calls alone (113; core.52). weir implements it for no
//! store; a deliverer implements it over its own, which lies where its own
//! statement says (C.8; core.51).

use std::fmt::Display;

use crate::record::Record;

/// A deliverer's store of delivery records (112).
pub trait Store {
    /// Why a read or an add failed, in words the pack shows.
    type Error: Display;

    /// Every record the store holds, in the order added, each as it was
    /// given (112, 113). Reading changes nothing.
    fn read(&self) -> Result<Vec<Record>, Self::Error>;

    /// Holds `records` after the others, in the order given: every one or
    /// none, saying why where it failed (112). A store drops none of them
    /// but those of a done delivery its deliverer lets go of whole (107,
    /// 113).
    fn add(&mut self, records: &[Record]) -> Result<(), Self::Error>;
}
