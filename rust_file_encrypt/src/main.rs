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

pub mod crypto;

use clap::{Parser, Subcommand};
use crypto::FileCrypto;
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
