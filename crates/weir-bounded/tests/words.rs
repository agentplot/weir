//! The family's limit and grace, and a stopped call's words (C.3; record
//! 0008; core.53).

use std::time::Duration;

use weir_bounded::{span, stopped, GRACE, TRY_FOR};

#[test]
fn a_stopped_calls_words_name_its_call_remote_and_limit() {
    assert_eq!(
        stopped("git fetch", "origin", TRY_FOR),
        "git fetch with origin was still going after 30s, the most a try may take, and was stopped"
    );
}

#[test]
fn a_limit_is_said_in_seconds_or_milliseconds() {
    assert_eq!(span(Duration::from_secs(30)), "30s");
    assert_eq!(span(Duration::from_millis(300)), "300ms");
}

#[test]
fn the_limit_and_grace_are_the_familys() {
    assert_eq!(TRY_FOR, Duration::from_secs(30));
    assert_eq!(GRACE, Duration::from_secs(2));
}
