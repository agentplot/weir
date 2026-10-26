//! The lock that keeps one machine's writers in turn, held to the scenarios
//! of shared-branch-lock (44, 49; core.28).

mod common;

use std::fs;
use std::path::Path;
use std::sync::Barrier;
use std::thread;
use std::time::Duration;

use common::{ask, at, commits_since, left_in, now, seed, write_of, Stage};
use weir_branch::{write, Branch, Landed, WriteError, Written, LOCK};

fn landed(written: Written) -> Landed {
    match written {
        Written::Landed(landed) => landed,
        other => panic!("not landed: {other:?}"),
    }
}

/// No lock stands in `git_dir`, and none of the files a writer takes or
/// breaks one through is left.
fn no_lock_left(git_dir: &Path) {
    assert!(!git_dir.join(LOCK).exists(), "a lock stands");
    assert!(
        left_in(git_dir, "weir.lock").is_empty(),
        "{:?}",
        left_in(git_dir, "weir.lock")
    );
}

// Taken from the read to the landing, and let go

#[test]
fn two_writers_on_one_machine() {
    let stage = Stage::new();
    let a = seed(
        &stage.remote,
        "inbox",
        &[("inbox/a", "a\n")],
        "start",
        "operator@one",
    );
    let barrier = Barrier::new(2);
    let landed: Vec<Landed> = thread::scope(|scope| {
        let writers: Vec<_> = ["operator@one", "curation@one"]
            .into_iter()
            .map(|who| {
                let (stage, barrier) = (&stage, &barrier);
                scope.spawn(move || {
                    barrier.wait();
                    let path = format!("inbox/{who}");
                    landed(
                        write(
                            &stage.branch("cache"),
                            &ask("add a file", who, &now),
                            |_| {
                                let lock =
                                    fs::read_to_string(stage.cache("cache").join(LOCK)).unwrap();
                                assert!(lock.starts_with(&format!("holder: {who}\n")), "{lock}");
                                thread::sleep(Duration::from_millis(150));
                                Ok(write_of(&[(&path, "x\n")]))
                            },
                        )
                        .unwrap(),
                    )
                })
            })
            .collect();
        writers
            .into_iter()
            .map(|writer| writer.join().unwrap())
            .collect()
    });
    assert!(
        landed.iter().all(|landed| landed.tries == 1),
        "a push was refused: {landed:?}"
    );
    let (first, second) = if landed[0].on.as_deref() == Some(a.as_str()) {
        (&landed[0], &landed[1])
    } else {
        (&landed[1], &landed[0])
    };
    assert_eq!(first.on.as_deref(), Some(a.as_str()));
    assert_eq!(second.on.as_deref(), Some(first.commit.as_str()));
    assert_eq!(
        commits_since(&stage.remote, Some(&a), &second.commit),
        vec![first.commit.clone(), second.commit.clone()]
    );
    no_lock_left(&stage.cache("cache"));
}

#[test]
fn many_writers_on_one_machine_each_land_on_the_last() {
    let stage = Stage::new();
    let a = seed(
        &stage.remote,
        "inbox",
        &[("inbox/a", "a\n")],
        "start",
        "operator@one",
    );
    let barrier = Barrier::new(4);
    let tries: Vec<u32> = thread::scope(|scope| {
        let writers: Vec<_> = (0..4)
            .map(|n| {
                let (stage, barrier) = (&stage, &barrier);
                scope.spawn(move || {
                    barrier.wait();
                    (0..3)
                        .map(|m| {
                            let who = format!("writer{n}@one");
                            let path = format!("inbox/{n}/{m}");
                            landed(
                                write(&stage.branch("cache"), &ask("add", &who, &now), |_| {
                                    Ok(write_of(&[(&path, "x\n")]))
                                })
                                .unwrap(),
                            )
                            .tries
                        })
                        .collect::<Vec<_>>()
                })
            })
            .collect();
        writers
            .into_iter()
            .flat_map(|writer| writer.join().unwrap())
            .collect()
    });
    assert!(
        tries.iter().all(|&tries| tries == 1),
        "a push was refused: {tries:?}"
    );
    let tip = stage.tip().unwrap();
    assert_eq!(commits_since(&stage.remote, Some(&a), &tip).len(), 12);
    assert_eq!(common::files(&stage.remote, &tip).len(), 13);
    no_lock_left(&stage.cache("cache"));
}

#[test]
fn the_lock_is_let_go_after_a_refusal() {
    let stage = Stage::new();
    seed(
        &stage.remote,
        "inbox",
        &[("inbox/a", "a\n")],
        "start",
        "operator@one",
    );
    let cache = stage.cache("cache");
    let refused = write(
        &stage.branch("cache"),
        &ask("x", "operator@one", &now),
        |_| {
            assert!(
                cache.join(LOCK).exists(),
                "no lock held while the write is formed"
            );
            Err("a standing move".to_string())
        },
    )
    .unwrap_err();
    assert!(matches!(refused, WriteError::Refused { .. }), "{refused:?}");
    no_lock_left(&cache);
}

#[test]
fn the_lock_is_let_go_on_every_end() {
    let stage = Stage::new();
    seed(
        &stage.remote,
        "inbox",
        &[("inbox/a", "a\n")],
        "start",
        "operator@one",
    );
    let branch = stage.branch("cache");
    let cache = stage.cache("cache");
    let asked = ask("x", "operator@one", &now);

    write(&branch, &asked, |_| Ok(write_of(&[("inbox/b", "b\n")]))).unwrap();
    no_lock_left(&cache);
    write(&branch, &asked, |_| Ok(weir_branch::Files::new())).unwrap();
    no_lock_left(&cache);
    write(&branch, &asked, |_| {
        Ok(write_of(&[("inbox/c.rec", "no field\n")]))
    })
    .unwrap_err();
    no_lock_left(&cache);
    let panicked = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
        let _ = write(&branch, &asked, |_| panic!("the caller fails"));
    }));
    assert!(panicked.is_err());
    no_lock_left(&cache);

    let repo = common::bare(stage.folder.path(), "local.git");
    write(&Branch::local("inbox", &repo), &asked, |_| {
        assert!(
            repo.join(LOCK).exists(),
            "no lock in the repository's git directory"
        );
        Ok(write_of(&[("a", "a\n")]))
    })
    .unwrap();
    no_lock_left(&repo);
}

#[test]
fn reading_a_held_lock() {
    let stage = Stage::new();
    let clock = || at("2026-10-17T06:30:05+02:00");
    let cache = stage.cache("cache");
    write(
        &stage.branch("cache"),
        &ask("take release/acme/v1", "operator@deneb", &clock),
        |_| {
            assert_eq!(
                fs::read_to_string(cache.join(LOCK)).unwrap(),
                "holder: operator@deneb\ntaken_at: 2026-10-17T06:30:05+02:00\n"
            );
            Ok(write_of(&[("inbox/a", "a\n")]))
        },
    )
    .unwrap();
    no_lock_left(&cache);
}
