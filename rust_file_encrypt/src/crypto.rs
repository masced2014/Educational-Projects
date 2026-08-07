//! Cryptographic operations for file encryption and decryption.
//!
//! This module provides secure file encryption and decryption using AES-256-GCM
//! (Advanced Encryption Standard with 256-bit keys in Galois/Counter Mode) and
//! Argon2id for password-based key derivation.
//!
//! # Security Features
//!
//! - **Authenticated Encryption**: AES-GCM provides both confidentiality and authenticity
//! - **Key Derivation**: Argon2id protects against brute-force and GPU attacks
//! - **Random Parameters**: Each encryption uses unique salt and nonce values
//! - **Memory Security**: Encryption keys are automatically zeroized after use
//!
//! # File Format
//!
//! Encrypted files have the following structure:
//! ```text
//! [SALT (16 bytes)][NONCE (12 bytes)][CIPHERTEXT || AUTH_TAG]
//! ```
//!
//! Where:
//! - **SALT**: Random 16-byte value for Argon2 key derivation
//! - **NONCE**: Random 12-byte value for AES-GCM encryption
//! - **CIPHERTEXT**: The encrypted data
//! - **AUTH_TAG**: 16-byte authentication tag for integrity verification
//!
//! # Examples
//!
//! ```no_run
//! use secure_file_crypto::crypto::FileCrypto;
//!
//! // Encrypt a file
//! FileCrypto::encrypt_file("input.txt", "output.enc", "my_password")?;
//!
//! // Decrypt a file
//! FileCrypto::decrypt_file("output.enc", "decrypted.txt", "my_password")?;
//! # Ok::<(), anyhow::Error>(())
//! ```

use aes_gcm::{
    aead::{Aead, KeyInit},
    Aes256Gcm, Nonce,
};
use argon2::{
    password_hash::rand_core::{OsRng, RngCore},
    Argon2,
};
#[cfg(feature = "fast-kdf")]
use argon2::{Algorithm, Params, Version};
use anyhow::{Context, Result};
use std::fs::File;
use std::io::{BufReader, BufWriter, Read, Write};
use zeroize::Zeroizing;

/// Salt size for Argon2 (16 bytes is standard)
const SALT_SIZE: usize = 16;
/// Nonce size for AES-GCM (12 bytes is standard for AES-GCM)
const NONCE_SIZE: usize = 12;
/// Key size for AES-256 (32 bytes = 256 bits)
const KEY_SIZE: usize = 32;
/// Authentication tag size for AES-GCM (16 bytes)
/// This must match the tag size produced by AES-GCM (implicitly 16 bytes for Aes256Gcm)
const AUTH_TAG_SIZE: usize = 16;

/// Stateless helper for encrypting and decrypting files using AES-256-GCM
/// with keys derived from passwords via Argon2id.
///
/// This type provides simple, high-level methods that operate on file paths.
/// The exact on-disk encrypted file format is documented at the module level.
///
/// # Examples
///
/// ```no_run
/// use secure_file_crypto::crypto::FileCrypto;
///
/// // Encrypt a file
/// let result = FileCrypto::encrypt_file("plaintext.txt", "encrypted.bin", "strong_password");
/// assert!(result.is_ok());
///
/// // Decrypt the file
/// let result = FileCrypto::decrypt_file("encrypted.bin", "decrypted.txt", "strong_password");
/// assert!(result.is_ok());
/// # Ok::<(), anyhow::Error>(())
/// ```
pub struct FileCrypto;

