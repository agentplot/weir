//! The content hash the built-in adapters key files by: sha256, as 64
//! lowercase hexadecimal digits, the same for the same bytes whatever the
//! file's name (61, 62; C.6; core.34).

use std::fmt::Write as _;
use std::io::{self, Read};

use sha2::{Digest, Sha256};

/// The sha256 of `bytes`.
pub fn content_hash(bytes: &[u8]) -> String {
    hex(&Sha256::digest(bytes))
}

/// The sha256 of everything `reader` reads, read in pieces, so a file is
/// never held whole.
pub fn content_hash_reader(mut reader: impl Read) -> io::Result<String> {
    let mut hasher = Sha256::new();
    let mut buffer = [0u8; 64 * 1024];
    loop {
        match reader.read(&mut buffer) {
            Ok(0) => break,
            Ok(n) => hasher.update(&buffer[..n]),
            Err(error) if error.kind() == io::ErrorKind::Interrupted => {}
            Err(error) => return Err(error),
        }
    }
    Ok(hex(&hasher.finalize()))
}

fn hex(digest: &[u8]) -> String {
    let mut text = String::with_capacity(digest.len() * 2);
    for byte in digest {
        write!(text, "{byte:02x}").expect("writing to a string never fails");
    }
    text
}

#[cfg(test)]
mod tests {
    use std::fs::File;
    use std::io::Write as _;

    use super::*;

    #[test]
    fn known_bytes_hash_as_sha256() {
        assert_eq!(
            content_hash(b"abc"),
            "ba7816bf8f01cfea414140de5dae2223b00361a396177a9cb410ff61f20015ad"
        );
        assert_eq!(
            content_hash(b""),
            "e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855"
        );
        assert_eq!(
            content_hash_reader(&b"abc"[..]).unwrap(),
            content_hash(b"abc")
        );
    }

    #[test]
    fn the_two_agree_on_a_file_of_a_mebibyte() {
        let bytes: Vec<u8> = (0..1024 * 1024).map(|n: u32| (n % 251) as u8).collect();
        let mut file = tempfile::NamedTempFile::new().unwrap();
        file.write_all(&bytes).unwrap();
        file.flush().unwrap();
        let from_file = content_hash_reader(File::open(file.path()).unwrap()).unwrap();
        assert_eq!(from_file, content_hash(&bytes));
        assert_eq!(from_file.len(), 64);
        assert!(from_file
            .chars()
            .all(|c| c.is_ascii_digit() || ('a'..='f').contains(&c)));
    }
}
