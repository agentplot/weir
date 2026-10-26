//! Where each record of an inbox lies (42; C.2): every capture at the one
//! place its key fixes, with its signals beside it, and moves, batches and
//! items under roots no key reaches (4). A path is read into what lies
//! there, or refused (43; core.23).

use weir_capture::Key;

/// The tree an inbox is, at the root of a tip (43; C.2).
pub const INBOX: &str = "inbox";

pub const CAPTURES: &str = "captures";
pub const MOVES: &str = "moves";
pub const ITEMS: &str = "items";

/// The roots under `inbox/`, none of which a key's first segment is (4).
pub const ROOTS: [&str; 3] = [CAPTURES, MOVES, ITEMS];

/// The folder under `moves/` that holds a file per batch.
pub const BATCHES: &str = "batches";

pub const CAPTURE_FILE: &str = "capture.rec";
pub const SIGNALS_FILE: &str = "signals.rec";
pub const MOVES_FILE: &str = "moves.rec";
pub const ITEM_FILE: &str = "item.rec";
pub const BRIEF_FILE: &str = "brief.md";
pub const SENDS_FILE: &str = "sends.rec";

/// Every file name the layout places, none of which a key holds as a
/// segment, since each segment of a key is a folder (4). A batch's file,
/// `<batch>.rec` under `moves/batches/`, lies where no key reaches.
pub const FILES: [&str; 6] = [
    CAPTURE_FILE,
    SIGNALS_FILE,
    MOVES_FILE,
    ITEM_FILE,
    BRIEF_FILE,
    SENDS_FILE,
];

/// What lies at one path of an inbox.
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Place {
    /// `captures/<key>/capture.rec`.
    Capture(Key),
    /// `captures/<key>/signals.rec`.
    Signals(Key),
    /// `moves/moves.rec`.
    Moves,
    /// `moves/batches/<batch>.rec`.
    Batch(String),
    /// `items/<name>/item.rec`.
    Item(String),
    /// `items/<name>/brief.md`.
    Brief(String),
    /// `items/<name>/sends.rec`.
    Sends(String),
}

impl Place {
    /// The path from the tip's root.
    pub fn path(&self) -> String {
        match self {
            Place::Capture(key) => format!("{INBOX}/{CAPTURES}/{key}/{CAPTURE_FILE}"),
            Place::Signals(key) => format!("{INBOX}/{CAPTURES}/{key}/{SIGNALS_FILE}"),
            Place::Moves => format!("{INBOX}/{MOVES}/{MOVES_FILE}"),
            Place::Batch(batch) => format!("{INBOX}/{MOVES}/{BATCHES}/{batch}.rec"),
            Place::Item(name) => format!("{INBOX}/{ITEMS}/{name}/{ITEM_FILE}"),
            Place::Brief(name) => format!("{INBOX}/{ITEMS}/{name}/{BRIEF_FILE}"),
            Place::Sends(name) => format!("{INBOX}/{ITEMS}/{name}/{SENDS_FILE}"),
        }
    }

    /// What lies at `path`: nothing of the inbox for a path outside
    /// `inbox/`, the place for a path the layout places, and why not for
    /// every other path under `inbox/`.
    pub fn of(path: &str) -> Option<Result<Place, String>> {
        let rest = path.strip_prefix(INBOX)?.strip_prefix('/')?;
        Some(place_of(rest))
    }
}

/// The place `rest`, a path under `inbox/`, names.
fn place_of(rest: &str) -> Result<Place, String> {
    let segments: Vec<&str> = rest.split('/').collect();
    match segments.as_slice() {
        [CAPTURES, folders @ .., file] if !folders.is_empty() => {
            let folders = folders.join("/");
            let key = Key::parse(&folders).map_err(|error| {
                format!("its folders under `{CAPTURES}/` are `{folders}`, which is no key: {error}")
            })?;
            match *file {
                CAPTURE_FILE => Ok(Place::Capture(key)),
                SIGNALS_FILE => Ok(Place::Signals(key)),
                _ => Err(unplaced(&format!(
                    "a capture's folder holds `{CAPTURE_FILE}` and `{SIGNALS_FILE}` alone"
                ))),
            }
        }
        [MOVES, MOVES_FILE] => Ok(Place::Moves),
        [MOVES, BATCHES, file] => {
            let batch = file
                .strip_suffix(".rec")
                .ok_or_else(|| unplaced("a batch's file is named `<batch>.rec`"))?;
            check_name(batch).map_err(|reason| format!("its batch, `{batch}`, {reason}"))?;
            Ok(Place::Batch(batch.to_string()))
        }
        [ITEMS, name, file] => {
            check_name(name).map_err(|reason| format!("its item, `{name}`, {reason}"))?;
            let name = name.to_string();
            match *file {
                ITEM_FILE => Ok(Place::Item(name)),
                BRIEF_FILE => Ok(Place::Brief(name)),
                SENDS_FILE => Ok(Place::Sends(name)),
                _ => Err(unplaced(&format!(
                    "an item's folder holds `{ITEM_FILE}`, `{BRIEF_FILE}` and `{SENDS_FILE}` alone"
                ))),
            }
        }
        _ => Err(unplaced(
            "an inbox holds `captures/`, `moves/moves.rec`, `moves/batches/` and `items/` alone",
        )),
    }
}