impl FileCrypto {
    /// Derives a 256-bit encryption key from a password using Argon2id.
    ///
    /// This function converts a user password into a cryptographically strong
    /// 256-bit key suitable for AES-256.  The Argon2 parameters used depend on
    /// the active feature flags (see **Security Notes** below).
    ///
    /// # Arguments
    ///
    /// * `password` - The user's password as a string
    /// * `salt` - A random 16-byte salt value (should be unique per encryption)
    ///
    /// # Returns
    ///
    /// Returns a [`Zeroizing`] wrapper containing the derived key. The key is automatically
    /// zeroed from memory when dropped, preventing sensitive data from lingering.
    ///
    /// # Security Notes
    ///
    /// **Without the `fast-kdf` feature** (production default): uses `Argon2::default()`
    /// parameters — memory-hard Argon2id with production-strength cost settings
    /// (~32 ms per call).
    ///
    /// **With the `fast-kdf` feature** (fuzz / CI only): uses explicitly minimised
    /// parameters (m=8 KiB, t=1 iteration, p=1 lane) so that fuzz targets can
    /// exercise hundreds of encrypt/decrypt cycles per second.  **Never enable
    /// `fast-kdf` in a production build** — these parameters provide no meaningful
    /// brute-force resistance.
    ///
    /// The current implementation does not store Argon2 version or parameters in
    /// the file header, which means files are tied to the crate's compile-time
    /// defaults.  If parameters need to change in the future, consider:
    /// (1) implementing file-format versioning with explicit KDF parameters, or
    /// (2) documenting the Argon2 version and settings used to create existing
    /// files for potential migration tools.
    ///
    /// # Errors
    ///
    /// Returns an error if the key derivation process fails.
    fn derive_key(password: &str, salt: &[u8]) -> Result<Zeroizing<[u8; KEY_SIZE]>> {
        // Production build: use Argon2id default parameters (memory-hard, ~32 ms/call).
        // fast-kdf feature: use absolute minimum parameters so fuzz targets can run at
        // hundreds of iterations per second.  NEVER enable `fast-kdf` in production.
        #[cfg(not(feature = "fast-kdf"))]
        let argon2 = Argon2::default();

        #[cfg(feature = "fast-kdf")]
        let argon2 = Argon2::new(
            Algorithm::Argon2id,
            Version::V0x13,
            // m=8 KiB, t=1 iteration, p=1 lane — absolute minimum accepted by argon2 0.5
            Params::new(8, 1, 1, None)
                .map_err(|e| anyhow::anyhow!("fast-kdf Params::new failed: {}", e))?,
        );

        let mut key = Zeroizing::new([0u8; KEY_SIZE]);
        
        argon2
            .hash_password_into(password.as_bytes(), salt, key.as_mut())
            .map_err(|e| anyhow::anyhow!("Failed to derive key: {}", e))?;
        
        Ok(key)
    }

    /// Encrypts a file using AES-256-GCM with password-based key derivation.
    ///
    /// This function reads a plaintext file, encrypts its contents using AES-256-GCM,
    /// and writes the encrypted data to an output file. The encryption process includes:
    /// - Generating a random salt for key derivation
    /// - Deriving an encryption key from the password using Argon2id
    /// - Generating a random nonce for AES-GCM
    /// - Encrypting the data with authenticated encryption
    ///
    /// # Arguments
    ///
    /// * `input_path` - Path to the plaintext file to encrypt
    /// * `output_path` - Path where the encrypted file will be written
    /// * `password` - Password used for encryption (will be used with Argon2id key derivation)
    ///
    /// # Returns
    ///
    /// Returns `Ok(())` if encryption succeeds, or an error if any step fails.
    ///
    /// # File Format
    ///
    /// The output file contains:
    /// ```text
    /// [16-byte salt][12-byte nonce][encrypted data with 16-byte auth tag]
    /// ```
    ///
    /// # Security
    ///
    /// - Each encryption generates unique random salt and nonce values
    /// - Authentication tag ensures data integrity and prevents tampering
    /// - Encryption key is automatically zeroed from memory after use
    ///
    /// # Performance
    ///
    /// For large files, consider the memory usage as this loads the entire file into memory.
    /// The implementation uses buffered I/O but still processes the file in memory.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use secure_file_crypto::crypto::FileCrypto;
    ///
    /// let result = FileCrypto::encrypt_file(
    ///     "secret.txt",
    ///     "secret.txt.enc",
    ///     "my_strong_password"
    /// );
    /// assert!(result.is_ok());
    /// # Ok::<(), anyhow::Error>(())
    /// ```
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// - The input file cannot be read
    /// - The output file cannot be created or written
    /// - Key derivation fails
    /// - Encryption fails
    pub fn encrypt_file(input_path: &str, output_path: &str, password: &str) -> Result<()> {
        // Read the input file with buffering
        let input_file = File::open(input_path)
            .context(format!("Failed to open input file: {}", input_path))?;
        let mut reader = BufReader::new(input_file);
        let mut plaintext = Vec::new();
        reader.read_to_end(&mut plaintext)
            .context("Failed to read input file")?;

        // Generate random salt
        let mut salt = [0u8; SALT_SIZE];
        OsRng.fill_bytes(&mut salt);

        // Derive encryption key from password (automatically zeroized via Zeroizing)
        let key = Self::derive_key(password, &salt)?;

        // Create cipher instance
        let cipher = Aes256Gcm::new_from_slice(key.as_ref())
            .map_err(|e| anyhow::anyhow!("Failed to create cipher: {}", e))?;

        // Generate random nonce
        let mut nonce_bytes = [0u8; NONCE_SIZE];
        OsRng.fill_bytes(&mut nonce_bytes);
        let nonce = Nonce::from_slice(&nonce_bytes);

        // Encrypt the data
        let ciphertext = cipher
            .encrypt(nonce, plaintext.as_ref())
            .map_err(|e| anyhow::anyhow!("Encryption failed: {}", e))?;

        // Write salt, nonce, and ciphertext to output file
        let output_file = File::create(output_path)
            .context(format!("Failed to create output file: {}", output_path))?;
        let mut writer = BufWriter::new(output_file);

        writer
            .write_all(&salt)
            .context("Failed to write salt")?;
        writer
            .write_all(&nonce_bytes)
            .context("Failed to write nonce")?;
        writer
            .write_all(&ciphertext)
            .context("Failed to write ciphertext")?;
        
        writer.flush().context("Failed to flush output")?;

        // Key is automatically zeroized when it goes out of scope (Zeroizing wrapper)
        Ok(())
    }

