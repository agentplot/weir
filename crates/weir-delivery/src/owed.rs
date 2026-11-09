//! `owed` (111): what each reader is owed, read from a store's records and
//! a time alone (104 to 107; I9; core.49, core.50, core.51).

use crate::calls::Refusal;
use crate::record::{Delivery, Handover, Record};
use crate::text::{Key, Reader, Time};

/// What `owed` gives: each reader with something it has not confirmed, and
/// every delivery done (111).
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Owed {
    /// Each reader owed a delivery, in the order a delivery was first owed
    /// to it as the records hold them.
    pub readers: Vec<ReaderOwed>,
    /// The key of each delivery every one of whose readers has confirmed
    /// it, in the order recorded (107).
    pub done: Vec<Key>,
}

/// One reader and every delivery it has not confirmed (105).
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ReaderOwed {
    /// The reader, as its deliverer names it.
    pub reader: Reader,
    /// Whether a `weir-known/1` record names it: whether a reader has ever
    /// confirmed under this name (106; core.50).
    pub known: bool,
    /// Every delivery naming it that it has not confirmed, in the order
    /// recorded.
    pub deliveries: Vec<OwedDelivery>,
}

/// One delivery a reader has not confirmed, owed or expired (104, 105).
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct OwedDelivery {
    /// The delivery, as recorded.
    pub delivery: Delivery,
    /// Whether the time asked at is at or past its `expires_at` (104).
    pub expired: bool,
    /// Each hand-over of it to this reader, in the order added.
    pub handovers: Vec<Handover>,
}

/// `owed` (111): every delivery each reader has not confirmed, owed or
/// expired at `now`, with its hand-overs to that reader, whether the
/// reader's name is known, and every delivery done; or a refusal naming
/// `now` when it is not a time. It reads the deliveries in the order the
/// records hold them, the first under each key alone (101), and leaves out
/// nothing a reader has not confirmed (105; I9). A hand-over or a
/// confirmation naming no delivery of the records is passed over.
pub fn owed(records: &[Record], now: &str) -> Result<Owed, Refusal> {
    let now = Time::parse(now, "now")?;
    let mut deliveries: Vec<&Delivery> = Vec::new();
    let mut handovers: Vec<&Handover> = Vec::new();
    let mut confirmed: Vec<(&Key, &Reader)> = Vec::new();
    let mut known: Vec<&Reader> = Vec::new();
    for record in records {
        match record {
            Record::Delivery(delivery) => {
                if !deliveries.iter().any(|held| held.key() == delivery.key()) {
                    deliveries.push(delivery);
                }
            }
            Record::Handover(handover) => handovers.push(handover),
            Record::Confirmation(confirmation) => {
                confirmed.push((&confirmation.key, &confirmation.reader));
            }
            Record::Known(name) => known.push(&name.reader),
        }
    }
    let mut readers: Vec<ReaderOwed> = Vec::new();
    let mut done = Vec::new();
    for delivery in deliveries {
        let mut every_reader_confirmed = true;
        for reader in delivery.readers() {
            if confirmed.contains(&(delivery.key(), reader)) {
                continue;
            }
            every_reader_confirmed = false;
            let owed = OwedDelivery {
                delivery: delivery.clone(),
                expired: now.instant() >= delivery.expires_at().instant(),
                handovers: handovers
                    .iter()
                    .filter(|handover| {
                        handover.key == *delivery.key() && handover.reader == *reader
                    })
                    .map(|handover| (*handover).clone())
                    .collect(),
            };
            match readers.iter_mut().find(|owed| owed.reader == *reader) {
                Some(owed_to) => owed_to.deliveries.push(owed),
                None => readers.push(ReaderOwed {
                    reader: reader.clone(),
                    known: known.contains(&reader),
                    deliveries: vec![owed],
                }),
            }
        }
        if every_reader_confirmed {
            done.push(delivery.key().clone());
        }
    }
    Ok(Owed { readers, done })
}
