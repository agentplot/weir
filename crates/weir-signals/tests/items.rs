//! Items, held to the scenarios of inbox-items: made by a join or from a
//! conclusion, named once, their signals and weight read from the standing
//! moves (34, 35; core.20).

mod common;

use weir_capture::rec;
use weir_signals::{InboxAt, Item, MadeBy, NewMove, Outcome, SendRecord, Weight, Write};

use common::{
    a_move, after, capture, empty, moved, paths, recfix, recsel_value, signal, with_read,
    with_signals,
};

const AT: &str = "2026-10-25T09:00:00Z";

fn ids(inbox: &InboxAt, name: &str) -> Vec<String> {
    inbox
        .item_signals(name)
        .iter()
        .map(|id| id.to_string())
        .collect()
}

#[test]
fn an_item_from_a_conclusion() {
    let inbox = empty();
    let write = inbox
        .propose_item("pin-mise", "inquiry/toolchain#3", "operator", AT)
        .unwrap();
    assert_eq!(paths(&write), ["inbox/items/pin-mise/item.rec"]);
    let text = &write.files["inbox/items/pin-mise/item.rec"];
    assert_eq!(
        text,
        "format: weir-item/1\n\
         item: pin-mise\n\
         made_by: conclusion\n\
         conclusion: inquiry/toolchain#3\n\
         named_by: operator\n\
         named_at: 2026-10-25T09:00:00Z\n"
    );
    recfix(text);
    assert_eq!(recsel_value(text, "conclusion"), "inquiry/toolchain#3");
    let inbox = after(&inbox, &write);
    let item = inbox.item("pin-mise").unwrap();
    assert_eq!(
        item.item,
        Item {
            item: "pin-mise".into(),
            made_by: MadeBy::Conclusion("inquiry/toolchain#3".into()),
            named_by: "operator".into(),
            named_at: AT.into(),
        }
    );
    assert!(!item.is_sent());
    assert!(inbox.item_signals("pin-mise").is_empty());
    assert_eq!(
        inbox.weight("pin-mise"),
        Some(Weight {
            signals: 0,
            sources: Vec::new(),
            earliest: None,
            latest: None,
        })
    );
    assert_eq!(inbox.weight("no-such-item"), None);
}

#[test]
fn an_item_with_no_conclusion_named() {
    let refused = empty()
        .propose_item("pin-mise", "", "operator", AT)
        .unwrap_err();
    assert_eq!(refused.0[0].what, "pin-mise");
    assert_eq!(refused.0[0].field.as_deref(), Some("conclusion"));
    let refused = empty()
        .propose_item("pin-mise", "inquiry/toolchain#3", "", AT)
        .unwrap_err();
    assert_eq!(refused.0[0].field.as_deref(), Some("named_by"));
    let refused = empty()
        .propose_item("pin-mise", "inquiry/toolchain#3", "operator", "soon")
        .unwrap_err();
    assert_eq!(refused.0[0].field.as_deref(), Some("named_at"));
}

#[test]
fn a_name_already_had() {
    let inbox = with_signals(&empty(), "release/x", 1);
    let inbox = moved(
        &inbox,
        &a_move("release/x#1", "join", "shim-timeout", "curation", AT),
    );
    let refused = inbox
        .propose_item("shim-timeout", "inquiry/toolchain#3", "operator", AT)
        .unwrap_err();
    assert_eq!(refused.0[0].what, "shim-timeout");
    assert!(refused.0[0].why.contains("has had"), "{refused}");
}

#[test]
fn a_nested_name() {
    for name in ["shim/timeout", "", ".", ".."] {
        let refused = empty()
            .propose_item(name, "inquiry/toolchain#3", "operator", AT)
            .unwrap_err();
        assert_eq!(refused.0[0].what, name);
        assert_eq!(refused.0[0].field.as_deref(), Some("item"));
    }
}

#[test]
fn a_join_replaced() {
    let mut inbox = with_signals(&empty(), "release/x", 3);
    for n in 1..=3 {
        inbox = moved(
            &inbox,
            &a_move(
                &format!("release/x#{n}"),
                "join",
                "shim-timeout",
                "curation",
                AT,
            ),
        );
    }
    let item_rec = inbox.files()["inbox/items/shim-timeout/item.rec"].clone();
    assert_eq!(
        ids(&inbox, "shim-timeout"),
        ["release/x#1", "release/x#2", "release/x#3"]
    );
    let drop = NewMove {
        replaces: Some(1),
        ..a_move("release/x#2", "drop", "a duplicate", "operator", AT)
    };
    let write = inbox.make_move(&drop).unwrap();
    assert_eq!(paths(&write), ["inbox/moves/moves.rec"]);
    let inbox = after(&inbox, &write);
    assert_eq!(ids(&inbox, "shim-timeout"), ["release/x#1", "release/x#3"]);
    assert_eq!(inbox.files()["inbox/items/shim-timeout/item.rec"], item_rec);
    // No record holds the item's signals or its weight.
    for text in inbox.files().values() {
        assert!(!text.contains("weight"), "{text}");
    }
    assert!(!item_rec.contains("release/x"), "{item_rec}");
}

