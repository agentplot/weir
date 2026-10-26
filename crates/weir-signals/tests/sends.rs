//! Briefs and sends, held to the scenarios of inbox-items: a brief is the
//! text handed, rewritten in place; a send keeps the brief as handed; what
//! became of a send is added beside it (36, 41; core.20, core.21).

mod common;

use weir_signals::{InboxAt, Outcome, SendRecord};

use common::{a_move, after, empty, moved, paths, recfix, recsel_value, with_signals};

const AT: &str = "2026-10-25T09:00:00Z";
const SENT_AT: &str = "2026-10-25T14:00:00+02:00";
const TARGET: &str = "agentplot/weir fix";

const BRIEF: &str =
    "# The shim times out\n\nPin mise to 2026.9.18 until upstream\nreleases the fix.\n";

/// An inbox holding the proposed items `shim-timeout` and `pin-mise`.
fn two_items() -> InboxAt {
    let inbox = with_signals(&empty(), "release/x", 1);
    let inbox = moved(
        &inbox,
        &a_move("release/x#1", "join", "shim-timeout", "curation", AT),
    );
    after(
        &inbox,
        &inbox
            .propose_item("pin-mise", "inquiry/toolchain#3", "operator", AT)
            .unwrap(),
    )
}

/// `inbox` with `text` written as `item`'s brief.
fn briefed(inbox: &InboxAt, item: &str, text: &str) -> InboxAt {
    after(inbox, &inbox.write_brief(item, text).unwrap())
}

/// `two_items()` with `shim-timeout` briefed and sent.
fn sent() -> InboxAt {
    let inbox = briefed(&two_items(), "shim-timeout", BRIEF);
    after(
        &inbox,
        &inbox.send(&["shim-timeout"], TARGET, SENT_AT).unwrap(),
    )
}

#[test]
fn a_brief_written_and_rewritten() {
    let inbox = two_items();
    let write = inbox.write_brief("shim-timeout", BRIEF).unwrap();
    assert_eq!(paths(&write), ["inbox/items/shim-timeout/brief.md"]);
    assert_eq!(
        write.files["inbox/items/shim-timeout/brief.md"],
        format!("<!-- format: weir-brief/1 -->\n{BRIEF}")
    );
    let inbox = after(&inbox, &write);
    assert_eq!(
        inbox.item("shim-timeout").unwrap().brief.as_deref(),
        Some(BRIEF)
    );
    let later = "# The shim times out\n\nUpstream has released the fix.\n";
    let write = inbox.write_brief("shim-timeout", later).unwrap();
    assert_eq!(paths(&write), ["inbox/items/shim-timeout/brief.md"]);
    let next = after(&inbox, &write);
    assert_eq!(
        next.files()["inbox/items/shim-timeout/brief.md"],
        format!("<!-- format: weir-brief/1 -->\n{later}")
    );
    // No other file changes.
    for (path, text) in inbox.files() {
        if path != "inbox/items/shim-timeout/brief.md" {
            assert_eq!(&next.files()[path], text, "{path}");
        }
    }
}

#[test]
fn a_brief_for_no_item() {
    let refused = two_items().write_brief("no-such-item", BRIEF).unwrap_err();
    assert_eq!(refused.0[0].what, "no-such-item");
}

#[test]
fn the_first_send() {
    let inbox = briefed(&two_items(), "shim-timeout", BRIEF);
    assert!(!inbox.item("shim-timeout").unwrap().is_sent());
    let write = inbox.send(&["shim-timeout"], TARGET, SENT_AT).unwrap();
    assert_eq!(paths(&write), ["inbox/items/shim-timeout/sends.rec"]);
    let sends = &write.files["inbox/items/shim-timeout/sends.rec"];
    assert!(
        sends.starts_with(
            "format: weir-send/1\n\
             item: shim-timeout\n\
             target: agentplot/weir fix\n\
             brief: # The shim times out\n"
        ),
        "{sends}"
    );
    recfix(sends);
    assert_eq!(recsel_value(sends, "brief"), BRIEF);
    assert_eq!(recsel_value(sends, "outcome"), "open");
    let inbox = after(&inbox, &write);
    let item = inbox.item("shim-timeout").unwrap();
    assert!(item.is_sent());
    assert_eq!(
        item.sends,
        [SendRecord {
            item: "shim-timeout".into(),
            target: TARGET.into(),
            brief: Some(BRIEF.into()),
            sent_at: SENT_AT.into(),
            outcome: Outcome::Open,
        }]
    );
}