    /// Decrypts a file that was encrypted using AES-256-GCM.
    ///
    /// This function reads an encrypted file, extracts the salt and nonce, derives the
    /// decryption key from the password, and decrypts the data. The decryption process:
    /// - Reads and validates the encrypted file structure
    /// - Extracts the salt and nonce from the file header
    /// - Derives the decryption key from the password using Argon2id
    /// - Decrypts and authenticates the data using AES-GCM
    ///
    /// # Arguments
    ///
    /// * `input_path` - Path to the encrypted file
    /// * `output_path` - Path where the decrypted plaintext will be written
    /// * `password` - Password used for decryption (must match the encryption password)
    ///
    /// # Returns
    ///
    /// Returns `Ok(())` if decryption succeeds, or an error if any step fails.
    ///
    /// # Security
    ///
    /// - Authentication tag verification detects tampering
    /// - Incorrect passwords or corrupted files will fail authentication
    /// - Decryption key is automatically zeroed from memory after use
    ///
    /// # Performance
    ///
    /// For large files, consider the memory usage as this loads the entire file into memory.
    /// The implementation uses buffered I/O but still processes the file in memory.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use secure_file_crypto::crypto::FileCrypto;
    ///
    /// // Decrypt a file (password must match the one used for encryption)
    /// let result = FileCrypto::decrypt_file(
    ///     "secret.txt.enc",
    ///     "decrypted.txt",
    ///     "my_strong_password"
    /// );
    /// assert!(result.is_ok());
    /// # Ok::<(), anyhow::Error>(())
    /// ```
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// - The input file cannot be read
    /// - The file is too small or has invalid format
    /// - The password is incorrect
    /// - The file has been tampered with or corrupted
    /// - The output file cannot be created or written
    /// - Key derivation fails
    pub fn decrypt_file(input_path: &str, output_path: &str, password: &str) -> Result<()> {
        // Read the encrypted file with buffering
        let input_file = File::open(input_path)
            .context(format!("Failed to open input file: {}", input_path))?;
        let mut reader = BufReader::new(input_file);
        let mut encrypted_data = Vec::new();
        reader.read_to_end(&mut encrypted_data)
            .context("Failed to read encrypted file")?;

        // Verify minimum file size (salt + nonce + at least auth tag)
        let min_size = SALT_SIZE + NONCE_SIZE + AUTH_TAG_SIZE;
        if encrypted_data.len() < min_size {
            anyhow::bail!(
                "Invalid encrypted file: too small (expected at least {} bytes, got {})",
                min_size,
                encrypted_data.len()
            );
        }

        // Extract salt, nonce, and ciphertext
        let salt = &encrypted_data[0..SALT_SIZE];
        let nonce_bytes = &encrypted_data[SALT_SIZE..SALT_SIZE + NONCE_SIZE];
        let ciphertext = &encrypted_data[SALT_SIZE + NONCE_SIZE..];

        // Derive decryption key from password (automatically zeroized via Zeroizing)
        let key = Self::derive_key(password, salt)?;

        // Create cipher instance
        let cipher = Aes256Gcm::new_from_slice(key.as_ref())
            .map_err(|e| anyhow::anyhow!("Failed to create cipher: {}", e))?;

        // Create nonce
        let nonce = Nonce::from_slice(nonce_bytes);

        // Decrypt the data
        let plaintext = cipher
            .decrypt(nonce, ciphertext)
            .map_err(|_| anyhow::anyhow!("Decryption failed: incorrect password or corrupted file"))?;

        // Write plaintext to output file with buffering
        let output_file = File::create(output_path)
            .context(format!("Failed to create output file: {}", output_path))?;
        let mut writer = BufWriter::new(output_file);
        writer.write_all(&plaintext)
            .context("Failed to write decrypted data")?;
        writer.flush().context("Failed to flush output")?;

        // Key is automatically zeroized when it goes out of scope (Zeroizing wrapper)
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::TempDir;

    #[test]
    fn test_encrypt_decrypt_roundtrip() {
        let test_dir = TempDir::new().unwrap();
        let test_path = test_dir.path();

        let input_file = test_path.join("test_input.txt");
        let encrypted_file = test_path.join("test_encrypted.bin");
        let decrypted_file = test_path.join("test_decrypted.txt");

        // Create test data
        let test_data = b"This is a secret message that needs encryption!";
        fs::write(&input_file, test_data).unwrap();

        let password = "super_secret_password_123";

        // Encrypt
        FileCrypto::encrypt_file(
            input_file.to_str().unwrap(),
            encrypted_file.to_str().unwrap(),
            password,
        )
        .unwrap();

        // Verify encrypted file exists and is different from input
        assert!(fs::metadata(&encrypted_file).unwrap().len() > 0);
        let encrypted_data = fs::read(&encrypted_file).unwrap();
        assert_ne!(&encrypted_data[SALT_SIZE + NONCE_SIZE..], test_data);

        // Decrypt
        FileCrypto::decrypt_file(
            encrypted_file.to_str().unwrap(),
            decrypted_file.to_str().unwrap(),
            password,
        )
        .unwrap();

        // Verify decrypted data matches original
        let decrypted_data = fs::read(&decrypted_file).unwrap();
        assert_eq!(decrypted_data, test_data);

        // TempDir automatically cleans up when dropped
    }

    #[test]
    fn test_decrypt_wrong_password() {
        let test_dir = TempDir::new().unwrap();
        let test_path = test_dir.path();

        let input_file = test_path.join("test_input.txt");
        let encrypted_file = test_path.join("test_encrypted.bin");
        let decrypted_file = test_path.join("test_decrypted.txt");

        // Create test data
        let test_data = b"Secret data";
        fs::write(&input_file, test_data).unwrap();

        let password = "correct_password";
        let wrong_password = "wrong_password";

        // Encrypt
        FileCrypto::encrypt_file(
            input_file.to_str().unwrap(),
            encrypted_file.to_str().unwrap(),
            password,
        )
        .unwrap();

        // Try to decrypt with wrong password
        let result = FileCrypto::decrypt_file(
            encrypted_file.to_str().unwrap(),
            decrypted_file.to_str().unwrap(),
            wrong_password,
        );
        assert!(result.is_err());

        // TempDir automatically cleans up when dropped
    }

    #[test]
    fn test_decrypt_invalid_file() {
        let test_dir = TempDir::new().unwrap();
        let test_path = test_dir.path();

        let invalid_file = test_path.join("invalid.bin");
        let output_file = test_path.join("output.txt");

        // Create a file that's too small
        fs::write(&invalid_file, b"short").unwrap();

        let result = FileCrypto::decrypt_file(
            invalid_file.to_str().unwrap(),
            output_file.to_str().unwrap(),
            "password",
        );
        assert!(result.is_err());

        // TempDir automatically cleans up when dropped
    }

    #[test]
    fn test_empty_file_encryption() {
        let test_dir = TempDir::new().unwrap();
        let test_path = test_dir.path();

        let input_file = test_path.join("empty.txt");
        let encrypted_file = test_path.join("empty_encrypted.bin");
        let decrypted_file = test_path.join("empty_decrypted.txt");

        // Create empty file
        fs::write(&input_file, b"").unwrap();

        let password = "test_password";

        // Encrypt empty file
        FileCrypto::encrypt_file(
            input_file.to_str().unwrap(),
            encrypted_file.to_str().unwrap(),
            password,
        )
        .unwrap();

        // Verify encrypted file contains salt + nonce + auth tag
        let encrypted_size = fs::metadata(&encrypted_file).unwrap().len();
        assert_eq!(encrypted_size, (SALT_SIZE + NONCE_SIZE + AUTH_TAG_SIZE) as u64);

        // Decrypt and verify
        FileCrypto::decrypt_file(
            encrypted_file.to_str().unwrap(),
            decrypted_file.to_str().unwrap(),
            password,
        )
        .unwrap();

        let decrypted_data = fs::read(&decrypted_file).unwrap();
        assert_eq!(decrypted_data.len(), 0);
    }

    #[test]
    fn test_large_file_encryption() {
        let test_dir = TempDir::new().unwrap();
        let test_path = test_dir.path();

        let input_file = test_path.join("large.bin");
        let encrypted_file = test_path.join("large_encrypted.bin");
        let decrypted_file = test_path.join("large_decrypted.bin");

        // Create a 1MB file with repeating pattern
        let pattern = b"ABCDEFGHIJKLMNOPQRSTUVWXYZ0123456789";
        let mut large_data = Vec::new();
        for _ in 0..(1024 * 1024 / pattern.len()) {
            large_data.extend_from_slice(pattern);
        }
        fs::write(&input_file, &large_data).unwrap();

        let password = "large_file_password";

        // Encrypt
        FileCrypto::encrypt_file(
            input_file.to_str().unwrap(),
            encrypted_file.to_str().unwrap(),
            password,
        )
        .unwrap();

        // Decrypt
        FileCrypto::decrypt_file(
            encrypted_file.to_str().unwrap(),
            decrypted_file.to_str().unwrap(),
            password,
        )
        .unwrap();

        // Verify content matches
        let decrypted_data = fs::read(&decrypted_file).unwrap();
        assert_eq!(decrypted_data, large_data);
    }

    #[test]
    fn test_special_characters_in_password() {
        let test_dir = TempDir::new().unwrap();
        let test_path = test_dir.path();

        let input_file = test_path.join("test.txt");
        let encrypted_file = test_path.join("encrypted.bin");
        let decrypted_file = test_path.join("decrypted.txt");

        let test_data = b"Test data";
        fs::write(&input_file, test_data).unwrap();

        // Test with special characters, unicode, spaces, etc.
        let passwords = vec![
            "password with spaces",
            "pass!@#$%^&*()_+-={}[]|\\:;\"'<>,.?/",
            "пароль_кириллица_🔐",
            "密码中文",
            "パスワード日本語",
        ];

        for password in passwords {
            // Encrypt
            FileCrypto::encrypt_file(
                input_file.to_str().unwrap(),
                encrypted_file.to_str().unwrap(),
                password,
            )
            .unwrap();

            // Decrypt
            FileCrypto::decrypt_file(
                encrypted_file.to_str().unwrap(),
                decrypted_file.to_str().unwrap(),
                password,
            )
            .unwrap();

            // Verify
            let decrypted_data = fs::read(&decrypted_file).unwrap();
            assert_eq!(decrypted_data, test_data);

            // Clean up for next iteration
            fs::remove_file(&encrypted_file).unwrap();
            fs::remove_file(&decrypted_file).unwrap();
        }
    }

    #[test]
    fn test_salt_uniqueness() {
        let test_dir = TempDir::new().unwrap();
        let test_path = test_dir.path();

        let input_file = test_path.join("test.txt");
        fs::write(&input_file, b"Same data").unwrap();

        let password = "same_password";
        let mut salts = Vec::new();

        // Encrypt same data with same password multiple times
        for i in 0..10 {
            let encrypted_file = test_path.join(format!("encrypted_{}.bin", i));
            FileCrypto::encrypt_file(
                input_file.to_str().unwrap(),
                encrypted_file.to_str().unwrap(),
                password,
            )
            .unwrap();

            // Extract salt from encrypted file
            let encrypted_data = fs::read(&encrypted_file).unwrap();
            let salt = &encrypted_data[0..SALT_SIZE];
            salts.push(salt.to_vec());
        }

        // Verify all salts are unique
        for i in 0..salts.len() {
            for j in (i + 1)..salts.len() {
                assert_ne!(
                    salts[i], salts[j],
                    "Salts should be unique for each encryption"
                );
            }
        }
    }

    #[test]
    fn test_nonce_uniqueness() {
        let test_dir = TempDir::new().unwrap();
        let test_path = test_dir.path();

        let input_file = test_path.join("test.txt");
        fs::write(&input_file, b"Same data").unwrap();

        let password = "same_password";
        let mut nonces = Vec::new();

        // Encrypt same data with same password multiple times
        for i in 0..10 {
            let encrypted_file = test_path.join(format!("encrypted_{}.bin", i));
            FileCrypto::encrypt_file(
                input_file.to_str().unwrap(),
                encrypted_file.to_str().unwrap(),
                password,
            )
            .unwrap();

            // Extract nonce from encrypted file
            let encrypted_data = fs::read(&encrypted_file).unwrap();
            let nonce = &encrypted_data[SALT_SIZE..SALT_SIZE + NONCE_SIZE];
            nonces.push(nonce.to_vec());
        }

        // Verify all nonces are unique
        for i in 0..nonces.len() {
            for j in (i + 1)..nonces.len() {
                assert_ne!(
                    nonces[i], nonces[j],
                    "Nonces should be unique for each encryption"
                );
            }
        }
    }

    #[test]
    fn test_ciphertext_differs() {
        let test_dir = TempDir::new().unwrap();
        let test_path = test_dir.path();

        let input_file = test_path.join("test.txt");
        fs::write(&input_file, b"Same data for encryption").unwrap();

        let password = "same_password";
        let encrypted_file1 = test_path.join("encrypted1.bin");
        let encrypted_file2 = test_path.join("encrypted2.bin");

        // Encrypt same data twice
        FileCrypto::encrypt_file(
            input_file.to_str().unwrap(),
            encrypted_file1.to_str().unwrap(),
            password,
        )
        .unwrap();

        FileCrypto::encrypt_file(
            input_file.to_str().unwrap(),
            encrypted_file2.to_str().unwrap(),
            password,
        )
        .unwrap();

        // Read encrypted files
        let encrypted1 = fs::read(&encrypted_file1).unwrap();
        let encrypted2 = fs::read(&encrypted_file2).unwrap();

        // Verify encrypted outputs differ (due to random salt and nonce)
        assert_ne!(
            encrypted1, encrypted2,
            "Encrypted outputs should differ due to random salt/nonce"
        );
    }

    #[test]
    fn test_tampering_detection() {
        let test_dir = TempDir::new().unwrap();
        let test_path = test_dir.path();

        let input_file = test_path.join("test.txt");
        let encrypted_file = test_path.join("encrypted.bin");
        let decrypted_file = test_path.join("decrypted.txt");

        fs::write(&input_file, b"Important data").unwrap();
        let password = "test_password";

        // Encrypt
        FileCrypto::encrypt_file(
            input_file.to_str().unwrap(),
            encrypted_file.to_str().unwrap(),
            password,
        )
        .unwrap();

        // Tamper with the encrypted file
        let mut encrypted_data = fs::read(&encrypted_file).unwrap();
        // Flip a bit in the ciphertext
        if encrypted_data.len() > SALT_SIZE + NONCE_SIZE + 1 {
            encrypted_data[SALT_SIZE + NONCE_SIZE + 1] ^= 0x01;
        }
        fs::write(&encrypted_file, encrypted_data).unwrap();

        // Attempt to decrypt tampered file
        let result = FileCrypto::decrypt_file(
            encrypted_file.to_str().unwrap(),
            decrypted_file.to_str().unwrap(),
            password,
        );

        // Should fail due to authentication tag mismatch
        assert!(result.is_err());
    }

    #[test]
    fn test_key_derivation_consistency() {
        // Test that the same password and salt always produce the same key
        let password = "test_password_123";
        let salt = [0x42u8; SALT_SIZE];

        let key1 = FileCrypto::derive_key(password, &salt).unwrap();
        let key2 = FileCrypto::derive_key(password, &salt).unwrap();

        assert_eq!(
            key1.as_ref(),
            key2.as_ref(),
            "Same password and salt should produce same key"
        );
    }

    #[test]
    fn test_key_derivation_different_salts() {
        // Test that different salts produce different keys
        let password = "test_password_123";
        let salt1 = [0x42u8; SALT_SIZE];
        let salt2 = [0x43u8; SALT_SIZE];

        let key1 = FileCrypto::derive_key(password, &salt1).unwrap();
        let key2 = FileCrypto::derive_key(password, &salt2).unwrap();

        assert_ne!(
            key1.as_ref(),
            key2.as_ref(),
            "Different salts should produce different keys"
        );
    }

    #[test]
    fn test_missing_input_file() {
        let test_dir = TempDir::new().unwrap();
        let test_path = test_dir.path();

        let nonexistent_file = test_path.join("nonexistent.txt");
        let output_file = test_path.join("output.bin");

        let result = FileCrypto::encrypt_file(
            nonexistent_file.to_str().unwrap(),
            output_file.to_str().unwrap(),
            "password",
        );

        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("Failed to open input file"));
    }

