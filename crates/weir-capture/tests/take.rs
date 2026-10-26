//! Taking lines through a host's inbox and bank, held to the scenarios of
//! capture-take, with in-memory fakes of both.

mod fakes;

use time::format_description::well_known::Rfc3339;
use time::OffsetDateTime;
use weir_capture::take::{Bank, Inbox, RawFile, Take, TakeError, Taken};
use weir_capture::{CameIn, Capture, Key, SMALLEST_LIMIT};

use fakes::{host, Call};

const LIMIT: usize = 8192;

fn at(time: &str) -> OffsetDateTime {
    OffsetDateTime::parse(time, &Rfc3339).unwrap()
}

fn take_on(machine: &str) -> Take {
    Take::new(
        machine,
        at("2026-10-17T06:30:05+02:00"),
        LIMIT,
        CameIn::Run("releases".into()),
    )
    .unwrap()
}

#[test]
fn a_take_is_made_from_what_the_host_hands_in() {
    let take = take_on("deneb");
    assert_eq!(take.machine(), "deneb");
    assert_eq!(take.taken_at(), "2026-10-17T04:30:05Z");
    assert_eq!(take.limit(), LIMIT);
    assert_eq!(take.came_in(), &CameIn::Run("releases".into()));
}

#[test]
fn a_limit_below_the_widest_closing_line_is_refused() {
    let at = at("2026-10-17T04:30:05Z");
    assert_eq!(
        Take::new("deneb", at, SMALLEST_LIMIT - 1, CameIn::Intake),
        Err(TakeError::Limit(SMALLEST_LIMIT - 1))
    );
    assert!(Take::new("deneb", at, SMALLEST_LIMIT, CameIn::Intake).is_ok());
    for machine in ["", "deneb:1", "de neb"] {
        assert_eq!(
            Take::new(machine, at, LIMIT, CameIn::Intake),
            Err(TakeError::Machine(machine.into()))
        );
    }
}

#[test]
fn the_fakes_record_every_call_in_order() {
    let (mut inbox, mut bank, calls) = host();
    let key = Key::parse("release/x").unwrap();
    inbox.held.insert("release/held".into());
    assert!(!inbox.holds(&key));
    assert!(inbox.holds(&Key::parse("release/held").unwrap()));
    let files = [RawFile {
        path: "notes.md".into(),
        bytes: b"notes".to_vec(),
    }];
    assert_eq!(
        bank.bank(&key, &files, None),
        Ok("/data/inbox/release/x".into())
    );
    let taken = Taken {
        capture: Capture {
            key: key.clone(),
            source: "release".into(),
            event_at: "2026-10-10T14:02:11Z".into(),
            title: None,
            url: None,
            pointer: Some("deneb:/data/inbox/release/x".into()),
            came_in: CameIn::Intake,
            machine: "deneb".into(),
            taken_at: "2026-10-17T04:30:05Z".into(),
            unread: true,
        },
        own_signal: None,
    };
    inbox
        .refuse
        .insert("release/x".into(), "the tip moved".into());
    assert!(inbox.write(&taken).is_err());
    bank.discard(&key);
    assert!(bank.banked.is_empty());
    assert_eq!(
        *calls.borrow(),
        [
            Call::Holds("release/x".into()),
            Call::Holds("release/held".into()),
            Call::Bank("release/x".into()),
            Call::Write("release/x".into()),
            Call::Discard("release/x".into()),
        ]
    );
}
