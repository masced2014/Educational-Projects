#![no_main]
//! Fuzz target: feed arbitrary bytes to `decrypt_file`.
//!
//! Goal: ensure `decrypt_file` never panics on attacker-controlled input.
//! It exercises the minimum-size guard, auth-tag rejection, and all
//! error-handling branches.  Every run is expected to return an `Err`
//! (the bytes are not a valid encrypted file), but must never panic.

use libfuzzer_sys::fuzz_target;
use secure_file_crypto::crypto::FileCrypto;
use std::io::Write;
use tempfile::NamedTempFile;

fuzz_target!(|data: &[u8]| {
    let mut input_file = NamedTempFile::new().unwrap();
    input_file.write_all(data).unwrap();
    let output_file = NamedTempFile::new().unwrap();

    // Must not panic — errors are expected and acceptable.
    let _ = FileCrypto::decrypt_file(
        input_file.path().to_str().unwrap(),
        output_file.path().to_str().unwrap(),
        "fuzz_password",
    );
});
