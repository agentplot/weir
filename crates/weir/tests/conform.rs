//! `weir conform`, run as a host's tests run it: the built binary against
//! the example host, `examples/pack-host.rs`, held to the scenarios of
//! host-conformance (65, 87; C.7; S15).

use std::process::{Command, Output};

fn weir(args: &[&str]) -> Output {
    Command::new(env!("CARGO_BIN_EXE_weir"))
        .args(args)
        .output()
        .expect("the weir binary runs")
}

#[test]
fn no_host_command() {
    for args in [
        &["conform"][..],
        &["conform", "--pack"],
        &["conform", "--pack", "pack"],
    ] {
        let output = weir(args);
        assert_eq!(output.status.code(), Some(2), "{args:?}");
        let stderr = String::from_utf8_lossy(&output.stderr);
        assert!(
            stderr.contains("weir conform [--pack <dir>] <host command> [<arg>...]"),
            "{stderr}"
        );
        assert!(output.stdout.is_empty());
    }
}

#[test]
fn a_pack_that_is_not_there() {
    let output = weir(&["conform", "--pack", "/nonexistent/pack", "true"]);
    assert_eq!(output.status.code(), Some(1));
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stderr.starts_with("weir conform: the pack's"), "{stderr}");
}
