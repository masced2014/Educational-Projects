//! Robustness and edge case tests for the file crypto system

use std::fs;
use std::io::Write;
use std::process::Command;
use tempfile::TempDir;

/// Helper to get the path to the compiled binary
fn get_binary_path() -> String {
    // Use the path provided by Cargo for the compiled binary during tests.
    // This avoids relying on target directory layout or manual string concatenation.
    env!("CARGO_BIN_EXE_secure-file-crypto").to_string()
}

#[test]
fn test_corrupted_salt() {
    let test_dir = TempDir::new().unwrap();
    let test_path = test_dir.path();

    let input_file = test_path.join("input.txt");
    let encrypted_file = test_path.join("encrypted.bin");
    let decrypted_file = test_path.join("decrypted.txt");

    fs::write(&input_file, b"Test data").unwrap();

    let binary = get_binary_path();
    let password = "test_password";

    // Encrypt
    let encrypt_output = Command::new(&binary)
        .arg("encrypt")
        .arg("-i")
        .arg(input_file.to_str().unwrap())
        .arg("-o")
        .arg(encrypted_file.to_str().unwrap())
        .arg("-p")
        .arg(password)
        .output()
        .unwrap();
    
    assert!(encrypt_output.status.success(), 
        "Encrypt failed: {}", 
        String::from_utf8_lossy(&encrypt_output.stderr));

    // Corrupt the salt (first 16 bytes)
    let mut data = fs::read(&encrypted_file).unwrap();
    if data.len() > 8 {
        data[8] ^= 0xFF;
    }
    fs::write(&encrypted_file, &data).unwrap();

    // Try to decrypt - should fail due to wrong derived key
    let output = Command::new(&binary)
        .arg("decrypt")
        .arg("-i")
        .arg(encrypted_file.to_str().unwrap())
        .arg("-o")
        .arg(decrypted_file.to_str().unwrap())
        .arg("-p")
        .arg(password)
        .output()
        .unwrap();

    assert!(!output.status.success());
}

#[test]
fn test_corrupted_nonce() {
    let test_dir = TempDir::new().unwrap();
    let test_path = test_dir.path();

    let input_file = test_path.join("input.txt");
    let encrypted_file = test_path.join("encrypted.bin");
    let decrypted_file = test_path.join("decrypted.txt");

    fs::write(&input_file, b"Test data").unwrap();

    let binary = get_binary_path();
    let password = "test_password";

    // Encrypt
    let encrypt_output = Command::new(&binary)
        .arg("encrypt")
        .arg("-i")
        .arg(input_file.to_str().unwrap())
        .arg("-o")
        .arg(encrypted_file.to_str().unwrap())
        .arg("-p")
        .arg(password)
        .output()
        .unwrap();
    
    assert!(encrypt_output.status.success(), 
        "Encrypt failed: {}", 
        String::from_utf8_lossy(&encrypt_output.stderr));

    // Corrupt the nonce (bytes 16-27)
    let mut data = fs::read(&encrypted_file).unwrap();
    if data.len() > 20 {
        data[20] ^= 0xFF;
    }
    fs::write(&encrypted_file, &data).unwrap();

    // Try to decrypt - should fail
    let output = Command::new(&binary)
        .arg("decrypt")
        .arg("-i")
        .arg(encrypted_file.to_str().unwrap())
        .arg("-o")
        .arg(decrypted_file.to_str().unwrap())
        .arg("-p")
        .arg(password)
        .output()
        .unwrap();

    assert!(!output.status.success());
}

#[test]
fn test_truncated_file() {
    let test_dir = TempDir::new().unwrap();
    let test_path = test_dir.path();

    let input_file = test_path.join("input.txt");
    let encrypted_file = test_path.join("encrypted.bin");
    let decrypted_file = test_path.join("decrypted.txt");

    fs::write(&input_file, b"Test data for truncation").unwrap();

    let binary = get_binary_path();
    let password = "test_password";

    // Encrypt
    let encrypt_output = Command::new(&binary)
        .arg("encrypt")
        .arg("-i")
        .arg(input_file.to_str().unwrap())
        .arg("-o")
        .arg(encrypted_file.to_str().unwrap())
        .arg("-p")
        .arg(password)
        .output()
        .unwrap();
    
    assert!(encrypt_output.status.success(), 
        "Encrypt failed: {}", 
        String::from_utf8_lossy(&encrypt_output.stderr));

    // Truncate the file
    let mut data = fs::read(&encrypted_file).unwrap();
    data.truncate(data.len() - 5);
    fs::write(&encrypted_file, &data).unwrap();

    // Try to decrypt - should fail
    let output = Command::new(&binary)
        .arg("decrypt")
        .arg("-i")
        .arg(encrypted_file.to_str().unwrap())
        .arg("-o")
        .arg(decrypted_file.to_str().unwrap())
        .arg("-p")
        .arg(password)
        .output()
        .unwrap();

    assert!(!output.status.success());
}

