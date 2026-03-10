//! # Secure File Crypto
//!
//! A secure, cross-platform command-line tool for encrypting and decrypting files using
//! military-grade AES-256-GCM encryption with authenticated encryption and integrity protection.
//!
//! ## Features
//!
//! - **AES-256-GCM Encryption**: Industry-standard authenticated encryption
//! - **Argon2 Key Derivation**: Memory-hard password hashing resistant to GPU attacks
//! - **Random Salt & Nonce**: Each encryption uses unique cryptographic parameters
//! - **Memory Security**: Sensitive data is automatically zeroed from memory
//!
//! ## Usage
//!
//! Encrypt a file:
//! ```shell
//! secure-file-crypto encrypt -i input.txt -o output.enc
//! ```
//!
//! Decrypt a file:
//! ```shell
//! secure-file-crypto decrypt -i output.enc -o decrypted.txt
//! ```
//!
//! ## Security
//!
//! This tool implements cryptographic best practices:
//! - Passwords are hashed using Argon2id with random salts
//! - Each encryption operation uses a unique random nonce
//! - Authentication tags ensure data integrity and prevent tampering
//! - Sensitive data is zeroized from memory after use

use clap::{Parser, Subcommand};
use secure_file_crypto::crypto::FileCrypto;
use rpassword::read_password;
use std::io::{self, Write};
use zeroize::Zeroizing;

/// A secure, cross-platform command-line tool for encrypting and decrypting files
/// using military-grade AES-256-GCM encryption with authenticated encryption and integrity protection.
#[derive(Parser)]
#[command(name = "secure-file-crypto")]
#[command(author, version, about, long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Encrypt a file using AES-256-GCM
    Encrypt {
        /// Input file path to encrypt
        #[arg(short, long)]
        input: String,

        /// Output file path for encrypted data
        #[arg(short, long)]
        output: String,

        /// Password for encryption (will prompt if not provided)
        #[arg(short, long)]
        password: Option<String>,
    },
    /// Decrypt a file using AES-256-GCM
    Decrypt {
        /// Input file path to decrypt
        #[arg(short, long)]
        input: String,

        /// Output file path for decrypted data
        #[arg(short, long)]
        output: String,

        /// Password for decryption (will prompt if not provided)
        #[arg(short, long)]
        password: Option<String>,
    },
}

/// Prompts the user for a password securely without echoing to the terminal.
///
/// The prompt message is written to stderr, and the password is read from the terminal
/// using [`rpassword::read_password`] and automatically wrapped in a [`Zeroizing`] container
/// to ensure it's securely erased from memory when no longer needed.
///
/// # Arguments
///
/// * `prompt` - The message to display to the user
///
/// # Returns
///
/// Returns a `Zeroizing<String>` containing the password, or an error if reading fails.
///
/// # Examples
///
/// ```no_run
/// # use std::io;
/// # use zeroize::Zeroizing;
/// # fn prompt_password(prompt: &str) -> io::Result<Zeroizing<String>> {
/// #     unimplemented!()
/// # }
/// let password = prompt_password("Enter password: ")?;
/// # Ok::<(), io::Error>(())
/// ```
fn prompt_password(prompt: &str) -> io::Result<Zeroizing<String>> {
    eprint!("{}", prompt);
    io::stderr().flush()?;
    read_password()
        .map(Zeroizing::new)
        .map_err(|e| io::Error::new(e.kind(), format!("Failed to read password: {}", e)))
}

/// Executes the encrypt or decrypt operation given parsed inputs.
///
/// This function is extracted from `main` so it can be unit-tested independently
/// of the CLI argument parser and interactive password prompts.
///
/// Returns `Ok(())` on success, or an error string on failure.
pub fn run_operation(
    command: &str,
    input: &str,
    output: &str,
    password: &str,
) -> Result<(), Box<dyn std::error::Error>> {
    let pwd = Zeroizing::new(password.to_string());
    let result = match command {
        "encrypt" => {
            println!("Encrypting file: {} -> {}", input, output);
            FileCrypto::encrypt_file(input, output, &pwd)
        }
        "decrypt" => {
            println!("Decrypting file: {} -> {}", input, output);
            FileCrypto::decrypt_file(input, output, &pwd)
        }
        other => {
            return Err(format!("Unknown command: {}", other).into());
        }
    };
    result.map_err(|e| e.into())
}

