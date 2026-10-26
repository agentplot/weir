//! The fixtures every test reads: declarations under `tests/fixtures/`.

use weir_kinds::{read_kinds, Declaration, Refusal};

/// The fixture `name` read as a declaration.
pub fn read(name: &str) -> Result<Declaration, Refusal> {
    let path = format!("{}/tests/fixtures/{name}", env!("CARGO_MANIFEST_DIR"));
    let text = std::fs::read_to_string(&path).unwrap_or_else(|e| panic!("{path}: {e}"));
    read_kinds(&text)
}