#[test]
fn an_attach_counts() {
    let inbox = with_signals(&empty(), "release/x", 2);
    let inbox = moved(
        &inbox,
        &a_move("release/x#1", "join", "shim-timeout", "curation", AT),
    );
    // Sent through a write formed elsewhere, as a caller might hand it.
    let mut write = Write::new();
    write.put(
        "inbox/items/shim-timeout/brief.md",
        "<!-- format: weir-brief/1 -->\nthe brief\n",
    );
    let send = SendRecord {
        item: "shim-timeout".into(),
        target: "agentplot/weir fix".into(),
        brief: Some("the brief\n".into()),
        sent_at: AT.into(),
        outcome: Outcome::Open,
    };
    write.put(
        "inbox/items/shim-timeout/sends.rec",
        rec::write(&[send.to_record()]),
    );
    let inbox = after(&inbox, &write);
    let inbox = moved(
        &inbox,
        &a_move("release/x#2", "attach", "shim-timeout", "curation", AT),
    );
    assert_eq!(ids(&inbox, "shim-timeout"), ["release/x#1", "release/x#2"]);
    assert_eq!(inbox.weight("shim-timeout").unwrap().signals, 2);
}

#[test]
fn weight_from_two_sources() {
    let at = |key: &str, event_at: &str| weir_capture::Capture {
        event_at: event_at.into(),
        ..capture(key)
    };
    let inbox = with_read(&empty(), at("release/a", "2026-10-01T08:00:00Z"), 1);
    let inbox = with_read(&inbox, at("release/b", "2026-10-10T08:00:00+02:00"), 1);
    let inbox = with_read(&inbox, at("meeting/file/ab12", "2026-10-05T10:00:00Z"), 1);
    let inbox = with_read(&inbox, at("release/c", "2026-09-01T00:00:00Z"), 1);
    let mut inbox = inbox;
    for id in ["release/a#1", "meeting/file/ab12#1", "release/b#1"] {
        inbox = moved(&inbox, &a_move(id, "join", "shim-timeout", "curation", AT));
    }
    // A signal joined elsewhere weighs nothing here.
    inbox = moved(
        &inbox,
        &a_move("release/c#1", "join", "elsewhere", "curation", AT),
    );
    assert_eq!(
        inbox.weight("shim-timeout"),
        Some(Weight {
            signals: 3,
            sources: vec!["meeting".into(), "release".into()],
            earliest: Some("2026-10-01T08:00:00Z".into()),
            latest: Some("2026-10-10T08:00:00+02:00".into()),
        })
    );
    assert!(inbox.standing(&signal("release/c#1")).is_some());
}

#[test]
fn an_item_record_out_of_shape_is_refused_naming_the_field() {
    let item = Item {
        item: "pin-mise".into(),
        made_by: MadeBy::Conclusion("inquiry/toolchain#3".into()),
        named_by: "operator".into(),
        named_at: AT.into(),
    };
    let written = rec::write(&[item.to_record()]);
    let read = |text: &str| Item::from_record(&rec::read(text).unwrap()[0]);
    assert_eq!(read(&written), Ok(item));
    for (from, to, field) in [
        ("made_by: conclusion", "made_by: chance", "made_by"),
        ("conclusion: inquiry/toolchain#3\n", "", "conclusion"),
        ("made_by: conclusion", "made_by: join", "conclusion"),
        (
            "named_at: 2026-10-25T09:00:00Z",
            "named_at: today",
            "named_at",
        ),
        ("item: pin-mise", "item: pin/mise", "item"),
    ] {
        let error = read(&written.replace(from, to)).unwrap_err();
        assert_eq!(error.field.as_deref(), Some(field), "{to}");
    }
    // An item.rec names the item its folder is.
    let error = InboxAt::read([("inbox/items/other/item.rec", written.as_str())]).unwrap_err();
    assert_eq!(error.field.as_deref(), Some("item"));
}
