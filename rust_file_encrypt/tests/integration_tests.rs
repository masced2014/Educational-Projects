use std::fs;
use std::process::Command;
use tempfile::TempDir;

/// Helper to get the path to the compiled binary
fn get_binary_path() -> String {
    // Use the path provided by Cargo for the built binary under test.
    // This works reliably in CI and on all platforms.
    env!("CARGO_BIN_EXE_secure-file-crypto").to_string()
}

#[test]
fn test_cli_encrypt_decrypt_with_password_arg() {
    let test_dir = TempDir::new().unwrap();
    let test_path = test_dir.path();

    let input_file = test_path.join("input.txt");
    let encrypted_file = test_path.join("encrypted.bin");
    let decrypted_file = test_path.join("decrypted.txt");

    let test_data = b"Hello from integration test!";
    fs::write(&input_file, test_data).unwrap();

    let binary = get_binary_path();
    let password = "test_password_123";

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
        .expect("Failed to execute encrypt command");

    assert!(encrypt_output.status.success(), 
        "Encrypt failed: {}", 
        String::from_utf8_lossy(&encrypt_output.stderr));

    // Verify encrypted file exists
    assert!(encrypted_file.exists());

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
        .expect("Failed to execute decrypt command");

    assert!(decrypt_output.status.success(), 
        "Decrypt failed: {}", 
        String::from_utf8_lossy(&decrypt_output.stderr));

    // Verify decrypted data matches original
    let decrypted_data = fs::read(&decrypted_file).unwrap();
    assert_eq!(decrypted_data, test_data);
}

#[test]
fn test_cli_decrypt_wrong_password() {
    let test_dir = TempDir::new().unwrap();
    let test_path = test_dir.path();

    let input_file = test_path.join("input.txt");
    let encrypted_file = test_path.join("encrypted.bin");
    let decrypted_file = test_path.join("decrypted.txt");

    fs::write(&input_file, b"Secret data").unwrap();

    let binary = get_binary_path();

    // Encrypt with correct password
    let encrypt_output = Command::new(&binary)
        .arg("encrypt")
        .arg("-i")
        .arg(input_file.to_str().unwrap())
        .arg("-o")
        .arg(encrypted_file.to_str().unwrap())
        .arg("-p")
        .arg("correct_password")
        .output()
        .expect("Failed to execute encrypt command");

    assert!(encrypt_output.status.success());

    // Try to decrypt with wrong password
    let decrypt_output = Command::new(&binary)
        .arg("decrypt")
        .arg("-i")
        .arg(encrypted_file.to_str().unwrap())
        .arg("-o")
        .arg(decrypted_file.to_str().unwrap())
        .arg("-p")
        .arg("wrong_password")
        .output()
        .expect("Failed to execute decrypt command");

    // Should fail
    assert!(!decrypt_output.status.success());
    
    // Verify error message mentions decryption failure
    let stderr = String::from_utf8_lossy(&decrypt_output.stderr);
    assert!(stderr.contains("Decryption failed") || stderr.contains("Error"));
}

#[test]
fn test_cli_missing_input_file() {
    let test_dir = TempDir::new().unwrap();
    let test_path = test_dir.path();

    let nonexistent_file = test_path.join("nonexistent.txt");
    let output_file = test_path.join("output.bin");

    let binary = get_binary_path();

    let output = Command::new(&binary)
        .arg("encrypt")
        .arg("-i")
        .arg(nonexistent_file.to_str().unwrap())
        .arg("-o")
        .arg(output_file.to_str().unwrap())
        .arg("-p")
        .arg("password")
        .output()
        .expect("Failed to execute command");

    // Should fail
    assert!(!output.status.success());
    
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stderr.contains("Error") || stderr.contains("Failed"));
}

#[test]
fn test_cli_help_command() {
    let binary = get_binary_path();

    let output = Command::new(&binary)
        .arg("--help")
        .output()
        .expect("Failed to execute help command");

    assert!(output.status.success());
    
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("encrypt"));
    assert!(stdout.contains("decrypt"));
}

#[test]
fn test_cli_version_command() {
    let binary = get_binary_path();

    let output = Command::new(&binary)
        .arg("--version")
        .output()
        .expect("Failed to execute version command");

    assert!(output.status.success());
}

#[test]
fn test_cli_encrypt_help() {
    let binary = get_binary_path();

    let output = Command::new(&binary)
        .arg("encrypt")
        .arg("--help")
        .output()
        .expect("Failed to execute encrypt help");

    assert!(output.status.success());
    
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("input"));
    assert!(stdout.contains("output"));
    assert!(stdout.contains("password"));
}

#[test]
fn test_cli_decrypt_help() {
    let binary = get_binary_path();

    let output = Command::new(&binary)
        .arg("decrypt")
        .arg("--help")
        .output()
        .expect("Failed to execute decrypt help");

    assert!(output.status.success());
    
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("input"));
    assert!(stdout.contains("output"));
    assert!(stdout.contains("password"));
}

