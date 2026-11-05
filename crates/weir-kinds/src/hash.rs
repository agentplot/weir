//! A package's content hash: `sha256:` and the lowercase hex sha256 of its
//! regular files, each with its path within the package, computed through
//! weir's one sha256 (90, C.4; core.44).

use std::fmt;
use std::io::{self, Read};
use std::str::FromStr;

/// What a content hash is, as every reason that refuses one says it.
const CONTENT_HASH: &str = "a content hash is `sha256:` and 64 lowercase hex digits";

/// The prefix every content hash is spelled with.
const PREFIX: &str = "sha256:";

/// The byte that ends a path and a length in what is hashed.
const NUL: &[u8] = b"\0";

/// A package's content hash, spelled `sha256:` and 64 lowercase hex digits
/// (90, C.4). It is computed by [`ContentHash::of`] alone, or read back from
/// its spelling, as a lock records it (91).
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct ContentHash {
    /// The whole spelling, `sha256:` among it.
    text: String,
}

impl ContentHash {
    /// The content hash of a package's files, each a path within the
    /// package and its bytes: the sha256 of the concatenation, in byte
    /// order of the paths, of each path, a NUL byte, its length in bytes
    /// in decimal, a NUL byte and its bytes (90, C.4; core.44). Nothing but
    /// the paths and the bytes reaches it, so where a package lies and when
    /// its files were written never do.
    ///
    /// A package holds one file at each path, which `read_package` holds
    /// it to; the files are hashed in that order whatever order they are
    /// given in.
    pub fn of<P: AsRef<[u8]>, B: AsRef<[u8]>>(files: impl IntoIterator<Item = (P, B)>) -> Self {
        let mut files: Vec<(P, B)> = files.into_iter().collect();
        files.sort_by(|(a, a_bytes), (b, b_bytes)| {
            (a.as_ref(), a_bytes.as_ref()).cmp(&(b.as_ref(), b_bytes.as_ref()))
        });
        let lengths: Vec<String> = files
            .iter()
            .map(|(_, bytes)| bytes.as_ref().len().to_string())
            .collect();
        let pieces = files
            .iter()
            .zip(&lengths)
            .flat_map(|((path, bytes), length)| {
                [path.as_ref(), NUL, length.as_bytes(), NUL, bytes.as_ref()]
            });
        let hex = weir_capture::content_hash_reader(Pieces {
            pieces,
            current: &[],
        })
        .expect("reading bytes held in memory never fails");
        ContentHash {
            text: format!("{PREFIX}{hex}"),
        }
    }

    /// Its 64 lowercase hex digits, without `sha256:`.
    pub fn hex(&self) -> &str {
        &self.text[PREFIX.len()..]
    }
}

impl FromStr for ContentHash {
    type Err = HashRefusal;

    /// A content hash from its spelling, or refused saying why it is none.
    fn from_str(text: &str) -> Result<Self, Self::Err> {
        let fault = match text.strip_prefix(PREFIX) {
            None => "does not begin with `sha256:`".to_owned(),
            Some(hex) => match hex
                .chars()
                .find(|c| !(c.is_ascii_digit() || ('a'..='f').contains(c)))
            {
                Some(c) => format!("holds {c:?}"),
                None if hex.len() != 64 => format!("has {} hex digits", hex.len()),
                None => {
                    return Ok(ContentHash {
                        text: text.to_owned(),
                    })
                }
            },
        };
        Err(HashRefusal {
            text: text.to_owned(),
            reason: format!("is no content hash: it {fault}, and {CONTENT_HASH}"),
        })
    }
}

impl fmt::Display for ContentHash {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.text)
    }
}

/// Text refused as a content hash, with why (90, C.4). Its
/// [`Display`](fmt::Display) gives the text and the reason.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct HashRefusal {
    /// The text refused, as given.
    pub text: String,
    /// Why it is refused, saying what a content hash is.
    pub reason: String,
}

impl fmt::Display for HashRefusal {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if self.text.is_empty() {
            write!(f, "the text {}", self.reason)
        } else {
            write!(f, "`{}` {}", self.text, self.reason)
        }
    }
}

impl std::error::Error for HashRefusal {}

/// The concatenation C.4 hashes, read one piece after another, so that no
/// file's bytes are copied to be hashed.
struct Pieces<'a, I> {
    pieces: I,
    current: &'a [u8],
}

impl<'a, I: Iterator<Item = &'a [u8]>> Read for Pieces<'a, I> {
    fn read(&mut self, buffer: &mut [u8]) -> io::Result<usize> {
        while self.current.is_empty() {
            match self.pieces.next() {
                Some(piece) => self.current = piece,
                None => return Ok(0),
            }
        }
        self.current.read(buffer)
    }
}