    #[test]
    fn test_binary_data_encryption() {
        let test_dir = TempDir::new().unwrap();
        let test_path = test_dir.path();

        let input_file = test_path.join("binary.bin");
        let encrypted_file = test_path.join("encrypted.bin");
        let decrypted_file = test_path.join("decrypted.bin");

        // Create binary data with all possible byte values
        let binary_data: Vec<u8> = (0..=255).collect();
        fs::write(&input_file, &binary_data).unwrap();

        let password = "binary_test";

        // Encrypt
        FileCrypto::encrypt_file(
            input_file.to_str().unwrap(),
            encrypted_file.to_str().unwrap(),
            password,
        )
        .unwrap();

        // Decrypt
        FileCrypto::decrypt_file(
            encrypted_file.to_str().unwrap(),
            decrypted_file.to_str().unwrap(),
            password,
        )
        .unwrap();

        // Verify all bytes preserved
        let decrypted_data = fs::read(&decrypted_file).unwrap();
        assert_eq!(decrypted_data, binary_data);
    }

    #[test]
    fn test_file_format_validation() {
        let test_dir = TempDir::new().unwrap();
        let test_path = test_dir.path();

        let invalid_file = test_path.join("invalid.bin");
        let output_file = test_path.join("output.txt");

        // Test with exact minimum size (should still fail as it's not valid encrypted data)
        let min_size = SALT_SIZE + NONCE_SIZE + AUTH_TAG_SIZE;
        let invalid_data = vec![0u8; min_size];
        fs::write(&invalid_file, invalid_data).unwrap();

        let result = FileCrypto::decrypt_file(
            invalid_file.to_str().unwrap(),
            output_file.to_str().unwrap(),
            "password",
        );

        // Should fail authentication
        assert!(result.is_err());
    }

