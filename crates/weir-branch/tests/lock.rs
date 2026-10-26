//! The lock that keeps one machine's writers in turn, held to the scenarios
//! of shared-branch-lock (44, 49; core.28).

mod common;

use std::cell::Cell;
use std::fs;
use std::path::Path;
use std::sync::Barrier;
use std::thread;
use std::time::{Duration, Instant};

use common::{ask, at, commits_since, left_in, now, parents, seed, write_of, Stage};
use time::format_description::well_known::Rfc3339;
use weir_branch::{write, Branch, Broken, Landed, WriteError, Written, LOCK};

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

// Waiting for a held lock, and breaking a stale one

/// Places a lock held by `holder`, taken `age` before now, in `git_dir`, as
/// a writer would have; gives its text.
fn place_lock(git_dir: &Path, holder: &str, age: Duration) -> String {
    fs::create_dir_all(git_dir).unwrap();
    let taken_at = (now() - age).format(&Rfc3339).unwrap();
    let text = format!("holder: {holder}\ntaken_at: {taken_at}\n");
    fs::write(git_dir.join(LOCK), &text).unwrap();
    text
}

#[test]
fn a_lock_let_go_while_a_writer_waits() {
    let stage = Stage::new();
    let cache = stage.cache("cache");
    place_lock(&cache, "curation@two", Duration::ZERO);
    let started = Instant::now();
    let written = thread::scope(|scope| {
        let lock = cache.join(LOCK);
        scope.spawn(move || {
            thread::sleep(Duration::from_millis(200));
            fs::remove_file(lock).unwrap();
        });
        write(
            &stage.branch("cache"),
            &ask("add a", "operator@one", &now),
            |_| Ok(write_of(&[("inbox/a", "a\n")])),
        )
        .unwrap()
    });
    assert!(
        started.elapsed() >= Duration::from_millis(200),
        "the writer did not wait"
    );
    assert_eq!(written.broke(), None);
    assert_eq!(stage.tip(), Some(landed(written).commit));
    no_lock_left(&cache);
}

#[test]
fn a_stale_lock() {
    let stage = Stage::new();
    let cache = stage.cache("cache");
    let text = place_lock(&cache, "curation@two", Duration::from_secs(120));
    let taken_at = text
        .lines()
        .nth(1)
        .unwrap()
        .strip_prefix("taken_at: ")
        .unwrap();
    let written = write(
        &stage.branch("cache"),
        &ask("add a", "operator@one", &now),
        |_| Ok(write_of(&[("inbox/a", "a\n")])),
    )
    .unwrap();
    let broke = Broken {
        holder: "curation@two".to_string(),
        taken_at: taken_at.to_string(),
    };
    assert_eq!(written.broke(), Some(&broke));
    assert_eq!(stage.tip(), Some(landed(written).commit));
    no_lock_left(&cache);

    // A refused write names the lock it broke too.
    place_lock(&cache, "curation@two", Duration::from_secs(120));
    let refused = write(
        &stage.branch("cache"),
        &ask("x", "operator@one", &now),
        |_| Err("a standing move".to_string()),
    )
    .unwrap_err();
    assert_eq!(
        refused.broke().map(|broke| broke.holder.as_str()),
        Some("curation@two")
    );
    assert!(refused.to_string().contains("curation@two"), "{refused}");
}

#[test]
fn a_stale_lock_is_broken_once_however_many_find_it() {
    let stage = Stage::new();
    let cache = stage.cache("cache");
    place_lock(&cache, "curation@two", Duration::from_secs(120));
    let barrier = Barrier::new(3);
    let written: Vec<Written> = thread::scope(|scope| {
        let writers: Vec<_> = (0..3)
            .map(|n| {
                let (stage, barrier) = (&stage, &barrier);
                scope.spawn(move || {
                    barrier.wait();
                    let who = format!("writer{n}@one");
                    let path = format!("inbox/{n}");
                    write(&stage.branch("cache"), &ask("add", &who, &now), |_| {
                        Ok(write_of(&[(&path, "x\n")]))
                    })
                    .unwrap()
                })
            })
            .collect();
        writers
            .into_iter()
            .map(|writer| writer.join().unwrap())
            .collect()
    });
    let broke = written
        .iter()
        .filter(|written| written.broke().is_some())
        .count();
    assert_eq!(broke, 1, "{written:?}");
    assert!(written
        .iter()
        .all(|written| matches!(written, Written::Landed(_))));
    no_lock_left(&cache);
}

#[test]
fn another_machine_writes_while_the_lock_is_held() {
    let stage = Stage::new();
    let a = seed(
        &stage.remote,
        "inbox",
        &[("inbox/a", "a\n")],
        "start",
        "operator@one",
    );
    let tried = Cell::new(0);
    let b = Cell::new(None);
    let first = landed(
        write(
            &stage.branch("one"),
            &ask("add c", "operator@one", &now),
            |tip| {
                tried.set(tried.get() + 1);
                if tried.get() == 1 {
                    assert_eq!(tip.unwrap().commit(), a);
                    let landed = landed(
                        write(
                            &stage.branch("two"),
                            &ask("add b", "curation@two", &now),
                            |_| {
                                assert!(stage.cache("one").join(LOCK).exists());
                                Ok(write_of(&[("inbox/b", "b\n")]))
                            },
                        )
                        .unwrap(),
                    );
                    b.set(Some(landed.commit));
                }
                Ok(write_of(&[("inbox/c", "c\n")]))
            },
        )
        .unwrap(),
    );
    let b = b.take().unwrap();
    assert_eq!(first.tries, 2);
    assert_eq!(parents(&stage.remote, &first.commit), vec![b]);
}

#[test]
fn a_lock_that_cannot_be_read() {
    let stage = Stage::new();
    let cache = stage.cache("cache");
    fs::create_dir_all(&cache).unwrap();
    fs::write(cache.join(LOCK), "this is no lock\n").unwrap();
    let refused = write(
        &stage.branch("cache"),
        &ask("x", "operator@one", &now),
        |_| panic!("a write was formed"),
    )
    .unwrap_err();
    match &refused {
        WriteError::Failed { step, said, broke } => {
            assert_eq!(*step, weir_branch::Step::Lock);
            assert!(
                said.contains(&cache.join(LOCK).display().to_string()),
                "{said}"
            );
            assert_eq!(*broke, None);
        }
        other => panic!("not failed: {other:?}"),
    }
    assert_eq!(
        fs::read_to_string(cache.join(LOCK)).unwrap(),
        "this is no lock\n"
    );
}
