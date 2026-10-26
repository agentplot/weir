//! In-memory fakes of a host's inbox and raw bank, each recording every
//! call, in order, in one list the two share, so a test can say which
//! came first.

use std::cell::RefCell;
use std::collections::{BTreeMap, BTreeSet};
use std::rc::Rc;

use weir_capture::take::{Bank, Inbox, RawFile, Refusal, Taken};
use weir_capture::Key;

/// One call made of a fake, by the key it named.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Call {
    Holds(String),
    Bank(String),
    Write(String),
    Discard(String),
}

pub type Calls = Rc<RefCell<Vec<Call>>>;

/// An inbox that answers `holds` from the keys it held before the take,
/// as an inbox read from one tip does (43), and keeps every write.
pub struct FakeInbox {
    pub calls: Calls,
    pub held: BTreeSet<String>,
    pub written: Vec<Taken>,
    /// Keys whose write it refuses, with the reason it gives.
    pub refuse: BTreeMap<String, String>,
}

/// A bank that keeps what it is handed under `<root>/<key>`.
pub struct FakeBank {
    pub calls: Calls,
    pub root: String,
    pub banked: BTreeMap<String, Banked>,
    /// Refuses every bank, with this reason.
    pub refuse: Option<String>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Banked {
    pub files: Vec<RawFile>,
    pub whole_excerpt: Option<String>,
}

/// An inbox and a bank on one machine, sharing one record of calls.
pub fn host() -> (FakeInbox, FakeBank, Calls) {
    let calls = Calls::default();
    let inbox = FakeInbox {
        calls: calls.clone(),
        held: BTreeSet::new(),
        written: Vec::new(),
        refuse: BTreeMap::new(),
    };
    let bank = FakeBank {
        calls: calls.clone(),
        root: "/data/inbox".into(),
        banked: BTreeMap::new(),
        refuse: None,
    };
    (inbox, bank, calls)
}

impl Inbox for FakeInbox {
    fn holds(&self, key: &Key) -> bool {
        self.calls.borrow_mut().push(Call::Holds(key.to_string()));
        self.held.contains(key.as_str())
    }

    fn write(&mut self, taken: &Taken) -> Result<(), Refusal> {
        let key = taken.capture.key.to_string();
        self.calls.borrow_mut().push(Call::Write(key.clone()));
        if let Some(reason) = self.refuse.get(&key) {
            return Err(Refusal(reason.clone()));
        }
        self.written.push(taken.clone());
        Ok(())
    }
}

impl Bank for FakeBank {
    fn bank(
        &mut self,
        key: &Key,
        files: &[RawFile],
        whole_excerpt: Option<&str>,
    ) -> Result<String, Refusal> {
        self.calls.borrow_mut().push(Call::Bank(key.to_string()));
        if let Some(reason) = &self.refuse {
            return Err(Refusal(reason.clone()));
        }
        self.banked.insert(
            key.to_string(),
            Banked {
                files: files.to_vec(),
                whole_excerpt: whole_excerpt.map(str::to_string),
            },
        );
        Ok(format!("{}/{key}", self.root))
    }

    fn discard(&mut self, key: &Key) {
        self.calls.borrow_mut().push(Call::Discard(key.to_string()));
        self.banked.remove(key.as_str());
    }
}