#[test]
fn test_extended_file() {
    let test_dir = TempDir::new().unwrap();
    let test_path = test_dir.path();

    let input_file = test_path.join("input.txt");
    let encrypted_file = test_path.join("encrypted.bin");
    let decrypted_file = test_path.join("decrypted.txt");

    fs::write(&input_file, b"Test data").unwrap();

    let binary = get_binary_path();
    let password = "test_password";

    // Encrypt
    let encrypt_output = Command::new(&binary)
        .arg("encrypt")
        .arg("-i")
        .arg(input_file.to_str().unwrap())
        .arg("-o")
        .arg(encrypted_file.to_str().unwrap())
        .arg("-p")
        .arg(password)
        .output()
        .unwrap();
    
    assert!(encrypt_output.status.success(), 
        "Encrypt failed: {}", 
        String::from_utf8_lossy(&encrypt_output.stderr));

    // Append extra bytes to the file
    let mut file = fs::OpenOptions::new()
        .append(true)
        .open(&encrypted_file)
        .unwrap();
    file.write_all(b"EXTRA_DATA").unwrap();
    drop(file);

    // Try to decrypt - should fail due to authentication
    let output = Command::new(&binary)
        .arg("decrypt")
        .arg("-i")
        .arg(encrypted_file.to_str().unwrap())
        .arg("-o")
        .arg(decrypted_file.to_str().unwrap())
        .arg("-p")
        .arg(password)
        .output()
        .unwrap();

    assert!(!output.status.success());
}

#[test]
fn test_very_large_file() {
    let test_dir = TempDir::new().unwrap();
    let test_path = test_dir.path();

    let input_file = test_path.join("large.bin");
    let encrypted_file = test_path.join("large.enc");
    let decrypted_file = test_path.join("large.dec");

    // Create a 5MB file
    let size = 5 * 1024 * 1024;
    let data: Vec<u8> = (0..size).map(|i| (i % 256) as u8).collect();
    fs::write(&input_file, &data).unwrap();

    let binary = get_binary_path();
    let password = "large_file_password";

    // Encrypt
    let encrypt_output = Command::new(&binary)
        .arg("encrypt")
        .arg("-i")
        .arg(input_file.to_str().unwrap())
        .arg("-o")
        .arg(encrypted_file.to_str().unwrap())
        .arg("-p")
        .arg(password)
        .output()
        .unwrap();

    assert!(encrypt_output.status.success());

    // Decrypt
    let decrypt_output = Command::new(&binary)
        .arg("decrypt")
        .arg("-i")
        .arg(encrypted_file.to_str().unwrap())
        .arg("-o")
        .arg(decrypted_file.to_str().unwrap())
        .arg("-p")
        .arg(password)
        .output()
        .unwrap();

    assert!(decrypt_output.status.success());

    // Verify size
    let decrypted_size = fs::metadata(&decrypted_file).unwrap().len();
    assert_eq!(decrypted_size, size as u64);
}