#[test]
fn test_cli_multiple_file_operations() {
    let test_dir = TempDir::new().unwrap();
    let test_path = test_dir.path();

    let binary = get_binary_path();
    let password = "multi_file_password";

    // Create multiple test files
    let files = vec![
        ("file1.txt", b"Content 1"),
        ("file2.txt", b"Content 2"),
        ("file3.txt", b"Content 3"),
    ];

    for (filename, content) in &files {
        let input_file = test_path.join(filename);
        let encrypted_file = test_path.join(format!("{}.enc", filename));
        let decrypted_file = test_path.join(format!("{}.dec", filename));

        fs::write(&input_file, content).unwrap();

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

        // Verify
        let decrypted_data = fs::read(&decrypted_file).unwrap();
        assert_eq!(&decrypted_data, content);
    }
}

#[test]
fn test_cli_empty_file() {
    let test_dir = TempDir::new().unwrap();
    let test_path = test_dir.path();

    let input_file = test_path.join("empty.txt");
    let encrypted_file = test_path.join("empty.enc");
    let decrypted_file = test_path.join("empty.dec");

    fs::write(&input_file, b"").unwrap();

    let binary = get_binary_path();
    let password = "empty_file_pass";

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

    // Verify empty file
    let decrypted_data = fs::read(&decrypted_file).unwrap();
    assert_eq!(decrypted_data.len(), 0);
}

#[test]
fn test_cli_binary_file() {
    let test_dir = TempDir::new().unwrap();
    let test_path = test_dir.path();

    let input_file = test_path.join("binary.bin");
    let encrypted_file = test_path.join("binary.enc");
    let decrypted_file = test_path.join("binary.dec");

    // Create binary data with all byte values
    let binary_data: Vec<u8> = (0..=255).collect();
    fs::write(&input_file, &binary_data).unwrap();

    let binary = get_binary_path();
    let password = "binary_test_pass";

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

    // Verify all bytes preserved
    let decrypted_data = fs::read(&decrypted_file).unwrap();
    assert_eq!(decrypted_data, binary_data);
}

#[test]
fn test_cli_invalid_encrypted_file() {
    let test_dir = TempDir::new().unwrap();
    let test_path = test_dir.path();

    let invalid_file = test_path.join("invalid.bin");
    let output_file = test_path.join("output.txt");

    // Create an invalid encrypted file
    fs::write(&invalid_file, b"not a valid encrypted file").unwrap();

    let binary = get_binary_path();

    let output = Command::new(&binary)
        .arg("decrypt")
        .arg("-i")
        .arg(invalid_file.to_str().unwrap())
        .arg("-o")
        .arg(output_file.to_str().unwrap())
        .arg("-p")
        .arg("password")
        .output()
        .unwrap();

    // Should fail
    assert!(!output.status.success());
}

// ── Interactive / stdin password tests ──────────────────────────────────────

/// Encrypt with mismatched interactive passwords must exit non-zero.
/// We use `echo` piping via shell to provide two different passwords on stdin.
/// Encrypt to a directory path (not a file) must hit the error branch and exit non-zero.
#[test]
fn test_cli_encrypt_output_is_directory() {
    let test_dir = TempDir::new().unwrap();
    let test_path = test_dir.path();

    let input_file = test_path.join("input.txt");
    fs::write(&input_file, b"dir output test").unwrap();

    // Use the temp dir itself as the output path — that is a directory, not a file
    let binary = get_binary_path();
    let output = Command::new(&binary)
        .arg("encrypt")
        .arg("-i")
        .arg(input_file.to_str().unwrap())
        .arg("-o")
        .arg(test_path.to_str().unwrap())
        .arg("-p")
        .arg("some_pass")
        .output()
        .expect("Failed to execute command");

    assert!(
        !output.status.success(),
        "Should fail when output path is a directory"
    );
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stderr.contains("Error") || stderr.contains("Failed") || stderr.contains("error"));
}

/// Decrypt a valid encrypted file to a directory path must hit the error branch.
#[test]
fn test_cli_decrypt_output_is_directory() {
    let test_dir = TempDir::new().unwrap();
    let test_path = test_dir.path();

    let input_file = test_path.join("input.txt");
    let encrypted_file = test_path.join("encrypted.bin");

    fs::write(&input_file, b"dir output decrypt test").unwrap();

    let binary = get_binary_path();
    let password = "dir_test_pass";

    // First encrypt successfully
    let enc = Command::new(&binary)
        .arg("encrypt")
        .arg("-i")
        .arg(input_file.to_str().unwrap())
        .arg("-o")
        .arg(encrypted_file.to_str().unwrap())
        .arg("-p")
        .arg(password)
        .output()
        .unwrap();
    assert!(enc.status.success());

    // Now decrypt with the directory as output — must fail
    let output = Command::new(&binary)
        .arg("decrypt")
        .arg("-i")
        .arg(encrypted_file.to_str().unwrap())
        .arg("-o")
        .arg(test_path.to_str().unwrap())
        .arg("-p")
        .arg(password)
        .output()
        .expect("Failed to execute command");

    assert!(
        !output.status.success(),
        "Should fail when output path is a directory"
    );
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stderr.contains("Error") || stderr.contains("Failed") || stderr.contains("error"));
}
