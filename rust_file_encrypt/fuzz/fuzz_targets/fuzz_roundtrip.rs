#![no_main]
//! Fuzz target: structured (plaintext + password), roundtrip + wrong-password check.
//!
//! Goal: cover the entire Argon2 key-derivation path with arbitrary passwords
//! (including empty, very long, and non-UTF-8 inputs) and verify:
//!   1. Correct password → successful roundtrip
//!   2. Wrong password → decryption error (AEAD authentication failure)

use libfuzzer_sys::{arbitrary::Arbitrary, fuzz_target};
use secure_file_crypto::crypto::FileCrypto;
use std::io::Write;
use tempfile::NamedTempFile;

#[derive(Arbitrary, Debug)]
struct FuzzInput {
    plaintext: Vec<u8>,
    password_bytes: Vec<u8>,
}

fuzz_target!(|input: FuzzInput| {
    // Only test valid UTF-8 passwords (non-UTF-8 is rejected before KDF).
    let password = match std::str::from_utf8(&input.password_bytes) {
        Ok(s) => s,
        Err(_) => return,
    };

    let mut input_file = NamedTempFile::new().unwrap();
    input_file.write_all(&input.plaintext).unwrap();
    let enc_file = NamedTempFile::new().unwrap();
    let dec_file = NamedTempFile::new().unwrap();

    if FileCrypto::encrypt_file(
        input_file.path().to_str().unwrap(),
        enc_file.path().to_str().unwrap(),
        password,
    )
    .is_ok()
    {
        // Correct password: must decrypt successfully and restore original bytes.
        let dec_result = FileCrypto::decrypt_file(
            enc_file.path().to_str().unwrap(),
            dec_file.path().to_str().unwrap(),
            password,
        );
        assert!(dec_result.is_ok(), "Decrypt with correct password must succeed");

        let decrypted = std::fs::read(dec_file.path()).unwrap();
        assert_eq!(decrypted, input.plaintext, "Roundtrip must preserve plaintext");

        // Wrong password: AEAD auth tag must reject the decryption.
        // Guard against the edge case where the fuzz-derived password happens to
        // equal the hardcoded "wrong" password, which would make this a correct
        // password and cause the assertion below to fire spuriously.
        const WRONG_PASSWORD: &str = "definitely_wrong_password_xyz123!@#";
        if password != WRONG_PASSWORD {
            let wrong_dec_file = NamedTempFile::new().unwrap();
            let wrong_result = FileCrypto::decrypt_file(
                enc_file.path().to_str().unwrap(),
                wrong_dec_file.path().to_str().unwrap(),
                WRONG_PASSWORD,
            );
            assert!(wrong_result.is_err(), "Decrypt with wrong password must fail");
        }
    }
});