fn unplaced(rule: &str) -> String {
    format!("lies where the layout places no record: {rule}")
}

/// The rule every item's and every batch's name keeps: one folder or file
/// of its own, so it nests under no other (35). Whether the inbox has had
/// the name is the inbox's to say.
pub fn check_name(name: &str) -> Result<(), String> {
    if name.is_empty() {
        return Err("is empty".to_string());
    }
    if name == "." || name == ".." {
        return Err(format!("is `{name}`, which names no place of its own"));
    }
    if name.contains('/') {
        return Err("holds a `/`, and a name nests under no other".to_string());
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn key(text: &str) -> Key {
        Key::parse(text).unwrap()
    }

    #[test]
    fn the_layouts_names_are_names_no_key_holds() {
        for file in FILES {
            for text in [
                file.to_string(),
                format!("release/{file}"),
                format!("release/{file}/x"),
            ] {
                assert!(Key::parse(&text).is_err(), "{text}");
            }
        }
        for root in ROOTS {
            assert!(Key::parse(root).is_err(), "{root}");
            assert!(Key::parse(&format!("{root}/x")).is_err(), "{root}");
            // A root's name past the first segment is a key's own folder.
            assert!(Key::parse(&format!("deploy/{root}")).is_ok(), "{root}");
        }
    }

    #[test]
    fn each_place_reads_back_from_its_path() {
        for place in [
            Place::Capture(key("release/mise/2026.10.10")),
            Place::Signals(key("release/mise/2026.10.10")),
            Place::Moves,
            Place::Batch("2026-10-25-a".into()),
            Place::Item("shim-timeout".into()),
            Place::Brief("shim-timeout".into()),
            Place::Sends("shim-timeout".into()),
        ] {
            let path = place.path();
            assert_eq!(Place::of(&path), Some(Ok(place)), "{path}");
        }
        assert_eq!(
            Place::Capture(key("release/mise/2026.10.10")).path(),
            "inbox/captures/release/mise/2026.10.10/capture.rec"
        );
        assert_eq!(
            Place::Batch("2026-10-25-a".into()).path(),
            "inbox/moves/batches/2026-10-25-a.rec"
        );
        assert_eq!(
            Place::Item("shim-timeout".into()).path(),
            "inbox/items/shim-timeout/item.rec"
        );
    }

    #[test]
    fn a_path_outside_the_inbox_is_none_of_its() {
        for path in ["README.md", "inbox", "inboxes/moves/moves.rec", ".inbox/x"] {
            assert_eq!(Place::of(path), None, "{path}");
        }
    }

    #[test]
    fn a_path_the_layout_does_not_place_is_refused() {
        for path in [
            "inbox/moves/notes.txt",
            "inbox/moves/moves.rec/x",
            "inbox/notes.md",
            "inbox/captures/capture.rec",
            "inbox/captures/release/x/raw.md",
            "inbox/captures/moves/x/capture.rec",
            "inbox/captures/release//x/capture.rec",
            "inbox/moves/batches/a.txt",
            "inbox/moves/batches/.rec",
            "inbox/moves/batches/a/b.rec",
            "inbox/items/x/notes.md",
            "inbox/items/x/y/item.rec",
            "inbox/items/../item.rec",
            "inbox/items/item.rec",
            "inbox/",
        ] {
            assert!(matches!(Place::of(path), Some(Err(_))), "{path}");
        }
        let Some(Err(reason)) = Place::of("inbox/captures/moves/x/capture.rec") else {
            panic!("refused");
        };
        assert!(reason.contains("`moves/x`, which is no key"), "{reason}");
    }

    #[test]
    fn a_name_nests_under_no_other() {
        assert_eq!(check_name("shim-timeout"), Ok(()));
        for name in ["", ".", "..", "shim/timeout", "/x"] {
            assert!(check_name(name).is_err(), "{name:?}");
        }
    }
}