#[test]
fn test_single_byte_file() {
    let test_dir = TempDir::new().unwrap();
    let test_path = test_dir.path();

    let input_file = test_path.join("single.txt");
    let encrypted_file = test_path.join("single.enc");
    let decrypted_file = test_path.join("single.dec");

    fs::write(&input_file, b"A").unwrap();

    let binary = get_binary_path();
    let password = "test";

    // Encrypt
    let encrypt_output = Command::new(&binary)
        .arg("encrypt")
        .arg("-i")
        .arg(input_file.to_str().unwrap())
        .arg("-o")
        .arg(encrypted_file.to_str().unwrap())
        .arg("-p")
        .arg(password)
        .output()
        .unwrap();

    assert!(encrypt_output.status.success());

    // Decrypt
    let decrypt_output = Command::new(&binary)
        .arg("decrypt")
        .arg("-i")
        .arg(encrypted_file.to_str().unwrap())
        .arg("-o")
        .arg(decrypted_file.to_str().unwrap())
        .arg("-p")
        .arg(password)
        .output()
        .unwrap();

    assert!(decrypt_output.status.success());

    let decrypted = fs::read(&decrypted_file).unwrap();
    assert_eq!(decrypted, b"A");
}

#[test]
fn test_null_bytes_in_file() {
    let test_dir = TempDir::new().unwrap();
    let test_path = test_dir.path();

    let input_file = test_path.join("nulls.bin");
    let encrypted_file = test_path.join("nulls.enc");
    let decrypted_file = test_path.join("nulls.dec");

    let data = vec![0u8; 1000];
    fs::write(&input_file, &data).unwrap();

    let binary = get_binary_path();
    let password = "null_test";

    // Encrypt
    Command::new(&binary)
        .arg("encrypt")
        .arg("-i")
        .arg(input_file.to_str().unwrap())
        .arg("-o")
        .arg(encrypted_file.to_str().unwrap())
        .arg("-p")
        .arg(password)
        .output()
        .unwrap();

    // Decrypt
    Command::new(&binary)
        .arg("decrypt")
        .arg("-i")
        .arg(encrypted_file.to_str().unwrap())
        .arg("-o")
        .arg(decrypted_file.to_str().unwrap())
        .arg("-p")
        .arg(password)
        .output()
        .unwrap();

    let decrypted = fs::read(&decrypted_file).unwrap();
    assert_eq!(decrypted, data);
}

#[test]
fn test_repeating_pattern() {
    let test_dir = TempDir::new().unwrap();
    let test_path = test_dir.path();

    let input_file = test_path.join("pattern.bin");
    let encrypted_file = test_path.join("pattern.enc");
    let decrypted_file = test_path.join("pattern.dec");

    // Create file with repeating pattern
    let pattern = b"AAAAAAAA";
    let mut data = Vec::new();
    for _ in 0..1000 {
        data.extend_from_slice(pattern);
    }
    fs::write(&input_file, &data).unwrap();

    let binary = get_binary_path();
    let password = "pattern_test";

    // Encrypt
    let encrypt_output = Command::new(&binary)
        .arg("encrypt")
        .arg("-i")
        .arg(input_file.to_str().unwrap())
        .arg("-o")
        .arg(encrypted_file.to_str().unwrap())
        .arg("-p")
        .arg(password)
        .output()
        .unwrap();
    
    assert!(encrypt_output.status.success(), 
        "Encrypt failed: {}", 
        String::from_utf8_lossy(&encrypt_output.stderr));

    // Verify encrypted data doesn't contain obvious patterns
    let encrypted_data = fs::read(&encrypted_file).unwrap();
    // Skip salt and nonce (first 28 bytes), check ciphertext
    if encrypted_data.len() > 28 {
        let ciphertext = &encrypted_data[28..];
        // Check that ciphertext is not identical to the corresponding plaintext bytes
        let plaintext_portion_len = ciphertext.len().saturating_sub(16); // Exclude auth tag
        if plaintext_portion_len > 0 && plaintext_portion_len <= data.len() {
            assert_ne!(&ciphertext[..plaintext_portion_len], &data[..plaintext_portion_len]);
        }
    }

    // Decrypt and verify
    let decrypt_output = Command::new(&binary)
        .arg("decrypt")
        .arg("-i")
        .arg(encrypted_file.to_str().unwrap())
        .arg("-o")
        .arg(decrypted_file.to_str().unwrap())
        .arg("-p")
        .arg(password)
        .output()
        .unwrap();

    assert!(decrypt_output.status.success(), 
        "Decrypt failed: {}", 
        String::from_utf8_lossy(&decrypt_output.stderr));

    let decrypted = fs::read(&decrypted_file).unwrap();
    assert_eq!(decrypted, data);
}

