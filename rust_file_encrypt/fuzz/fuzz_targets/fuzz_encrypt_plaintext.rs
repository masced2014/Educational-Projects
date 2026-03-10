#![no_main]
//! Fuzz target: encrypt arbitrary plaintext, then decrypt and verify roundtrip.
//!
//! Goal: prove that `encrypt_file` + `decrypt_file` is a perfect identity
//! function for every possible plaintext, catching any data-loss or
//! corruption bug.

use libfuzzer_sys::fuzz_target;
use secure_file_crypto::crypto::FileCrypto;
use std::io::Write;
use tempfile::NamedTempFile;

fuzz_target!(|data: &[u8]| {
    let mut input_file = NamedTempFile::new().unwrap();
    input_file.write_all(data).unwrap();
    let enc_file = NamedTempFile::new().unwrap();
    let dec_file = NamedTempFile::new().unwrap();

    let password = "fuzz_password";

    if FileCrypto::encrypt_file(
        input_file.path().to_str().unwrap(),
        enc_file.path().to_str().unwrap(),
        password,
    )
    .is_ok()
    {
        let dec_result = FileCrypto::decrypt_file(
            enc_file.path().to_str().unwrap(),
            dec_file.path().to_str().unwrap(),
            password,
        );
        assert!(dec_result.is_ok(), "Decrypt after encrypt must succeed");

        let decrypted = std::fs::read(dec_file.path()).unwrap();
        assert_eq!(decrypted, data, "Decrypted data must match original plaintext");
    }
});