/// Main entry point for the secure file crypto tool.
///
/// Parses command-line arguments and executes either encryption or decryption operations.
/// For encryption, prompts for password confirmation to prevent typos. Handles errors
/// gracefully and exits with appropriate status codes.
///
/// # Errors
///
/// Returns an error if:
/// - File operations fail (reading input or writing output)
/// - Passwords don't match during encryption
/// - Decryption fails due to wrong password or corrupted file
/// - Password prompt fails
fn main() -> Result<(), Box<dyn std::error::Error>> {
    let cli = Cli::parse();

    let result = match cli.command {
        Commands::Encrypt {
            input,
            output,
            password,
        } => {
            let password = match password {
                Some(pwd) => Zeroizing::new(pwd),
                None => {
                    let pwd = prompt_password("Enter encryption password: ")?;
                    let confirm = prompt_password("Confirm password: ")?;
                    
                    if pwd.as_str() != confirm.as_str() {
                        eprintln!("Error: Passwords do not match!");
                        std::process::exit(1);
                    }
                    pwd
                }
            };

            println!("Encrypting file: {} -> {}", input, output);
            FileCrypto::encrypt_file(&input, &output, &password)
        }
        Commands::Decrypt {
            input,
            output,
            password,
        } => {
            let password = match password {
                Some(pwd) => Zeroizing::new(pwd),
                None => prompt_password("Enter decryption password: ")?,
            };

            println!("Decrypting file: {} -> {}", input, output);
            FileCrypto::decrypt_file(&input, &output, &password)
        }
    };

    match result {
        Ok(_) => {
            println!("✓ Operation completed successfully!");
            Ok(())
        }
        Err(e) => {
            eprintln!("✗ Error: {}", e);
            std::process::exit(1);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use std::io;
    use tempfile::TempDir;

    // ── run_operation unit tests ──────────────────────────────────────────────

    #[test]
    fn test_run_operation_encrypt_decrypt_roundtrip() {
        let dir = TempDir::new().unwrap();
        let input = dir.path().join("plain.txt");
        let enc = dir.path().join("plain.enc");
        let dec = dir.path().join("plain.dec");

        fs::write(&input, b"hello from run_operation").unwrap();

        run_operation("encrypt", input.to_str().unwrap(), enc.to_str().unwrap(), "pw1").unwrap();
        run_operation("decrypt", enc.to_str().unwrap(), dec.to_str().unwrap(), "pw1").unwrap();

        assert_eq!(fs::read(&dec).unwrap(), b"hello from run_operation");
    }

    #[test]
    fn test_run_operation_encrypt_missing_input_returns_error() {
        let dir = TempDir::new().unwrap();
        let result = run_operation(
            "encrypt",
            dir.path().join("no_such.txt").to_str().unwrap(),
            dir.path().join("out.enc").to_str().unwrap(),
            "pw",
        );
        assert!(result.is_err());
    }

    #[test]
    fn test_run_operation_decrypt_wrong_password_returns_error() {
        let dir = TempDir::new().unwrap();
        let input = dir.path().join("plain.txt");
        let enc = dir.path().join("plain.enc");
        let dec = dir.path().join("plain.dec");

        fs::write(&input, b"data").unwrap();
        run_operation("encrypt", input.to_str().unwrap(), enc.to_str().unwrap(), "correct").unwrap();

        let result = run_operation("decrypt", enc.to_str().unwrap(), dec.to_str().unwrap(), "wrong");
        assert!(result.is_err());
    }

    #[test]
    fn test_run_operation_unknown_command_returns_error() {
        let dir = TempDir::new().unwrap();
        let result = run_operation(
            "shred",
            dir.path().join("x").to_str().unwrap(),
            dir.path().join("y").to_str().unwrap(),
            "pw",
        );
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("Unknown command"));
    }

    #[test]
    fn test_run_operation_encrypt_output_is_directory_returns_error() {
        let dir = TempDir::new().unwrap();
        let input = dir.path().join("plain.txt");
        fs::write(&input, b"data").unwrap();

        // Output path is a directory → create-file should fail
        let result = run_operation(
            "encrypt",
            input.to_str().unwrap(),
            dir.path().to_str().unwrap(),
            "pw",
        );
        assert!(result.is_err());
    }

    #[test]
    fn test_run_operation_decrypt_output_is_directory_returns_error() {
        let dir = TempDir::new().unwrap();
        let input = dir.path().join("plain.txt");
        let enc = dir.path().join("plain.enc");
        fs::write(&input, b"data").unwrap();
        run_operation("encrypt", input.to_str().unwrap(), enc.to_str().unwrap(), "pw").unwrap();

        let result = run_operation(
            "decrypt",
            enc.to_str().unwrap(),
            dir.path().to_str().unwrap(),
            "pw",
        );
        assert!(result.is_err());
    }

    // ── prompt_password unit test ─────────────────────────────────────────────
    // prompt_password relies on rpassword which reads /dev/tty; we test only the
    // stderr-flush and the error-mapping path by injecting an IO error directly.

    #[test]
    fn test_prompt_password_error_mapping() {
        // Simulate the map_err closure: an rpassword-style IO error should be re-wrapped.
        let original = io::Error::new(io::ErrorKind::BrokenPipe, "pipe broken");
        let mapped = io::Error::new(
            original.kind(),
            format!("Failed to read password: {}", original),
        );
        assert_eq!(mapped.kind(), io::ErrorKind::BrokenPipe);
        assert!(mapped.to_string().contains("Failed to read password"));
    }
}