#[test]
fn test_unicode_filename() {
    let test_dir = TempDir::new().unwrap();
    let test_path = test_dir.path();

    let input_file = test_path.join("файл_测试_🔐.txt");
    let encrypted_file = test_path.join("файл_测试_🔐.enc");
    let decrypted_file = test_path.join("файл_测试_🔐.dec");

    fs::write(&input_file, b"Unicode filename test").unwrap();

    let binary = get_binary_path();
    let password = "unicode_test";

    // Encrypt
    let encrypt_output = Command::new(&binary)
        .arg("encrypt")
        .arg("-i")
        .arg(input_file.to_str().unwrap())
        .arg("-o")
        .arg(encrypted_file.to_str().unwrap())
        .arg("-p")
        .arg(password)
        .output()
        .unwrap();

    assert!(encrypt_output.status.success());

    // Decrypt
    let decrypt_output = Command::new(&binary)
        .arg("decrypt")
        .arg("-i")
        .arg(encrypted_file.to_str().unwrap())
        .arg("-o")
        .arg(decrypted_file.to_str().unwrap())
        .arg("-p")
        .arg(password)
        .output()
        .unwrap();

    assert!(decrypt_output.status.success());

    let decrypted = fs::read(&decrypted_file).unwrap();
    assert_eq!(decrypted, b"Unicode filename test");
}

#[test]
fn test_very_long_password() {
    let test_dir = TempDir::new().unwrap();
    let test_path = test_dir.path();

    let input_file = test_path.join("input.txt");
    let encrypted_file = test_path.join("encrypted.bin");
    let decrypted_file = test_path.join("decrypted.txt");

    fs::write(&input_file, b"Test data").unwrap();

    let binary = get_binary_path();
    // 1000 character password
    let password: String = (0..1000).map(|i| ((i % 26) as u8 + b'a') as char).collect();

    // Encrypt
    let encrypt_output = Command::new(&binary)
        .arg("encrypt")
        .arg("-i")
        .arg(input_file.to_str().unwrap())
        .arg("-o")
        .arg(encrypted_file.to_str().unwrap())
        .arg("-p")
        .arg(&password)
        .output()
        .unwrap();

    assert!(encrypt_output.status.success());

    // Decrypt
    let decrypt_output = Command::new(&binary)
        .arg("decrypt")
        .arg("-i")
        .arg(encrypted_file.to_str().unwrap())
        .arg("-o")
        .arg(decrypted_file.to_str().unwrap())
        .arg("-p")
        .arg(&password)
        .output()
        .unwrap();

    assert!(decrypt_output.status.success());

    let decrypted = fs::read(&decrypted_file).unwrap();
    assert_eq!(decrypted, b"Test data");
}

#[test]
fn test_same_input_output_file_error() {
    let test_dir = TempDir::new().unwrap();
    let test_path = test_dir.path();

    let file = test_path.join("same_file.txt");
    fs::write(&file, b"Test data").unwrap();

    let binary = get_binary_path();

    // Try to encrypt with same input and output - this should either fail or succeed
    // depending on implementation, but should not corrupt the data
    let original_data = fs::read(&file).unwrap();
    
    let _output = Command::new(&binary)
        .arg("encrypt")
        .arg("-i")
        .arg(file.to_str().unwrap())
        .arg("-o")
        .arg(file.to_str().unwrap())
        .arg("-p")
        .arg("password")
        .output()
        .unwrap();

    // Verify either file is unchanged or properly encrypted
    let new_data = fs::read(&file).unwrap();
    assert!(!new_data.is_empty(), "File should not be empty");
    // Either unchanged or encrypted (which would be longer)
    assert!(new_data == original_data || new_data.len() > original_data.len());
}

