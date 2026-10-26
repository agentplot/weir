//! A file copied into the raw place as its raw material, hashed as it is
//! copied, so the bytes banked are the bytes keyed (6, 61, 62; core.34).
//! The copy is written under a temporary name in the raw place and renamed
//! to `<sha256>/<file name>` once its hash is known.

use std::fs::{self, File};
use std::io::{self, Read, Write};
use std::path::{Path, PathBuf};

use weir_capture::content_hash_reader;

/// Where a copy is written while its hash is not yet known: a name no
/// `<sha256>` folder has.
const COPYING: &str = ".copying";

/// A file copied whole into the raw place, not yet under its hash.
pub(crate) struct Copied {
    pub hash: String,
    temp: PathBuf,
}

/// Reads through `reader`, writing every byte it reads to `writer`.
struct Tee<R, W> {
    reader: R,
    writer: W,
}

impl<R: Read, W: Write> Read for Tee<R, W> {
    fn read(&mut self, buffer: &mut [u8]) -> io::Result<usize> {
        let n = self.reader.read(buffer)?;
        self.writer.write_all(&buffer[..n])?;
        Ok(n)
    }
}

/// Copies the file at `path` into `raw_dir` under the temporary name,
/// hashing the bytes copied. Nothing is left in the raw place when it fails.
pub(crate) fn copy(path: &Path, raw_dir: &Path) -> io::Result<Copied> {
    let source = File::open(path)?;
    let temp = raw_dir.join(COPYING);
    let copied = File::create(&temp).and_then(|writer| {
        let mut tee = Tee {
            reader: source,
            writer,
        };
        let hash = content_hash_reader(&mut tee)?;
        tee.writer.flush()?;
        Ok(hash)
    });
    match copied {
        Ok(hash) => Ok(Copied { hash, temp }),
        Err(error) => {
            let _ = fs::remove_file(&temp);
            Err(error)
        }
    }
}

impl Copied {
    /// Moves the copy to `<raw_dir>/<sha256>/<name>`, and returns that path
    /// relative to the raw place, as an offer's `raw` names it.
    pub(crate) fn bank(self, raw_dir: &Path, name: &str) -> io::Result<String> {
        let folder = raw_dir.join(&self.hash);
        let banked =
            fs::create_dir_all(&folder).and_then(|()| fs::rename(&self.temp, folder.join(name)));
        if let Err(error) = banked {
            let _ = fs::remove_file(&self.temp);
            return Err(error);
        }
        Ok(format!("{}/{name}", self.hash))
    }

    /// Removes the copy, whose content was offered already.
    pub(crate) fn discard(self) {
        let _ = fs::remove_file(&self.temp);
    }
}
