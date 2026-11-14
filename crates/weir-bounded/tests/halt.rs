//! A caller's halt: one flag its clones share, and a command ended at the
//! first of its limit and its halt (record 0008 as amended 2026-11-12;
//! core.1, core.53). Each command is `sh`, and reaches no network.

use weir_bounded::Halt;

#[test]
fn a_halt_is_one_flag_its_clones_share() {
    let first = Halt::default();
    let clone = first.clone();
    assert!(!first.halted(), "a halt made is set");
    clone.halt();
    clone.halt();
    let second = Halt::default();
    assert!(first.halted(), "the clone's halt did not reach the first");
    assert!(!second.halted(), "a second halt is set");
    assert_eq!(first, clone);
    assert_ne!(first, second);
}