#[test]
#[cfg(unix)]
fn test_output_to_readonly_location() {
    use std::os::unix::fs::PermissionsExt;
    
    // Skip test if running as root, as root can write to read-only directories
    let is_root = unsafe { libc::geteuid() } == 0;
    if is_root {
        eprintln!("Skipping test_output_to_readonly_location: running as root");
        return;
    }
    
    let test_dir = TempDir::new().unwrap();
    let test_path = test_dir.path();

    let input_file = test_path.join("input.txt");
    fs::write(&input_file, b"Test data").unwrap();

    let binary = get_binary_path();

    // Try to write to a read-only location (this should fail gracefully)
    let readonly_dir = test_path.join("readonly");
    fs::create_dir(&readonly_dir).unwrap();
    
    // Make directory read-only
    let mut perms = fs::metadata(&readonly_dir).unwrap().permissions();
    perms.set_mode(0o444);
    fs::set_permissions(&readonly_dir, perms).unwrap();

    let output_file = readonly_dir.join("output.enc");
    
    let output = Command::new(&binary)
        .arg("encrypt")
        .arg("-i")
        .arg(input_file.to_str().unwrap())
        .arg("-o")
        .arg(output_file.to_str().unwrap())
        .arg("-p")
        .arg("password")
        .output()
        .unwrap();

    // Should fail gracefully
    assert!(!output.status.success());
    
    // Clean up by making directory writable again
    let mut perms = fs::metadata(&readonly_dir).unwrap().permissions();
    perms.set_mode(0o755);
    let _ = fs::set_permissions(&readonly_dir, perms);
}

#[test]
fn test_whitespace_only_password() {
    let test_dir = TempDir::new().unwrap();
    let test_path = test_dir.path();

    let input_file = test_path.join("input.txt");
    let encrypted_file = test_path.join("encrypted.bin");
    let decrypted_file = test_path.join("decrypted.txt");

    fs::write(&input_file, b"Test data").unwrap();

    let binary = get_binary_path();
    let password = "   \t\n   ";

    // Encrypt with whitespace password
    let encrypt_output = Command::new(&binary)
        .arg("encrypt")
        .arg("-i")
        .arg(input_file.to_str().unwrap())
        .arg("-o")
        .arg(encrypted_file.to_str().unwrap())
        .arg("-p")
        .arg(password)
        .output()
        .unwrap();

    assert!(encrypt_output.status.success());

    // Decrypt with same whitespace password
    let decrypt_output = Command::new(&binary)
        .arg("decrypt")
        .arg("-i")
        .arg(encrypted_file.to_str().unwrap())
        .arg("-o")
        .arg(decrypted_file.to_str().unwrap())
        .arg("-p")
        .arg(password)
        .output()
        .unwrap();

    assert!(decrypt_output.status.success());

    let decrypted = fs::read(&decrypted_file).unwrap();
    assert_eq!(decrypted, b"Test data");
}

#[test]
fn test_sequential_operations() {
    let test_dir = TempDir::new().unwrap();
    let test_path = test_dir.path();

    let file1 = test_path.join("file1.txt");
    let file2 = test_path.join("file2.txt");
    let encrypted1 = test_path.join("encrypted1.bin");
    let encrypted2 = test_path.join("encrypted2.bin");
    let decrypted1 = test_path.join("decrypted1.txt");
    let decrypted2 = test_path.join("decrypted2.txt");

    fs::write(&file1, b"Data 1").unwrap();
    fs::write(&file2, b"Data 2").unwrap();

    let binary = get_binary_path();

    // Perform multiple sequential operations
    for _ in 0..5 {
        Command::new(&binary)
            .arg("encrypt")
            .arg("-i")
            .arg(file1.to_str().unwrap())
            .arg("-o")
            .arg(encrypted1.to_str().unwrap())
            .arg("-p")
            .arg("password1")
            .output()
            .unwrap();

        Command::new(&binary)
            .arg("encrypt")
            .arg("-i")
            .arg(file2.to_str().unwrap())
            .arg("-o")
            .arg(encrypted2.to_str().unwrap())
            .arg("-p")
            .arg("password2")
            .output()
            .unwrap();

        Command::new(&binary)
            .arg("decrypt")
            .arg("-i")
            .arg(encrypted1.to_str().unwrap())
            .arg("-o")
            .arg(decrypted1.to_str().unwrap())
            .arg("-p")
            .arg("password1")
            .output()
            .unwrap();

        Command::new(&binary)
            .arg("decrypt")
            .arg("-i")
            .arg(encrypted2.to_str().unwrap())
            .arg("-o")
            .arg(decrypted2.to_str().unwrap())
            .arg("-p")
            .arg("password2")
            .output()
            .unwrap();

        // Verify
        assert_eq!(fs::read(&decrypted1).unwrap(), b"Data 1");
        assert_eq!(fs::read(&decrypted2).unwrap(), b"Data 2");
    }
}