#[test]
fn a_brief_rewritten_after_a_send() {
    let inbox = sent();
    let inbox = briefed(&inbox, "shim-timeout", "an altogether other brief\n");
    let item = inbox.item("shim-timeout").unwrap();
    assert_eq!(item.brief.as_deref(), Some("an altogether other brief\n"));
    assert_eq!(item.sends[0].brief.as_deref(), Some(BRIEF));
}

#[test]
fn several_items_to_one_target() {
    let inbox = briefed(&two_items(), "shim-timeout", BRIEF);
    let inbox = briefed(&inbox, "pin-mise", "Pin mise.\n");
    let write = inbox
        .send(&["shim-timeout", "pin-mise"], "inquiry/toolchain", SENT_AT)
        .unwrap();
    assert_eq!(
        paths(&write),
        [
            "inbox/items/pin-mise/sends.rec",
            "inbox/items/shim-timeout/sends.rec",
        ]
    );
    let inbox = after(&inbox, &write);
    for name in ["shim-timeout", "pin-mise"] {
        let sends = &inbox.item(name).unwrap().sends;
        assert_eq!(sends.len(), 1);
        assert_eq!(sends[0].target, "inquiry/toolchain");
        assert_eq!(sends[0].sent_at, SENT_AT);
    }
    // A second send to another target appends after the first.
    let write = inbox.send(&["pin-mise"], TARGET, AT).unwrap();
    let before = &inbox.files()["inbox/items/pin-mise/sends.rec"];
    assert!(write.files["inbox/items/pin-mise/sends.rec"].starts_with(before.as_str()));
    let inbox = after(&inbox, &write);
    assert_eq!(inbox.item("pin-mise").unwrap().sends.len(), 2);
}

#[test]
fn an_item_with_no_brief() {
    let inbox = briefed(&two_items(), "shim-timeout", BRIEF);
    let refused = inbox
        .send(&["shim-timeout", "pin-mise"], TARGET, SENT_AT)
        .unwrap_err();
    assert_eq!(refused.0.len(), 1);
    assert_eq!(refused.0[0].what, "pin-mise");
    assert!(refused.0[0].why.contains("has no brief"), "{refused}");
    let refused = inbox.send(&["no-such-item"], TARGET, SENT_AT).unwrap_err();
    assert_eq!(refused.0[0].what, "no-such-item");
}

#[test]
fn one_send_to_a_target_at_a_time() {
    let inbox = sent();
    let refused = inbox.send(&["shim-timeout"], TARGET, SENT_AT).unwrap_err();
    assert!(refused.0[0].why.contains("already"), "{refused}");
    let inbox = briefed(&two_items(), "shim-timeout", BRIEF);
    assert!(inbox
        .send(&["shim-timeout", "shim-timeout"], TARGET, SENT_AT)
        .is_err());
    assert!(inbox.send(&[], TARGET, SENT_AT).is_err());
    let refused = inbox.send(&["shim-timeout"], "", SENT_AT).unwrap_err();
    assert_eq!(refused.0[0].field.as_deref(), Some("target"));
    let refused = inbox.send(&["shim-timeout"], TARGET, "now").unwrap_err();
    assert_eq!(refused.0[0].field.as_deref(), Some("sent_at"));
}

#[test]
fn a_send_that_landed() {
    let inbox = sent();
    let before = inbox.files()["inbox/items/shim-timeout/sends.rec"].clone();
    let landing = Outcome::Landing("agentplot/weir 1a2b3c4".into());
    let write = inbox
        .settle_send("shim-timeout", TARGET, SENT_AT, &landing)
        .unwrap();
    let sends = &write.files["inbox/items/shim-timeout/sends.rec"];
    assert!(sends.starts_with(&before), "{sends}");
    assert_eq!(
        &sends[before.len()..],
        "\nformat: weir-send/1\n\
         item: shim-timeout\n\
         target: agentplot/weir fix\n\
         sent_at: 2026-10-25T14:00:00+02:00\n\
         outcome: landing agentplot/weir 1a2b3c4\n"
    );
    recfix(sends);
    let inbox = after(&inbox, &write);
    let item = inbox.item("shim-timeout").unwrap();
    assert_eq!(item.sends.len(), 2);
    assert_eq!(item.sends[0].outcome, Outcome::Open);
    assert_eq!(item.sends[0].brief.as_deref(), Some(BRIEF));
    assert_eq!(item.sends[1].outcome, landing);
    assert_eq!(item.sends[1].brief, None);
}