    #[test]
    fn test_different_passwords_produce_different_ciphertexts() {
        let test_dir = TempDir::new().unwrap();
        let test_path = test_dir.path();

        let input_file = test_path.join("test.txt");
        fs::write(&input_file, b"Same plaintext").unwrap();

        let encrypted_file1 = test_path.join("encrypted1.bin");
        let encrypted_file2 = test_path.join("encrypted2.bin");

        // Encrypt with different passwords (but same salt would still differ due to key derivation)
        FileCrypto::encrypt_file(
            input_file.to_str().unwrap(),
            encrypted_file1.to_str().unwrap(),
            "password1",
        )
        .unwrap();

        FileCrypto::encrypt_file(
            input_file.to_str().unwrap(),
            encrypted_file2.to_str().unwrap(),
            "password2",
        )
        .unwrap();

        let encrypted1 = fs::read(&encrypted_file1).unwrap();
        let encrypted2 = fs::read(&encrypted_file2).unwrap();

        // The ciphertext portions should be different
        assert_ne!(encrypted1, encrypted2);
    }

    #[test]
    fn test_password_case_sensitivity() {
        let test_dir = TempDir::new().unwrap();
        let test_path = test_dir.path();

        let input_file = test_path.join("test.txt");
        let encrypted_file = test_path.join("encrypted.bin");
        let decrypted_file = test_path.join("decrypted.txt");

        fs::write(&input_file, b"Test data").unwrap();

        // Encrypt with lowercase password
        FileCrypto::encrypt_file(
            input_file.to_str().unwrap(),
            encrypted_file.to_str().unwrap(),
            "password",
        )
        .unwrap();

        // Try to decrypt with uppercase password
        let result = FileCrypto::decrypt_file(
            encrypted_file.to_str().unwrap(),
            decrypted_file.to_str().unwrap(),
            "PASSWORD",
        );

        // Should fail because passwords are case-sensitive
        assert!(result.is_err());
    }
}