#[test]
fn an_outcome_for_a_settled_send() {
    let inbox = sent();
    let inbox = after(
        &inbox,
        &inbox
            .settle_send("shim-timeout", TARGET, SENT_AT, &Outcome::Abandoned)
            .unwrap(),
    );
    let refused = inbox
        .settle_send(
            "shim-timeout",
            TARGET,
            SENT_AT,
            &Outcome::Conclusion("inquiry/toolchain#4".into()),
        )
        .unwrap_err();
    assert_eq!(refused.0[0].what, "shim-timeout");
    assert_eq!(refused.0[0].field.as_deref(), Some("outcome"));
    assert_eq!(
        refused.0[0].why,
        "of the send to `agentplot/weir fix` at `2026-10-25T14:00:00+02:00` is `abandoned` already"
    );
}

#[test]
fn an_outcome_for_no_send() {
    let inbox = sent();
    for (item, target, sent_at) in [
        ("shim-timeout", "elsewhere", SENT_AT),
        ("shim-timeout", TARGET, AT),
        ("pin-mise", TARGET, SENT_AT),
        ("no-such-item", TARGET, SENT_AT),
    ] {
        assert!(
            inbox
                .settle_send(item, target, sent_at, &Outcome::Abandoned)
                .is_err(),
            "{item} {target} {sent_at}"
        );
    }
    for outcome in [
        Outcome::Open,
        Outcome::Landing(String::new()),
        Outcome::Conclusion(String::new()),
    ] {
        let refused = inbox
            .settle_send("shim-timeout", TARGET, SENT_AT, &outcome)
            .unwrap_err();
        assert_eq!(refused.0[0].field.as_deref(), Some("outcome"));
    }
}

#[test]
fn every_outcome_reads_back() {
    for outcome in [
        Outcome::Open,
        Outcome::Abandoned,
        Outcome::Landing("agentplot/weir 1a2b3c4".into()),
        Outcome::Conclusion("inquiry/toolchain#4".into()),
    ] {
        assert_eq!(Outcome::parse(&outcome.to_string()), Some(outcome));
    }
    for text in ["", "landing", "landing ", "shipped x", "Open"] {
        assert_eq!(Outcome::parse(text), None, "{text}");
    }
}

#[test]
fn a_tip_whose_sends_break_the_rules_is_refused() {
    let inbox = sent();
    let sends = inbox.files()["inbox/items/shim-timeout/sends.rec"].clone();
    let read = |text: &str| {
        let mut files = inbox.files().clone();
        files.insert("inbox/items/shim-timeout/sends.rec".into(), text.into());
        InboxAt::read(files.iter())
    };
    // A send twice to one target at one time.
    assert!(read(&format!("{sends}\n{sends}")).is_err());
    // An outcome naming no send before it.
    let outcome = "format: weir-send/1\nitem: shim-timeout\ntarget: elsewhere\n\
                   sent_at: 2026-10-25T14:00:00+02:00\noutcome: abandoned\n";
    let error = read(&format!("{sends}\n{outcome}")).unwrap_err();
    assert_eq!(error.field.as_deref(), Some("sent_at"));
    // A send written other than open, or one with no brief.
    let error = read(&sends.replace("outcome: open", "outcome: abandoned")).unwrap_err();
    assert_eq!(error.field.as_deref(), Some("outcome"));
    // Sends of another item, or beside no item.
    let error = read(&sends.replace("item: shim-timeout", "item: pin-mise")).unwrap_err();
    assert_eq!(error.field.as_deref(), Some("item"));
    let error = InboxAt::read([("inbox/items/x/sends.rec", sends.as_str())]).unwrap_err();
    assert!(error.reason.contains("beside no `item.rec`"), "{error}");
}
